use crate::analysis::type_registry::TypeRegistry;
use crate::ast::ast_type::AstType::Void;
use crate::ast::statement::Statement::FunctionStmt;
use crate::ast::statement::{Statement, TypedStmt, UntypedStmt};
use crate::compiler::compiler::{CompilationResult, Compiler};
use crate::error::analysis_error::AnalysisError;
use crate::error::compilation_error::CompilationError;
use crate::error::context::{ErrorContext, Errors, Span};
use crate::error::lexer_error::LexerError;
use crate::error::parser_error::ParserError;
use crate::error::runtime_error::RuntimeError;
use crate::error::Error;
use crate::interpreter::heap::Heap;
use crate::interpreter::interpreter::Interpreter;
use crate::interpreter::value::Value;
use crate::lexer::{Lexer, TokenInfo};
use crate::parser::Parser;
use crate::util::spanned::Spanned;
use intrinsics::builtins::builtin_injector::inject_builtins;

pub mod lexer;
pub mod parser;
mod ast;
pub mod error;
pub mod analysis;
mod compiler;
pub mod interpreter;
mod util;
mod intrinsics;

#[cfg(test)]
mod testing;


pub fn lex_code(code: &mut String) -> (Vec<TokenInfo>, Errors<LexerError>) {
    inject_builtins(code);

    let mut lexer = Lexer::new(code.clone());
    let result = lexer.parse();

    (result, lexer.errors)
}

pub fn parse_tokens(tokens: Vec<TokenInfo>, registry: &mut TypeRegistry, entry_point: &str, arguments: Vec<i32>) -> (UntypedStmt, Errors<ParserError>) {
    let mut parser = Parser::new(tokens);

    let mut v = parser.parse(registry);

    // inject entry_point
    // TODO add arguments
    if let Statement::BlockStmt{body} = &mut v.node {
        body.push(Spanned::of(FunctionStmt {
            name: "#__start".to_string(),
            generics: vec![],
            params: vec![],
            return_type: registry.register(Void),
            body: vec![parse_isolated_string(format!("return {entry_point}();"))]
        }, Span::new(0,0)));
    }

    (v, parser.errors)
}

pub fn resolve_types(untyped: UntypedStmt, registry: &mut TypeRegistry) -> (TypedStmt, Errors<AnalysisError>) {
    let mut analyzer = analysis::analyzer::Analyzer::new(untyped);

    let resolved_root = analyzer.analyze(registry);

    (resolved_root, analyzer.errors)
}

pub fn compile(typed: TypedStmt, registry: &mut TypeRegistry, code: &String) -> Result<CompilationResult, CompilationError> {
    let comp = Compiler::new(typed, code.chars().collect());

    comp.compile(registry)
}

pub fn run_compiled(compilation_result: CompilationResult) -> Result<(Vec<Value>, Heap), ErrorContext<RuntimeError>> {
    let interpret = Interpreter::new(compilation_result);

    interpret.run_function("#__start_".to_string())
}


pub fn run_code(code: String, entry_point: &str) -> Result<(Vec<Value>, Heap), Errors<Box<dyn Error>>> {
    run_code_with_args(code, entry_point, vec![])
}

pub fn run_code_with_args(mut code: String, entry_point: &str, arguments: Vec<i32>) -> Result<(Vec<Value>, Heap), Errors<Box<dyn Error>>> {
    let (tokens, lexer_errors) = lex_code(&mut code);

    let mut registry = TypeRegistry::new();
    let (parsed, parser_errors) = parse_tokens(tokens, &mut registry, entry_point, arguments);

    let (typed, analysis_errors) = resolve_types(parsed, &mut registry);

    let mut errors: Vec<ErrorContext<Box<dyn Error>>> = vec![];


    // FIXME create a closure for this... idk how tho
    for err in lexer_errors {
        let ctx: ErrorContext<Box<dyn Error>> = ErrorContext { error: Box::new(err.error), span: err.span, message: err.message };
        errors.push(ctx)
    }
    for err in parser_errors {
        let ctx: ErrorContext<Box<dyn Error>> = ErrorContext { error: Box::new(err.error), span: err.span, message: err.message };
        errors.push(ctx)
    }
    for err in analysis_errors {
        let ctx: ErrorContext<Box<dyn Error>> = ErrorContext { error: Box::new(err.error), span: err.span, message: err.message };
        errors.push(ctx)
    }


    if !errors.is_empty() {
        return Err(errors);
    }

    let compiled = compile(typed, &mut registry, &code);

    match compiled {
        Ok(compilation_result) => {
            let run_result =  run_compiled(compilation_result);

            match run_result {
                Ok(value) => Ok(value),
                Err(err) => {
                    let ctx: ErrorContext<Box<dyn Error>> = ErrorContext { error: Box::new(err.error), span: err.span, message: err.message };
                    Err(vec![ctx])
                }
            }
        }
        Err(e) => {
            Err(vec![ErrorContext::new(Box::new(e), 0,0)])
        }
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


