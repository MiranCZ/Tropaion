use std::time::Instant;
use crate::analysis::symbol_table::SymbolTable;
use crate::analysis::type_registry::TypeRegistry;
use crate::ast::ast_type::AstType;
use crate::ast::ast_type::AstType::SymbolType;
use crate::compiler::compiler::Compiler;
use crate::interpreter::heap::Heap;
use crate::interpreter::interpreter::Interpreter;
use crate::interpreter::value::Value;
use crate::parser::Parser;

pub mod lexer;
mod parser;
mod ast;
pub mod error;
pub mod analysis;
mod compiler;
pub mod interpreter;

#[test]
pub fn main() {
    let text = r#"
    struct Rect(a: int, b: int) {

        fn value(n: int) -> int {
            return n * (a + b);
        }

    }

    fn main() -> int {
        let t = Rect(5, 10);

        return t.value(3);
    }
    "#;

    interpret(text);

}

pub fn get_interpreter_for(text: String) -> Interpreter {
    let mut lexer = lexer::Lexer::new(text.to_string());

    let tokens = lexer.parse();

    if let Err(e) = tokens {
        panic!("{e}");
    }
    
    let tokens = tokens.unwrap();

    let mut registry = TypeRegistry::new();

    let mut parser = Parser::new(tokens);

    let parsed = parser.parse(&mut registry);

    if let Err(e) = parsed {
        panic!("{e}");
    }

    let parsed = parsed.unwrap();

    let mut analyzer = analysis::analyzer::Analyzer::new(parsed);

    let resolved_root = analyzer.analyze(&mut registry);

    let mut comp = Compiler::new(resolved_root);

    let (instructions, functions) = comp.compile(&mut registry);

    let interpret = Interpreter::new(instructions, functions);

    interpret
}

fn interpret(text: &str) {
    let mut lexer = lexer::Lexer::new(text.to_string());

    println!("Tokenization of: \n{text}");

    let tokens = lexer.parse();

    println!("-------");
    println!();

    if let Err(e) = tokens {
        panic!("{e}");
    }
    let tokens = tokens.unwrap();

    let mut parser = Parser::new(tokens);


    let mut registry = TypeRegistry::new();

    let res = parser.parse(&mut registry);

    match res {
        Ok(v) => {
            println!("{v:#?}");

            let mut analyzer = analysis::analyzer::Analyzer::new(v);


            let resolved_root = analyzer.analyze(&mut registry);

            println!("{:#?}", resolved_root);

            let mut comp = Compiler::new(resolved_root);


            println!();
            println!();
            println!("{:#?}", registry);
            println!("-------------------");
            println!();

            let (instructions, functions) = comp.compile(&mut registry);

            println!("{:?}", functions);
            println!();

            for i in instructions.iter() {
                println!("{i:?}");
            }

            println!();

            let mut interpret = Interpreter::new(instructions, functions);

            let now = Instant::now();
            let result = interpret.run_function("main_".to_string());
            println!("Took {:?}", now.elapsed());
            println!("RESULT: {result:?}")
        }
        Err(e) => panic!("{e}")
    }
}


