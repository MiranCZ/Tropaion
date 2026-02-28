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
    fn main(a: &int) -> bool {
        let x = 5;
        // hello, I am a comment
        let y = "bye!";
        let test = "I am a \"quoated\" string";

        let z = 10;
        z++;

        let check = x < z;
        // let p = check + 1; // should error - it now does

        x *= z;

        return false;
    }
        /*
        multiline
        comment
        yeppie
        */
    "#;

    let text = r#"
    struct A(b: B?, i: int);
    struct B(a: A);

    fn create_a() -> A {
        let a = A(null, 5);
        let b = B(a);

        a.b = b;

        return a;
    }


    fn main() -> int {
        let a = create_a();

        return a.i;
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

            for i in instructions.iter() {
                println!("{i:?}");
            }

            println!();

            let mut interpret = Interpreter::new(instructions, functions);

            let result = interpret.run_function("main_".to_string());

            println!("RESULT: {result:?}")
        }
        Err(e) => panic!("{e}")
    }
}


