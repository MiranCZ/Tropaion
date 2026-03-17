use crate::analysis::type_registry::TypeRegistry;
use crate::ast::walking::visitor::Visitor;
use crate::compiler::compiler::Compiler;
use crate::interpreter::interpreter::Interpreter;
use crate::parser::Parser;
use intrinsics::builtins::builtin_injector::inject_builtins;
use std::time::Instant;

pub mod lexer;
pub mod parser;
mod ast;
pub mod error;
pub mod analysis;
mod compiler;
pub mod interpreter;
mod util;
mod intrinsics;

#[test]
pub fn main() {
    let text = r#"
    fn main() -> int {
        let v = Vec(2, 0, __heap_alloc(2));

        v.push(77);
        v.push(20);
        v.push(5);
        v.push(6);
        v.push(7);
        v.push(8);
        v.pop();
        v.push(50);
        return v.pop();
    }
    "#;

    let code = r#"
    struct Fuck() {
        fn test() -> int {
            return Box(109).get_value();
        }
    }

    fn main() -> int {
        return Fuck().test();
    }

    struct Box<T>(value: T) {
        fn get_value() -> T{
            return value;
        }
    }
    "#;


    interpret(code.to_string());
}

pub fn get_interpreter_for(mut text: String) -> Interpreter {
    inject_builtins(&mut text);

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

fn interpret(mut text: String) {
    inject_builtins(&mut text);

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

    let total_errors = lexer.errors.len() + parser.errors.len() + analyzer.errors.len();

    if total_errors != 0 {
        panic!("Exited with {total_errors} errors");
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

    println!("{functions:?}");

    // println!("{:?}", functions);
    // println!();



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


