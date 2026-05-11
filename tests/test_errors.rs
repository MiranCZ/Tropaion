use tropaion::analysis::type_registry::TypeRegistry;
use tropaion::error::analysis_error::AnalysisError;
use tropaion::error::lexer_error::LexerError;
use tropaion::{analysis, lexer};
use tropaion::error::analysis_error::AnalysisError::{NullableAccess, RedundantNullable};
use tropaion::lexer::token::SimpleToken;
use tropaion::parser::Parser;

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

    assert!(parser.errors.is_empty());


    let mut analyzer = analysis::analyzer::Analyzer::new(parsed);

    let resolved_root = analyzer.resolve_types(&mut registry);
    let resolved_root = analyzer.transform_syntax(&mut registry, resolved_root);

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

#[test]
fn test_illegal_access() {
    let code = r#"
    struct Foo(bar: int) {

        fn get_bar() -> int {
            return bar;
        }

    }

    fn main() {
        let f = Foo(10);

        let x = f.get_bar();
    }
    "#;

    test_analysis_error(code, AnalysisError::IllegalFuncArgs {func_name: "get_bar".to_string(), args: String::new()})
}

#[test]
fn test_illegal_access2() {
    let code = r#"
    struct Foo(bar: int) {

        fn larger(mult: int) {
            bar *= mult;
        }

        pub fn larger() {
            larger(10);
        }
    }

    fn main() -> int {
        let f = Foo(5);
        f.larger(); // legal
        f.larger(10); // should error

        return f.bar;
    }
    "#;

    test_analysis_error(code, AnalysisError::IllegalFuncArgs {func_name: "larger".to_string(), args: "int".to_string()})
}

// FIXME these snippets throw multiple errors, rewrite test suite to support that
// #[test]
// fn test_constructor_error1() {
//     let code = r#"
//
//     pub init() {
//         this(0);
//     }
//
//     struct Box(v: int) {
//     }
//
//     fn main() -> int {
//         return 0;
//     }
//     "#;
//
//     test_analysis_error(code, AnalysisError::DanglingConstructor);
// }
//
// #[test]
// fn test_constructor_error2() {
//     let code = r#"
//     struct Box(v: int) {
//         pub init() {
//             this(0);
//             this(5);
//         }
//
//     }
//
//     fn main() -> int {
//         return 0;
//     }
//     "#;
//
//     test_analysis_error(code, AnalysisError::MultipleThisCall);
// }

#[test]
fn test_constructor_error3() {
    let code = r#"
    struct Box(v: int) {
        pub init() {
            if false {
                this(0);
            }
            this(1);
        }

    }

    fn main() -> int {
        return 0;
    }
    "#;

    test_analysis_error(code, AnalysisError::IllegalThis);
}