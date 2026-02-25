use crate::compiler::compiler::Compiler;
use crate::interpreter::interpreter::Interpreter;
use crate::parser::Parser;

pub mod lexer;
mod parser;
mod ast;
pub mod error;
pub mod analysis;
mod compiler;
mod interpreter;

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
    struct Test(a: int, b: int) {
        fn sum() -> int {
            return a + this.b;
        }

    }

    fn main() -> int {
        let t: Test = Test(100, 200);

        let y = true;

        if y {
            t.a = 77;
        }

        let x = t.sum();

        return x;
    }
    "#;

    interpret(text);

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

    let res = parser.parse();

    match res {
        Ok(v) => {
            println!("{v:#?}");

            let mut analyzer = analysis::analyzer::Analyzer::new(v);

            let resolved_root = analyzer.analyze();

            println!("{:#?}", resolved_root);

            let mut comp = Compiler::new(resolved_root);


            println!();
            println!();
            println!("-------------------");
            println!();
            println!();

            comp.compile();

            let mut interpret = Interpreter::new(comp.generator.instructions, comp.generator.functions);

            let result = interpret.run_function("main_".to_string());

            println!("RESULT: {result:?}")
        }
        Err(e) => panic!("{e}")
    }
}


