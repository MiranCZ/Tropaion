use crate::parser::Parser;

pub mod lexer;
mod parser;
mod ast;
pub mod error;
pub mod analysis;

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
        let p = check + 1; // should error

        x *= z;

        return false;
    }
        /*
        multiline
        comment
        yeppie
        */
    "#;

    // let text = "let x: &[[(int, &float); 12]; 50] = -1 + 2 * 3;";

    // let text = r#"
    // const x = test() + call() + 1;
    //
    // fn call() -> int {
    //     return 5;
    // }
    //
    // fn test() -> int {
    //     return 1;
    // }
    // "#;

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

            analyzer.analyze();
        }
        Err(e) => panic!("{e}")
    }

    // loop {
    //     let token = lexer.read_next();
    //
    //     println!("\t{token:?}");
    //
    //     if token == lexer::token::Token::EOF {
    //         break;
    //     }
    // }
}


