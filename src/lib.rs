use std::io::stderr;
use std::time::Instant;
use crate::analysis::symbol_table::SymbolTable;
use crate::analysis::type_registry::TypeRegistry;
use crate::ast::ast_type::AstType;
use crate::ast::ast_type::AstType::SymbolType;
use crate::ast::walking::visitor::Visitor;
use crate::compiler::compiler::Compiler;
use crate::interpreter::heap::Heap;
use crate::interpreter::interpreter::Interpreter;
use crate::interpreter::value::Value;
use crate::parser::Parser;

pub mod lexer;
pub mod parser;
mod ast;
pub mod error;
pub mod analysis;
mod compiler;
pub mod interpreter;
mod util;

#[test]
pub fn main() {
    let text = r#"
    struct Scope<T>() {
        fn box(value: T) -> T {
            return value;
        }
    }

    fn main() -> int {
        let a = Scope().box(5);

        return a;
    }
    "#;


    interpret(text);
}

pub fn get_interpreter_for(text: String) -> Interpreter {
    let mut lexer = lexer::Lexer::new(text.to_string());

    let tokens = lexer.parse();

    if !lexer.errors.is_empty() {
        for e in lexer.errors.iter() {
            eprintln!("{}\n", e.format(text.chars().collect()))
        }

        panic!("Exited with {} errors", lexer.errors.len());
    }


    let mut registry = TypeRegistry::new();

    let mut parser = Parser::new(tokens);

    let parsed = parser.parse(&mut registry);

    if !parser.errors.is_empty() {
        for e in parser.errors.iter() {
            eprintln!("{}\n", e.format(text.chars().collect()))
        }

        panic!("Exited with {} errors", parser.errors.len());
    }


    let mut analyzer = analysis::analyzer::Analyzer::new(parsed);

    let resolved_root = analyzer.analyze(&mut registry);

    if !analyzer.errors.is_empty() {
        for e in analyzer.errors.iter() {
            eprintln!("{}\n", e.format(text.chars().collect()))
        }

        panic!("Exited with {} errors", analyzer.errors.len());
    }

    let mut comp = Compiler::new(resolved_root, text.chars().collect());

    let res = comp.compile(&mut registry);

    let (instructions, lines, functions) = if let Ok((i, l, f)) = res {
        (i,l, f)
    } else {
        panic!("Error {:?}", res.err().unwrap());
    };

    let interpret = Interpreter::new(instructions, lines, functions);

    interpret
}

fn interpret(text: &str) {
    let mut lexer = lexer::Lexer::new(text.to_string());

    println!("Tokenization of: \n{text}");

    let tokens = lexer.parse();

    if !lexer.errors.is_empty() {
        eprintln!("--------- Lexer has {} errors ---------- \n", lexer.errors.len());

        for e in lexer.errors.iter() {
            eprintln!("{}\n", e.format(text.chars().collect()))
        }
    }

    // println!("-------");
    // println!();


    let mut parser = Parser::new(tokens);


    let mut registry = TypeRegistry::new();

    let v = parser.parse(&mut registry);

    if !parser.errors.is_empty() {
        eprintln!("--------- Parser has {} errors ---------- \n", parser.errors.len());

        for e in parser.errors.iter() {
            eprintln!("{}\n", e.format(text.chars().collect()))
        }
    }

    // println!("{v:#?}");

    let mut analyzer = analysis::analyzer::Analyzer::new(v);

    let resolved_root = analyzer.analyze(&mut registry);

    if !analyzer.errors.is_empty() {
        eprintln!("--------- Analyzer has {} errors ---------- \n", analyzer.errors.len());

        for e in analyzer.errors.iter() {
            eprintln!("{}\n", e.format(text.chars().collect()))
        }
    }

    // println!("{:#?}", resolved_root);

    let mut comp = Compiler::new(resolved_root, text.chars().collect());

    // println!();
    // println!();
    // println!("{:#?}", registry);
    // println!("-------------------");
    // println!();

    let res = comp.compile(&mut registry);

    let (instructions, lines, functions) = if let Ok((i, l, f)) = res {
        (i, l, f)
    } else {
        panic!("Error {:?}", res.err().unwrap());
    };

    // println!("{:?}", functions);
    // println!();

    let total_errors = lexer.errors.len() + parser.errors.len() + analyzer.errors.len();

    if total_errors != 0 {
        panic!("Exited with {total_errors} errors");
    }

    for i in instructions.iter() {
        println!("{i:?}");
    }

    println!();

    let mut interpret = Interpreter::new(instructions, lines, functions);

    let now = Instant::now();
    let result = interpret.run_function("main_".to_string());

    let result = if let Ok(r) = result {
        r
    } else {
        panic!("{}", result.err().unwrap().format(text.chars().collect()));
    };

    println!("Took {:?}", now.elapsed());
    println!("RESULT: {result:?}")
}


