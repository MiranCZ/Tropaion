use crate::analysis::type_registry::TypeRegistry;
use crate::{compile, compile_typed, lex_code, lint, parse_tokens, resolve_types_and_transform, run_compiled};
use std::time::Instant;
use crate::interpreter::interpreter::Interpreter;
use crate::interpreter::interpreter_builder::InterpreterBuilder;
use crate::util::arg_convertor::{into_arg, struct_convertor, ValueConvertable, ValueLike};
use crate::util::ast_printer::AstPrinter;

#[test]
pub fn main() {
    let text = r#"
    fn step(game: Game) -> Direction {
    let head = game.snake.head();
    let reversed = head.x > (game.width / 2);

    if (reversed) {
        head.x -= game.width - 2;
    }

    if (head.y == 0) {
        if (head.x == 0) {
            return Direction.RIGHT;
        } else if (head.x == 1) {
            return Direction.DOWN;
        }
    } else if (head.y == 1) {
        if (head.x == 0) {
            return Direction.UP;
        } else if (head.x == 1) {
            return Direction.LEFT;
        }
    }

    if (head.y > 1) {
        return Direction.UP;
    }

    if (reversed) {
        return Direction.RIGHT;
    } else {
        return Direction.LEFT;a
    }
}

struct Point(
    x: int,
    y: int,
);

struct Snake(
    points: Vec<Point>,
    direction: Direction,
) {
    pub fn head() -> Point {
        return this.points.get(0);
    }

    pub fn tail() -> Point {
        return this.points.get(this.points.size() - 1);
    }

    pub fn size() -> int {
        return this.points.size();
    }
}

struct Game(
    width: int,
    height: int,
    snake: Snake,
    opponents: Vec<Snake>,
    apples: Vec<Point>,
);

    "#;

    interpret(text.to_string());
    panic!();

    let cursor = 676;
    let (suggestions, errors) = lint(text.to_string(), cursor);

    for x in suggestions {
        println!("{x:?}");
    }

    for e in errors {
        let msh = e.format(text.chars().collect());
        println!("{msh}");
    }

    for (i, ch) in text.chars().enumerate() {
        if i == cursor {
            print!("#");
        } else {
            print!("{ch}");
        }
    }

    println!("CURSOR {cursor}")
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

    let (resolved_root, analyzer_errors) = resolve_types_and_transform(v, &mut registry);

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

    println!("{}", AstPrinter::new(Some(&registry)).print_statement(&resolved_root));

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


    let points = vec![Point{x: 10, y: 66}, Point{x: 0, y:1}, Point{x: 100, y: 200}];

    let mut interpret = InterpreterBuilder::new(compilation_res).heap_size(100).max_instruction_cost(10_000_000).build();
    let result = run_compiled(&mut interpret, "main", vec![], &mut std::io::stdout());

    let result = if let Ok(r) = result {
        r
    } else {
        panic!("{}", result.err().unwrap().format(text.chars().collect()));
    };

    println!("Took {:?}", now.elapsed());
    println!("RESULT: {result:?}")
}

struct Point {
    x: i32,
    y: i32
}

impl ValueLike for Point {
    fn into_convertable(self) -> ValueConvertable {
        let mut struct_arg = struct_convertor("Point");

        struct_arg.add_field(self.x);
        struct_arg.add_field(self.y);

        struct_arg.convert()
    }
}


