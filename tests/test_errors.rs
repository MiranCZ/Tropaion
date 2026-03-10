use Tropaion::analysis::type_registry::TypeRegistry;
use Tropaion::error::analysis_error::AnalysisError;
use Tropaion::error::lexer_error::LexerError;
use Tropaion::{analysis, lexer};
use Tropaion::error::analysis_error::AnalysisError::{NullableAccess, RedundantNullable};
use Tropaion::lexer::token::SimpleToken;
use Tropaion::parser::Parser;

fn test_lexer_error(code: &str, expected: LexerError) {
    let mut lexer = lexer::Lexer::new(code.to_string());
    let res = lexer.parse();

    assert_eq!(lexer.errors.len(), 1);

    assert_eq!(lexer.errors[0].error, expected);
}

fn test_analysis_error(code: &str, expected: AnalysisError) {
    let mut lexer = lexer::Lexer::new(code.to_string());

    let tokens = lexer.parse();

    assert!(lexer.errors.is_empty());

    let mut registry = TypeRegistry::new();

    let mut parser = Parser::new(tokens);

    let parsed = parser.parse(&mut registry);

    if let Err(e) = parsed {
        panic!("{}", e.format(code.chars().collect()));
    }

    let parsed = parsed.unwrap();

    let mut analyzer = analysis::analyzer::Analyzer::new(parsed);

    let resolved_root = analyzer.analyze(&mut registry);

    assert_eq!(analyzer.errors.len(), 1);

    assert_eq!(analyzer.errors[0].error, expected);
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

#[test]
fn test_double_nullable() {
    let code = r#"
    fn main() {
        let x: int?? = null;
    }
    "#;

    test_analysis_error(code, RedundantNullable);
}

#[test]
fn test_unsafe_call() {
    let code = r#"
    struct A(i: int);

    fn do_stuff(inp: bool) -> int {
        let a: A? = A(5);

        if inp {
            a = null;
        }

        return a.i;
    }

    fn main() {
        do_stuff(false);
    }
    "#;

    test_analysis_error(code, NullableAccess);
}

#[test]
fn test_illegal_deconstruct() {
    let code = r#"
    fn main() -> int {
        let a: int? = null;
        let b: int? = 10;

        // `int?` ?? `int?` should not be allowed
        let c = b ?? a;

        return c;
    }
    "#;

    test_analysis_error(code, AnalysisError::IllegalBinaryExpression {
        left: "int?".to_string(),
        op: SimpleToken::TwoQuestion,
        right: "int?".to_string()
    });
}