use std::collections::HashMap;
use std::error::Error;
use std::io::{stdout, Write};
use std::panic::catch_unwind;
use crate::analysis::type_registry::TypeRegistry;
use crate::ast::statement::{TypedStmt, UntypedStmt};
use crate::compiler::compiler::{CompilationResult, Compiler};
use crate::error::analysis_error::AnalysisError;
use crate::error::compilation_error::CompilationError;
use crate::error::context::{ErrorContext, Errors};
use crate::error::lexer_error::LexerError;
use crate::error::parser_error::ParserError;
use crate::interpreter::interpreter::Interpreter;
use crate::lexer::{Lexer, TokenInfo};
use crate::memory_blob::MemoryBlob;
use crate::parser::Parser;
use crate::util::arg_convertor::ValueConvertable;
use intrinsics::builtins::builtin_injector::inject_builtins;
use crate::completion::code_completion::get_code_suggestions;
use crate::completion::completion_type::CompletionType;
use crate::error::error_type::ErrorType;
use crate::error::state_error::StateError;
use crate::interpreter::interpreter_builder::InterpreterBuilder;
use crate::interpreter::runtime_error_context::RuntimeErrorContext;
use crate::util::ast_printer::AstPrinter;

pub mod lexer;
pub mod parser;
pub mod ast;
pub mod error;
pub mod analysis;
pub mod compiler;
pub mod interpreter;
pub mod util;
mod intrinsics;

#[cfg(test)]
mod testing;
pub mod memory_blob;
pub mod completion;

pub type RunResult = Result<MemoryBlob, ErrorType<Errors<Box<dyn Error>>, RuntimeErrorContext>>;


pub fn lex_code(code: &mut String) -> (Vec<TokenInfo>, Errors<LexerError>) {
    inject_builtins(code);

    let mut lexer = Lexer::new(code.clone());
    let result = lexer.parse();

    (result, lexer.errors)
}

pub fn parse_tokens(tokens: Vec<TokenInfo>, registry: &mut TypeRegistry) -> (UntypedStmt, Errors<ParserError>) {
    let mut parser = Parser::new(tokens);

    let untyped = parser.parse(registry);

    (untyped, parser.errors)
}

pub fn resolve_types_and_transform(untyped: UntypedStmt, registry: &mut TypeRegistry) -> (TypedStmt, Errors<AnalysisError>) {
    let mut analyzer = analysis::analyzer::Analyzer::new(untyped);

    let resolved_root = analyzer.resolve_types(registry);
    let transformed = analyzer.transform_syntax(registry, resolved_root);

    (transformed, analyzer.errors)
}

pub fn compile_typed(typed: TypedStmt, registry: &mut TypeRegistry, code: &String) -> Result<CompilationResult, CompilationError> {
    let comp = Compiler::new(typed, code.chars().collect());

    comp.compile(registry)
}


pub fn compile(code: String) -> Result<CompilationResult, Errors<Box<dyn Error>>> {
    let res = catch_unwind(|| _compile(code));

    res.unwrap_or_else(
        |_| Err(vec![ErrorContext::unknown(Box::new(StateError::InternalError))])
    )
}

fn _compile(mut code: String) -> Result<CompilationResult, Errors<Box<dyn Error>>> {
    let (tokens, lexer_errors) = lex_code(&mut code);

    let mut registry = TypeRegistry::new();
    let (parsed, parser_errors) = parse_tokens(tokens, &mut registry);

    let (typed, analysis_errors) = resolve_types_and_transform(parsed, &mut registry);

    let mut errors: Vec<ErrorContext<Box<dyn Error>>> = vec![];


    // FIXME create a closure for this... idk how tho
    for err in lexer_errors {
        let ctx: ErrorContext<Box<dyn Error>> = ErrorContext { error: Box::new(err.error), span_type: err.span_type, message: err.message };
        errors.push(ctx)
    }
    for err in parser_errors {
        let ctx: ErrorContext<Box<dyn Error>> = ErrorContext { error: Box::new(err.error), span_type: err.span_type, message: err.message };
        errors.push(ctx)
    }
    for err in analysis_errors {
        let ctx: ErrorContext<Box<dyn Error>> = ErrorContext { error: Box::new(err.error), span_type: err.span_type, message: err.message };
        errors.push(ctx)
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    let compiled = compile_typed(typed, &mut registry, &code);

    if let Err(e) = compiled {
        let ctx: ErrorContext<Box<dyn Error>> = ErrorContext::unknown(Box::new(e));
        return Err(vec![ctx]);
    } else if let Ok(res) = compiled {
        return Ok(res);
    }

    unreachable!()
}

pub fn lint(mut code: String, cursor: usize) -> (HashMap<String, CompletionType>, Errors<Box<dyn Error>>) {
    let (tokens, lexer_errors) = lex_code(&mut code);

    let mut registry = TypeRegistry::new();
    let (parsed, parser_errors) = parse_tokens(tokens, &mut registry);

    let mut analyzer = analysis::analyzer::Analyzer::new(parsed);

    let resolved_root = analyzer.resolve_types(&mut registry);

    println!("{}", AstPrinter::new(Some(&registry)).print_statement(&resolved_root));
    let suggestions = get_code_suggestions(cursor,&resolved_root, &mut registry);

    let typed = analyzer.transform_syntax(&mut registry, resolved_root);

    let analysis_errors =  analyzer.errors;

    let mut errors: Vec<ErrorContext<Box<dyn Error>>> = vec![];

    // FIXME create a closure for this... idk how tho
    for err in lexer_errors {
        let ctx: ErrorContext<Box<dyn Error>> = ErrorContext { error: Box::new(err.error), span_type: err.span_type, message: err.message };
        errors.push(ctx)
    }
    for err in parser_errors {
        let ctx: ErrorContext<Box<dyn Error>> = ErrorContext { error: Box::new(err.error), span_type: err.span_type, message: err.message };
        errors.push(ctx)
    }
    for err in analysis_errors {
        let ctx: ErrorContext<Box<dyn Error>> = ErrorContext { error: Box::new(err.error), span_type: err.span_type, message: err.message };
        errors.push(ctx)
    }


    let compiled = compile_typed(typed, &mut registry, &code);
    if let Err(e) = compiled {
        let ctx: ErrorContext<Box<dyn Error>> = ErrorContext::unknown(Box::new(e));
        errors.push(ctx);
    }


    (suggestions, errors)
}

pub fn run_compiled(interpreter: &mut Interpreter, entry_point: &str, args: Vec<ValueConvertable>, out: &mut impl Write) -> Result<MemoryBlob, RuntimeErrorContext> {
    let mut mangled = format!("{entry_point}_");

    for a in args.iter() {
        mangled.push_str(a.get_mangled().as_str());
    }

    interpreter.run_function(mangled, args, out)
}


pub fn run_code(code: String, entry_point: &str) -> RunResult {
    run_code_with_out(code, entry_point, &mut stdout())
}

pub fn run_code_with_out(code: String, entry_point: &str, out: &mut impl Write) -> RunResult {
    run_code_with_args(code, entry_point, vec![], out)
}

pub fn run_code_with_args(code: String, entry_point: &str, arguments: Vec<ValueConvertable>, out: &mut impl Write) -> RunResult {
    let compilation_result = match compile(code) {
        Ok(v) => v,
        Err(e) => return Err(ErrorType::COMPILETIME(e))
    };

    let mut interpreter = InterpreterBuilder::new(compilation_result).build();
    let run_result =  run_compiled(&mut interpreter, entry_point, arguments, out);

    match run_result {
        Ok(value) => Ok(value),
        Err(err) => Err(ErrorType::RUNTIME(err))
    }
}

// FIXME handle errors
fn parse_isolated_string(str: String) -> UntypedStmt {
    let mut lexer = Lexer::new(str.to_string());

    let tokens = lexer.parse();

    if !lexer.errors.is_empty() {
        eprintln!("--------- Lexer has {} errors ---------- \n", lexer.errors.len());

        for e in lexer.errors.iter() {
            eprintln!("{}\n", e.format(str.chars().collect()))
        }
    }

    let mut parser = Parser::new(tokens);
    let mut registry = TypeRegistry::new();

    parser.parse(&mut registry)
}


