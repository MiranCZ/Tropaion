use Tropaion::get_interpreter_for;
use Tropaion::interpreter::value::Value;
use Tropaion::interpreter::value::Value::IntValue;


fn test_simple_code(main: &str, code: &str, expected: i32) {
    let main = main.to_owned() + "_"; // FIXME should really fix the trailing `_` at some point bruh

    let mut interpret = get_interpreter_for(code.to_string());

    let (stack, heap) = interpret.run_function(main.to_string());


    assert_eq!(stack.len(), 2); // nullptr, value
    assert_eq!(stack[1], IntValue(expected));
}

fn test_math_expr(expr: &str, expected: i32) {
    let text = r#"
    fn main() -> int {
        return "#.to_owned() +expr+r#";
    }
    "#;
    let mut interpret = get_interpreter_for(text.to_string());

    let (stack, heap) = interpret.run_function("main_".to_string());

    assert_eq!(stack.len(), 2); // nullptr, value
    assert_eq!(stack[1], IntValue(expected), "Failed for expression '{} = {}'", expr, expected);
}

#[test]
pub fn test_math() {
    test_math_expr("1 + 1", 2);
    test_math_expr("1 - 1", 0);
    test_math_expr("-10", -10);
    test_math_expr("1 - -1", 2);
    test_math_expr("1 - 3", -2);
    test_math_expr("1 * 2", 2);
    test_math_expr("1 + 2 * 3", 7);
    test_math_expr("(1 + 2) * 3", 9);
    test_math_expr("(1 + 2) / 3", 1);
    test_math_expr("5/3", 1);
    test_math_expr("6/3", 2);
    test_math_expr("6 % 2", 0);
    test_math_expr("7 % 2", 1);
    test_math_expr("20 % 36", 20);
}

#[test]
pub fn test_if() {
    let if_stmt = r#"
    fn main() -> int {
        let x = 5;

        let y = true;

        if y {
            x -= 10;
        }

        return x;
    }
    "#;

    test_simple_code("main", if_stmt, -5);
}

#[test]
pub fn test_loops() {
    let loop_stmt = r#"
    fn main() -> int {
        let x = 17;

        let i = 0;

        while i < 10 {
            x += i;

            i++;
        }

        return x;
    }
    "#;

    test_simple_code("main", loop_stmt, 62);
}

#[test]
pub fn test_overloads() {
    let loop_stmt = r#"
    fn other(i: int) -> int{
        return i;
    }

    fn other(i: float) -> float {
        return i;
    }


    fn main() -> int {
        let x = 1 + other(2);

        return x;
    }
    "#;

    test_simple_code("main", loop_stmt, 3);
}


#[test]
pub fn test_nulls() {
    let loop_stmt = r#"
    fn main() -> int {
        let x: int? = null;

        let y = 5;

        if y > 1 {
            x = 7;
        }

        return x+y;
    }
    "#;

    test_simple_code("main", loop_stmt, 12);
}

// TODO test nullptr panics


#[test]
fn test_struct_promotion() {
    let code = r#"
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

        return a.i + 10;
    }
    "#;


    test_simple_code("main", code, 15);
}

#[test]
fn test_recursion() {
    let code = r#"
    fn fib(n: int) -> int {
        if n == 0 {
            return 0;
        }
        if n == 1 {
            return 1;
        }

        return fib(n-1) + fib(n-2);
    }


    fn main() -> int {
        return fib(7);
    }
    "#;


    test_simple_code("main", code, 13);
}

#[test]
fn test_cyclic_equals() {
    let code = r#"
    struct A(b: B?, i: int);
    struct B(a: A);

    fn create_a() -> A {
        let a = A(null, 5);
        let b = B(a);

        a.b = b;

        return a;
    }


    fn main() -> int {
        let a1 = create_a();
        let a2 = create_a();

        if a1 == a2 {
            return 1;
        } else {
            return 0;
        }
    }
    "#;


    test_simple_code("main", code, 1);

    let code = r#"
    struct A(b: B?, i: int);
    struct B(a: A);

    fn create_a() -> A {
        let a = A(null, 5);
        let b = B(a);

        a.b = b;

        return a;
    }


    fn main() -> int {
        let a1 = create_a();
        let a2 = create_a();
        a2.i = 6;

        if a1 == a2 {
            return 1;
        } else {
            return 0;
        }
    }
    "#;


    test_simple_code("main", code, 0);
}