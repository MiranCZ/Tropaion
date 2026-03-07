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
    fn main() -> int {
        let x: int? = null;
        x = 5;

        let y = x!! + 3;

        return y;
    }
    "#;

    interpret(text);

}

pub fn get_interpreter_for(text: String) -> Interpreter {
    let mut lexer = lexer::Lexer::new(text.to_string());

    let tokens = lexer.parse();

    if let Err(e) = tokens {
        panic!("{}", e.format(text.chars().collect()));
    }
    
    let tokens = tokens.unwrap();

    let mut registry = TypeRegistry::new();

    let mut parser = Parser::new(tokens);

    let parsed = parser.parse(&mut registry);

    if let Err(e) = parsed {
        panic!("{}", e.format(text.chars().collect()));
    }

    let parsed = parsed.unwrap();

    let mut analyzer = analysis::analyzer::Analyzer::new(parsed);

    let resolved_root = analyzer.analyze(&mut registry);

    let resolved_root = if let Ok(v) = resolved_root {
        v
    } else {
        panic!("{}", resolved_root.err().unwrap().format(text.chars().collect()));
    };

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

    println!("-------");
    println!();

    if let Err(e) = tokens {
        panic!("{}", e.format(text.chars().collect()));
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

            let resolved_root = if let Ok(v) = resolved_root {
                v
            } else {
                panic!("{}", resolved_root.err().unwrap().format(text.chars().collect()));
            };

            println!("{:#?}", resolved_root);

            let mut comp = Compiler::new(resolved_root, text.chars().collect());


            println!();
            println!();
            println!("{:#?}", registry);
            println!("-------------------");
            println!();

            let res = comp.compile(&mut registry);

            let (instructions, lines, functions) = if let Ok((i, l, f)) = res {
                (i,l, f)
            } else {
                panic!("Error {:?}", res.err().unwrap());
            };

            println!("{:?}", functions);
            println!();

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
        Err(e) => panic!("{}", e.format(text.chars().collect()))
    }
}


