use Tropaion::interpreter::value::Value::IntValue;
use Tropaion::run_code;

fn test_simple_code(main: &str, code: &str, expected: i32) {
    let (stack, heap) = run_code(code.to_string(), main).unwrap();

    assert_eq!(stack.len(), 2, "{:?}", &stack[0..stack.len()]); // nullptr, value
    assert_eq!(stack[1], IntValue(expected));
}

fn test_math_expr(expr: &str, expected: i32) {
    let text = r#"
    fn main() -> int {
        return "#.to_owned() +expr+r#";
    }
    "#;
    let (stack, heap) = run_code(text.to_string(), "main").unwrap();
    
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


    test_math_expr("10 - 3 - 2", 5);
}

#[test]
fn test_flattening() {
    let code = r#"
    struct Point(x: int, y: int);

    fn main() -> int {
        return 5 + Point(2, 4).x + 5;
    }
    "#;

    test_simple_code("main", code, 12);
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
pub fn test_else() {
    let create = |str: &str| {
        return r#"
        fn main() -> int {
            let x = -1;

        "#.to_owned() + str + r#"

            return x;
        }
        "#;
    };

    let case = r#"
    if false {
        x = 5;
    }
    "#;
    test_simple_code("main", create(case).as_ref(), -1);

    let case = r#"
    if false {
        x = 5;
    } else {
        x = 10;
    }
    "#;
    test_simple_code("main", create(case).as_ref(), 10);

    let case = r#"
    if false {
        x = 5;
    } else if true {
        x = 10;
    }
    "#;
    test_simple_code("main", create(case).as_ref(), 10);

    let case = r#"
    if false {
        x = 5;
    } else if false {
        x = 10;
    } else {
        x = 20;
    }
    "#;
    test_simple_code("main", create(case).as_ref(), 20);

    let case = r#"
    if false {
        x = 5;
    } else if true {
        x = 10;
    } else {
        x = 20;
    }
    "#;
    test_simple_code("main", create(case).as_ref(), 10);
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
fn test_funcs() {
    let code = r#"
    fn two_arg(a: int, b: float) -> int {
        return a;
    }

    fn main() -> int {
        return two_arg(5, 2.7);
    }
    "#;

    test_simple_code("main", code, 5);
}

#[test]
pub fn test_overloads() {
    let loop_stmt = r#"
    struct Eh();

    fn other(i: Eh) -> Eh {
        return i;
    }

    fn other(i: float) -> float {
        return i;
    }

    fn other(i: int) -> int{
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
        let x: int = x!!;

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

        return 5 + a.i + 5;
    }
    "#;


    test_simple_code("main", code, 15);


    let code = r#"
    struct Point(x: int, y: int);

    fn create_point(x: int, y: int) -> Point? {
        let p: Point? = Point(x, y);

        return p;
    }

    fn main() -> int {
        let p = create_point(10, 20);
        let p = p!!;

        return p.x + p.y;
    }
    "#;

    test_simple_code("main", code, 30);
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
}

#[test]
fn test_cyclic_equals2() {
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

#[test]
fn test_bool_ops() {
    let code = r#"
    struct Holder(x: int);

    fn a(h: Holder) -> bool {
        h.x += 1;

        return false;
    }

    fn b(h: Holder) -> bool {
        h.x += 7;

        return true;
    }

    fn c(h: Holder) -> bool {
        h.x += 23;

        return true;
    }


    fn main() -> int {
        let h = Holder(0);

        let y = 0;
        if a(h) {
           y += 3;
        }
        // x: 1; y: 0

        if b(h) || a(h) {
            y += 5;
        }
        // x: 8; y: 5

        if a(h) && b(h) {
            y += 10;
        }
        // x: 9; y: 5

        if b(h) && (a(h) || c(h)) {
            y += 20;
        }
        // x: 40; y: 25

        return h.x + y;
    }
    "#;


    test_simple_code("main", code, 65);
}


#[test]
fn test_shadowing() {
    let code = r#"
    fn main() -> int {
        let x = 10;

        if true {
            let x = 50;
            x += 1;
        }

        return x;
    }
    "#;

    test_simple_code("main", code, 10);
}

#[test]
fn test_return() {
    let code = r#"
    fn main() -> int {
        let i = 0;
        while i < 100 {
            if i == 5 {
                return i;
            }
            i++;
        }
        return -1;
    }
    "#;

    test_simple_code("main", code, 5);
}

#[test]
fn test_null_comparison() {
    let code = r#"
    struct Point(x: int, y: int);

    fn main() -> int {
        let p: Point? = null;
        let p2: Point? = null;

        if p != p2 {
            return 1;
        }

        let p3 = Point(1, 2);

        if p == p3 {
            return 2;
        }
        if p3 == p {
            return 3;
        }

        let p4: Point? = Point(1, 2);

        if p3 != p4 {
            return 4;
        }

        if p3.x != p4?.x {
            return 5;
        }

        return 0;
    }
    "#;

    test_simple_code("main", code, 0);
}

#[test]
fn test_method_call() {
    let code = r#"
    struct T(a: int) {
        fn get() -> int {
            return a * 2;
        }

        fn sum() -> int {
            return get() + this.get();
        }

    }

    fn main() -> int {
        let t = T(5);

        return t.sum();
    }
    "#;


    test_simple_code("main", code, 20);
}


#[test]
fn test_method_call2() {
    let code = r#"
    struct Rect(a: int, b: int) {

        fn sum() -> int {
            return a + b;
        }

        fn cir() -> int {
            return sum() * 2;
        }

        fn value(n: int) -> int {
            return cir() * n;
        }

    }

    fn main() -> int {
        let t = Rect(5, 10);

        return t.value(3);
    }
    "#;


    test_simple_code("main", code, 90);
}

#[test]
fn test_arrays() {
    let code = r#"
    fn generate() -> [int] {
        let arr = [10, 2, 3, 4, 5];

        arr[0] = 1;

        return arr;
    }

    fn main() -> int {
        let arr = generate();

        if arr[0] != 1 {
            return 1;
        }

        if arr[4] != 5 {
            return 2;
        }

        arr[2] = 0;

        return arr[2];
    }
    "#;


    test_simple_code("main", code, 0);
}

#[test]
fn test_not() {
    let code = r#"
    fn main() -> int {
        if !false {
            return 0;
        }

        return 1;
    }
    "#;

    test_simple_code("main", code, 0);
}

#[test]
fn test_null_deref() {
    let code = r#"
    fn main() -> int {
        let x: int? = null;
        x = 5;

        let y = x!! + 3;

        return y;
    }
    "#;

    test_simple_code("main", code, 8);
}

#[test]
fn test_autoboxing() {
    let code = r#"
    fn calc(i: int?) -> int {
        if i == null {
            return 0;
        }

        return i!!;
    }

    fn main() -> int {
        let x: int = 2;
        let y: int? = 3;

        return calc(5) + calc(null) + calc(x) + calc(y);
    }
    "#;


    test_simple_code("main", code, 10);
}

#[test]
fn test_safe_calls() {
    let code = r#"
    struct Rect(a: int, b: int) {
        fn area() -> int {
            return a * b;
        }
    }

    fn main() -> int {
        let r: Rect? = Rect(4, 5);

        if r?.area() == 20 {
            r = null;

            if r?.area() == 20 || r?.area() != null {
                return 2;
            }

            return 0;
        }

        return 1;
    }
    "#;



    test_simple_code("main", code, 0);
}

#[test]
fn test_type_wrapping() {
    let code = r#"
    fn main() -> int {
        let i: int = 5;
        let x: int? = null;

        x = i;

        return 0;
    }
    "#;

    test_simple_code("main", code, 0);
}

#[test]
fn test_scopes() {
    let code = r#"
    struct Box(field: bool);

    fn main() -> int {
        if true {
            let bad_item = Box(true);
        }

        let sword = Box(false);

        let y = sword.field;

        return 0;
    }
    "#;

    test_simple_code("main", code, 0);
}

#[test]
fn test_a_lot() {
    let code = r#"
    struct Item(power: int, is_cursed: bool);

    struct Player(hp: int, score: int, current_item: Item?) {

        fn heal(amount: int) -> int {
            this.hp += amount;
            return this.hp;
        }

        fn equip(i: Item?) -> bool {
            if i == null {
                this.current_item = null;
                return false;
            }

            if i!!.is_cursed {
                this.hp -= 10;
                return false;
            }

            this.current_item = i;
            return true;
        }

        fn attack(base_dmg: int) -> int {
            let bonus = 0;

            if this.current_item != null {
                bonus = this.current_item!!.power;
            }

            this.score += base_dmg + bonus;
            return base_dmg + bonus;
        }
    }

    fn calculate_bonus(n: int) -> int {
        if n <= 1 {
            return 1;
        }
        return n + calculate_bonus(n - 1);
    }

    fn main() -> int {
        let p: Player = Player(100, 0, null);

        let events = [10, 20, 0, 50, 5];

        let i = 0;
        while i < 5 {
            let dmg = events[i];

            if dmg == 0 {
                p.heal(15);

                let i = 100;
                p.score += i;
            }
            else if dmg > 40 {
                let bad_item = Item(50, true);
                p.equip(bad_item);
            }
            else {
                let sword = Item(5, false);
                p.equip(sword);
                p.attack(dmg);
            }

            i++;
        }

        let final_bonus = calculate_bonus(5);
        p.score += final_bonus;

        return p.hp + p.score;
    }
    "#;


    test_simple_code("main", code, 270);
}

#[test]
fn test_null_deconstruct() {
    let code = r#"
    fn main() -> int {
        let x: int? = 5;

        let x = x ?? 0;

        return x * 2;
    }
    "#;

    test_simple_code("main", code, 10);


    let code = r#"
    fn main() -> int {
        let a: int? = null;
        let b: int? = null;
        let c: int = 10;

        // should be same as a ?? (b ?? c)
        let x = a ?? b ?? c;

        return x * 2;
    }
    "#;

    test_simple_code("main", code, 20);
}


#[test]
fn test_generics() {
    let code = r#"
    struct Vec2<T>(a: T, b: T);

    fn main() -> int {
        let point = Vec2(5, 10);

        return point.a + point.b;
    }
    "#;

    test_simple_code("main", code, 15);
}

#[test]
fn test_generics2() {
    let code = r#"
    fn box<T>(value: T) -> T {
        return value;
    }

    fn main() -> int {
        let a = box(10.27);
        let a = box(5);

        return a;
    }
    "#;

    test_simple_code("main", code, 5);
}

#[test]
fn test_generics_shadowing() {
    let code = r#"
    struct Scope<T>() {
        fn box(value: T) -> T {
            return value;
        }
    }

    fn box<T>(value: T) -> int {
        return 1;
    }

    fn main() -> int {
        let a = box(100);
        let b = Scope().box(5);

        return a+b;
    }
    "#;

    test_simple_code("main", code, 6);
}

#[test]
fn test_generics3() {
    let code = r#"
    struct Vec2<T>(a: T, b: T) {

        fn get_a() -> T {
            return a;
        }

    }

    fn main() -> int {
        let point = Vec2(5, 10);

        return point.get_a() + point.b;
    }
    "#;

    test_simple_code("main", code, 15);
}

#[test]
fn test_generics4() {
    let code = r#"
    struct Box<T>(value: T);

    fn box<T>(value: T) -> Box<T> {
        return Box(value);
    }

    fn main() -> int {
        let b: Box<int> = box(27);

        return b.value;
    }
    "#;

    test_simple_code("main", code, 27);
}

#[test]
fn test_generic_struct_shadow(){
    let code = r#"
    struct Box<T>(value: T);

    fn unbox(b: Box<int>) -> int {
        return b.value * 2;
    }

    fn unbox(b: Box<float>) -> int {
        return b.value/2.0;
    }

    fn main() -> int {
        let a = Box(10);
        let b = Box(200.0);

        let x = unbox(a);
        let y = unbox(b);

        return x;
    }
    "#;

    test_simple_code("main", code, 20);
}

#[test]
fn test_loop_interrupt() {
    let code = r#"
    fn main() -> int {
        let i = 0;
        let x = 0;

        while i < 10 {
            if i < 3 {
                i += 2;
                x++;
                continue;
            }

            i++;
        }

        return (i-9)*x*x;
    }
    "#;

    test_simple_code("main", code, 4);
}

#[test]
fn test_weird_order_bug() {
    let code = r#"
    fn main() -> int {
        return Box(109).get_value(); // 1-  the generic_helper is now checked for generics
    }

    struct Box<T>(value: T) {
        fn get_value() -> T{ // 2- only now the generic gets registered
            return value;
        }
    }
    "#;

    test_simple_code("main", code, 109);
}

#[test]
fn test_vec() {
    let code = r#"
    fn main() -> int {
        let v = Vec(2, 0, __heap_alloc(2));

        v.push(77);
        v.push(20);
        v.push(5);
        v.push(6);
        v.push(7);
        v.push(8);
        v.pop();
        v.push(50);
        v.pop();
        v.pop();
        return v.pop();
    }
    "#;

    test_simple_code("main", code, 6);
}


#[test]
fn test_tuples() {
    let code = r#"
    fn main() -> int {
        let t = (5, 17.23, 300, 400, 500);

        let x = t.2;

        return x;
    }
    "#;

    test_simple_code("main", code, 300);
}

#[test]
fn test_tuples_passing() {
    let code = r#"
    fn get_second(t: (float, int, float)) -> int {
        return t.1;
    }

    fn main() -> int {
        let t = (1.0, 2, 3.0);

        let x = get_second(t);

        return x;
    }
    "#;

    test_simple_code("main", code, 2);
}