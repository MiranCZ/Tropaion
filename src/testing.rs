use crate::analysis::type_registry::TypeRegistry;
use crate::{compile, compile_typed, lex_code, parse_tokens, resolve_types, run_compiled};
use std::time::Instant;
use crate::interpreter::interpreter::Interpreter;
use crate::util::arg_convertor::into_arg;

#[test]
pub fn main() {
    let text = r#"
    fn main() {
        let x = Direction.LEFT;

        let y = 100;
        print(x.to_string());
        print(x.opposite().to_string());
        print(x.rot_clockwise().to_string());
    }a"#;

    interpret(text.to_string());
}

fn interpret(mut text: String) {
    println!("Tokenization of: \n{}", &text);

    let (tokens, lexer_errors) = lex_code(&mut text);

    if !lexer_errors.is_empty() {
        eprintln!("--------- Lexer has {} errors ---------- \n", lexer_errors.len());

        for e in lexer_errors.iter() {
            eprintln!("{}\n", e.format(text.chars().collect()))
        }
    }

    // println!("-------");
    // println!();

    let mut registry = TypeRegistry::new();

    let (v, parser_errors) = parse_tokens(tokens, &mut registry);

    if !parser_errors.is_empty() {
        eprintln!("--------- Parser has {} errors ---------- \n", parser_errors.len());

        for e in parser_errors.iter() {
            eprintln!("{}\n", e.format(text.chars().collect()))
        }
    }

    // println!("{v:#?}");

    let (resolved_root, analyzer_errors) = resolve_types(v, &mut registry);

    if !analyzer_errors.is_empty() {
        eprintln!("--------- Analyzer has {} errors ---------- \n", analyzer_errors.len());

        for e in analyzer_errors.iter() {
            eprintln!("{}\n", e.format(text.chars().collect()))
        }
    }

    let total_errors = lexer_errors.len() + parser_errors.len() + analyzer_errors.len();

    if total_errors != 0 {
        panic!("Exited with {total_errors} errors");
    }

    // println!("{:#?}", resolved_root);


    // println!();
    // println!();
    // println!("{:#?}", registry);
    // println!("-------------------");
    // println!();

    let res = compile_typed(resolved_root, &mut registry, &text);

    let compilation_res = if let Ok(r) = res {
        r
    } else {
        panic!("Error {:?}", res.err().unwrap());
    };

    println!("{:?}", compilation_res.functions);

    // println!("{:?}", functions);
    // println!();



    for i in compilation_res.instructions.iter() {
        println!("{i:?}");
    }

    println!();

    let now = Instant::now();
    let result = run_compiled(&mut Interpreter::new(compilation_res), "main", vec![], &mut std::io::stdout());

    let result = if let Ok(r) = result {
        r
    } else {
        panic!("{}", result.err().unwrap().format(text.chars().collect()));
    };

    println!("Took {:?}", now.elapsed());
    println!("RESULT: {result:?}")
}


