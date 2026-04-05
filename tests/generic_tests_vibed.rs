use tropaion::interpreter::value::Value::IntValue;
use tropaion::run_code;


fn test_simple_code(main: &str, code: &str, expected: i32) {
    let mut blob = run_code(code.to_string(), main).unwrap();

    let value = blob.next_int().unwrap();
    blob.expect_end().unwrap();
    assert_eq!(value, expected);
}
// --- Structs ---

#[test]
fn test_generics_multiple_type_params() {
    // Generic struct with two independent type params
    let code = r#"
    struct Pair<A, B>(first: A, second: B);

    fn main() -> int {
        let p = Pair(10, true);

        if !p.second {
            return 0;
        }

        return p.first;
    }
    "#;

    test_simple_code("main", code, 10);
}

#[test]
fn test_generics_nested() {
    // Box<Box<int>> — generic type used as a type argument
    let code = r#"
    struct Box<T>(value: T);

    fn main() -> int {
        let inner = Box(7);
        let outer = Box(inner);

        return outer.value.value;
    }
    "#;

    test_simple_code("main", code, 7);
}

#[test]
fn test_generics_nullable_field() {
    // T field is itself nullable
    let code = r#"
    struct Box<T>(value: T?);

    fn main() -> int {
        let empty: Box<int> = Box(null);
        let full = Box(42);

        let a = empty.value ?? 0;
        let b = full.value ?? 0;

        return a + b;
    }
    "#;

    test_simple_code("main", code, 42);
}

#[test]
fn test_generics_struct_as_type_arg() {
    // T instantiated with a user-defined struct
    let code = r#"
    struct Point(x: int, y: int);
    struct Box<T>(value: T);

    fn main() -> int {
        let b = Box(Point(3, 4));

        return b.value.x + b.value.y;
    }
    "#;

    test_simple_code("main", code, 7);
}

#[test]
fn test_generics_struct_field_mutation() {
    // Mutating a field of type T inside a generic struct
    let code = r#"
    struct Box<T>(value: T);

    fn main() -> int {
        let b = Box(10);
        b.value = 99;

        return b.value;
    }
    "#;

    test_simple_code("main", code, 99);
}

#[test]
fn test_generics_equality() {
    // Two generic structs with equal and unequal contents
    let code = r#"
    struct Box<T>(value: T);

    fn main() -> int {
        let a = Box(5);
        let b = Box(5);
        let c = Box(6);

        if a != b {
            return 1;
        }

        if a == c {
            return 2;
        }

        return 0;
    }
    "#;

    test_simple_code("main", code, 0);
}

#[test]
fn test_generics_method_accepts_t_arg() {
    // Method takes an argument of type T
    let code = r#"
    struct Box<T>(value: T) {
        pub fn replace(new_val: T) -> T {
            let old = value;
            value = new_val;
            return old;
        }
    }

    fn main() -> int {
        let b = Box(10);
        let old = b.replace(99);

        return old + b.value;
    }
    "#;

    test_simple_code("main", code, 109);
}

// --- Functions ---

#[test]
fn test_generics_fn_multiple_type_params() {
    // Generic function with two independent type parameters
    let code = r#"
    fn first<A, B>(a: A, b: B) -> A {
        return a;
    }

    fn main() -> int {
        return first(42, true);
    }
    "#;

    test_simple_code("main", code, 42);
}

#[test]
fn test_generics_fn_returns_struct() {
    // Generic function wraps a value and returns a generic struct
    let code = r#"
    struct Box<T>(value: T);

    fn wrap<T>(value: T) -> Box<T> {
        return Box(value);
    }

    fn main() -> int {
        let b = wrap(55);

        return b.value;
    }
    "#;

    test_simple_code("main", code, 55);
}

#[test]
fn test_generics_fn_with_struct_arg() {
    // Generic function accepting a user-defined struct as T
    let code = r#"
    struct Point(x: int, y: int);

    fn identity<T>(v: T) -> T {
        return v;
    }

    fn main() -> int {
        let p = identity(Point(6, 7));

        return p.x + p.y;
    }
    "#;

    test_simple_code("main", code, 13);
}

#[test]
fn test_generics_fn_overload_coexistence() {
    // Generic function and a concrete overload with the same name can coexist
    let code = r#"
    fn describe<T>(v: T) -> int {
        return 0;
    }

    fn describe(v: int) -> int {
        return v * 2;
    }

    fn main() -> int {
        let a = describe(5);    // should hit the concrete int overload
        let b = describe(true); // should hit the generic overload

        return a + b;
    }
    "#;

    test_simple_code("main", code, 10);
}

// --- Combination ---

#[test]
fn test_generics_passed_to_function() {
    // A generic struct instance is passed as an argument to a function
    let code = r#"
    struct Box<T>(value: T);

    fn unbox<T>(b: Box<T>) -> T {
        return b.value;
    }

    fn main() -> int {
        let b = Box(33);

        return unbox(b);
    }
    "#;

    test_simple_code("main", code, 33);
}

#[test]
fn test_generics_recursive_fn() {
    // Generic function used recursively (e.g. identity applied n times)
    let code = r#"
    fn countdown<T>(n: int, v: T) -> T {
        if n == 0 {
            return v;
        }

        return countdown(n - 1, v);
    }

    fn main() -> int {
        return countdown(5, 99);
    }
    "#;

    test_simple_code("main", code, 99);
}

#[test]
fn test_generics_struct_as_field() {
    // Generic struct used as a field in another (non-generic) struct
    let code = r#"
    struct Box<T>(value: T);

    struct Wrapper(inner: Box<int>, label: int);

    fn main() -> int {
        let w = Wrapper(Box(21), 2);

        return w.inner.value * w.label;
    }
    "#;

    test_simple_code("main", code, 42);
}

#[test]
fn test_generics_nullable_instance() {
    // A generic struct instance is held as nullable
    let code = r#"
    struct Box<T>(value: T);

    fn main() -> int {
        let b: Box<int>? = null;

        let v = b?.value ?? 7;

        b = Box(3);

        return v + b!!.value;
    }
    "#;

    test_simple_code("main", code, 10);
}