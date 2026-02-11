pub mod lexer;

#[test]
pub fn main() {

    let text = r#"
    fn main(a: &int) -> bool {
        let x = 5;
        // hello, I am a comment
        let y = "bye!";
        let test = "I am a \"quoated\" string";

        return false;
    }
        /*
        multiline
        comment
        yeppie
        */
    "#;

    let mut lexer = lexer::Lexer::new(text.to_string());

    println!("Tokenization of: \n{text}");
    loop {
        let token = lexer.read_next();

        println!("\t{token:?}");

        if token == lexer::token::Token::EOF {
            break;
        }
    }
}


