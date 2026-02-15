use crate::parser::Parser;

pub mod lexer;
mod parser;
mod ast;
pub mod error;

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

        x *= z;

        return false;
    }
        /*
        multiline
        comment
        yeppie
        */
    "#;

    let text = "let x = 1 + 2 * 3;";

    let mut lexer = lexer::Lexer::new(text.to_string());

    println!("Tokenization of: \n{text}");

    let tokens = lexer.parse();

    println!("-------");
    println!();

    if tokens.is_err() {
        panic!("GOT PARSER ERROR {tokens:?}");
    }
    let tokens = tokens.unwrap();

    let mut parser = Parser::new(tokens);

    let res = parser.parse();

    println!("{res:#?}");
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


