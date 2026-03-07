use Tropaion::analysis::type_registry::TypeRegistry;
use Tropaion::error::analysis_error::AnalysisError;
use Tropaion::error::lexer_error::LexerError;
use Tropaion::{analysis, lexer};
use Tropaion::parser::Parser;

fn test_lexer_error(code: &str, expected: LexerError) {
    let mut lexer = lexer::Lexer::new(code.to_string());
    let res = lexer.parse();

    if let Err(e) = res {
        assert_eq!(e.error, expected);
    } else {
        panic!("Error expected")
    }
}

fn test_analysis_error(code: &str, expected: AnalysisError) {
    let mut lexer = lexer::Lexer::new(code.to_string());

    let tokens = lexer.parse();

    if let Err(e) = tokens {
        panic!("{}", e.format(code.chars().collect()));
    }

    let tokens = tokens.unwrap();

    let mut registry = TypeRegistry::new();

    let mut parser = Parser::new(tokens);

    let parsed = parser.parse(&mut registry);

    if let Err(e) = parsed {
        panic!("{}", e.format(code.chars().collect()));
    }

    let parsed = parsed.unwrap();

    let mut analyzer = analysis::analyzer::Analyzer::new(parsed);

    let resolved_root = analyzer.analyze(&mut registry);

    if let Err(e) = resolved_root {
        assert_eq!(e.error, expected);
    } else {
        panic!("Error expected")
    }
}


#[test]
fn test_unknown_symbol() {
    let code = r#"
    fn main() {
        @
    }
    "#;

    test_lexer_error(code, LexerError::UnknownToken('@'));
}

#[test]
fn test_unterminated() {
    let code = r#"
    fn main() {
        // "
        "
    }
    "#;

    test_lexer_error(code, LexerError::UnclosedString);

    let code = r#"
    fn main() {
    /*

        let x = 5 * 6;
    }
    "#;

    test_lexer_error(code, LexerError::UnclosedComment);
}
