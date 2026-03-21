use Tropaion::interpreter::value::Value::IntValue;
use Tropaion::run_code;


fn test_simple_code(main: &str, code: &str, expected: i32) {
    let (stack, heap) = run_code(code.to_string(), main).unwrap();

    assert_eq!(stack.len(), 2);
    assert_eq!(stack[1], IntValue(expected));
}

fn test_math_expr(expr: &str, expected: i32) {
    let text = r#"
    fn main() -> int {
        return "#
        .to_owned()
        + expr
        + r#";
    }
    "#;


    let (stack, heap) = run_code(text.to_string(), "main").unwrap();

    assert_eq!(stack.len(), 2);
    assert_eq!(
        stack[1],
        IntValue(expected),
        "Failed for expression '{} = {}'",
        expr,
        expected
    );
}

// ─── Math & Bitwise ──────────────────────────────────────────────────────────

#[test]
fn test_bitwise_ops() {
    // AND
    test_math_expr("6 & 3", 2);
    test_math_expr("0 & 255", 0);
    test_math_expr("255 & 255", 255);

    // OR
    test_math_expr("6 | 1", 7);
    test_math_expr("0 | 0", 0);

    // XOR
    test_math_expr("5 ^ 3", 6);
    test_math_expr("7 ^ 7", 0);

    // NOT (bitwise complement)
    test_math_expr("~0", -1);
    test_math_expr("~(-1)", 0);
    test_math_expr("~5", -6);

    // Shifts
    test_math_expr("1 << 3", 8);
    test_math_expr("16 >> 2", 4);
    test_math_expr("1 << 0", 1);
    test_math_expr("100 >> 1", 50);
}

#[test]
fn test_operator_precedence() {
    // Bitwise should be lower precedence than arithmetic
    test_math_expr("2 + 3 * 4", 14);
    test_math_expr("(2 + 3) * 4", 20);
    test_math_expr("10 - 2 * 3 + 1", 5);
    test_math_expr("8 / 2 + 6 / 3", 6);
    test_math_expr("3 + 4 * 2 - 1", 10);

    // Unary minus
    test_math_expr("-(-5)", 5);
    test_math_expr("-(3 + 2)", -5);
    test_math_expr("10 + -3", 7);
}

#[test]
fn test_int_overflow_wrap() {
    // i32::MAX + 1 should wrap to i32::MIN
    test_math_expr("2147483647 + 1", -2147483648_i32);
    // i32::MIN - 1 should wrap to i32::MAX
    test_math_expr("-2147483648 - 1", 2147483647);
}

#[test]
fn test_increment_decrement() {
    let code = r#"
    fn main() -> int {
        let x = 10;
        x++;
        x++;
        x--;
        x++;
        x--;
        x--;
        return x;
    }
    "#;
    test_simple_code("main", code, 10);
}

// ─── Variables & Scoping ─────────────────────────────────────────────────────

#[test]
fn test_deep_shadowing() {
    // Each nested scope shadows the outer `x` independently.
    let code = r#"
    fn main() -> int {
        let x = 1;

        if true {
            let x = 100;

            if true {
                let x = 999;
                // innermost `x` is 999, but we don't return it
            }

            x += 1; // modifies the middle `x` (100 → 101)
        }

        // outer `x` is still 1
        return x;
    }
    "#;
    test_simple_code("main", code, 1);
}

#[test]
fn test_scope_drop() {
    // Variables declared inside a scope must not survive outside it.
    // We verify this indirectly: the outer `result` is unaffected by inner
    // bindings of the same name.
    let code = r#"
    fn main() -> int {
        let result = 0;

        let i = 0;
        while i < 5 {
            let result = i * 10; // shadows outer result each iteration
            i++;
        }

        return result; // must still be 0
    }
    "#;
    test_simple_code("main", code, 0);
}

// ─── Control Flow ────────────────────────────────────────────────────────────

#[test]
fn test_nested_loops() {
    let code = r#"
    fn main() -> int {
        let total = 0;
        let i = 0;

        while i < 4 {
            let j = 0;

            while j < 4 {
                total += i * j;
                j++;
            }

            i++;
        }

        // sum of i*j for i,j in [0,4) = (0+1+2+3)^2 = 36
        return total;
    }
    "#;
    test_simple_code("main", code, 36);
}

#[test]
fn test_nested_if_in_loop() {
    let code = r#"
    fn main() -> int {
        let evens = 0;
        let odds  = 0;
        let i = 0;

        while i < 10 {
            if i % 2 == 0 {
                evens += i;
            } else {
                odds += i;
            }
            i++;
        }

        // evens: 0+2+4+6+8 = 20, odds: 1+3+5+7+9 = 25
        return evens - odds; // -5
    }
    "#;
    test_simple_code("main", code, -5);
}

#[test]
fn test_early_return_nested() {
    // Return should break out of multiple nested loops/ifs immediately.
    let code = r#"
    fn main() -> int {
        let i = 0;
        while i < 100 {
            let j = 0;
            while j < 100 {
                if i == 6 && j == 7 {
                    return i + j;
                }
                j++;
            }
            i++;
        }
        return -1;
    }
    "#;
    test_simple_code("main", code, 13);
}

#[test]
fn test_chained_else_if() {
    let classify = |n: i32| -> i32 {
        let code = format!(
            r#"
        fn main() -> int {{
            let n = {};
            if n < 0 {{
                return -1;
            }} else if n == 0 {{
                return 0;
            }} else if n < 10 {{
                return 1;
            }} else if n < 100 {{
                return 2;
            }} else {{
                return 3;
            }}
        }}
        "#,
            n
        );
        let (stack, _) = run_code(code, "main").unwrap();

        if let IntValue(v) = stack[1] { v } else { panic!("not int") }
    };

    assert_eq!(classify(-5), -1);
    assert_eq!(classify(0), 0);
    assert_eq!(classify(7), 1);
    assert_eq!(classify(42), 2);
    assert_eq!(classify(200), 3);
}

// ─── Functions & Overloading ──────────────────────────────────────────────────

#[test]
fn test_multiple_overloads_same_name() {
    let code = r#"
    fn double(x: int) -> int {
        return x * 2;
    }

    fn double(x: float) -> float {
        return x * 2.0;
    }

    fn double(x: bool) -> int {
        if x {
            return 2;
        }
        return 0;
    }

    fn main() -> int {
        return double(5) + double(true) + double(false);
    }
    "#;
    test_simple_code("main", code, 12);
}

#[test]
fn test_mutual_recursion() {
    // is_even / is_odd via mutual recursion
    let code = r#"
    fn is_even(n: int) -> bool {
        if n == 0 {
            return true;
        }
        return is_odd(n - 1);
    }

    fn is_odd(n: int) -> bool {
        if n == 0 {
            return false;
        }
        return is_even(n - 1);
    }

    fn main() -> int {
        let result = 0;

        if is_even(10) { result += 1; }
        if is_odd(7)   { result += 2; }
        if !is_even(3) { result += 4; }
        if !is_odd(4)  { result += 8; }

        return result; // all four branches taken → 15
    }
    "#;
    test_simple_code("main", code, 15);
}

#[test]
fn test_deep_recursion_sum() {
    // Triangular number T(100) = 5050
    let code = r#"
    fn tri(n: int) -> int {
        if n <= 0 {
            return 0;
        }
        return n + tri(n - 1);
    }

    fn main() -> int {
        return tri(100);
    }
    "#;
    test_simple_code("main", code, 5050);
}

#[test]
fn test_pass_struct_to_fn() {
    let code = r#"
    struct Vec2(x: int, y: int);

    fn dot(a: Vec2, b: Vec2) -> int {
        return a.x * b.x + a.y * b.y;
    }

    fn scale(v: Vec2, s: int) -> int {
        return v.x * s + v.y * s;
    }

    fn main() -> int {
        let a = Vec2(3, 4);
        let b = Vec2(1, 2);

        return dot(a, b) + scale(a, 2);
    }
    "#;
    // dot(a,b) = 3*1 + 4*2 = 11; scale(a,2) = (3+4)*2 = 14  → 25
    test_simple_code("main", code, 25);
}

// ─── Structs ─────────────────────────────────────────────────────────────────

#[test]
fn test_struct_field_mutation() {
    let code = r#"
    struct Counter(value: int) {
        fn increment() {
            this.value += 1;
        }

        fn add(n: int) {
            this.value += n;
        }

        fn get() -> int {
            return this.value;
        }
    }

    fn main() -> int {
        let c = Counter(0);

        c.increment();
        c.increment();
        c.add(8);
        c.increment();

        return c.get(); // 11
    }
    "#;
    test_simple_code("main", code, 11);
}

#[test]
fn test_struct_method_chain_logic() {
    let code = r#"
    struct Accumulator(total: int, count: int) {
        fn add(n: int) {
            this.total += n;
            this.count += 1;
        }

        fn average() -> int {
            if this.count == 0 {
                return 0;
            }
            return this.total / this.count;
        }

        fn weighted(factor: int) -> int {
            return average() * factor;
        }
    }

    fn main() -> int {
        let acc = Accumulator(0, 0);

        acc.add(10);
        acc.add(20);
        acc.add(30);
        // total=60, count=3, average=20

        return acc.weighted(3); // 60
    }
    "#;
    test_simple_code("main", code, 60);
}

#[test]
fn test_nested_struct_access() {
    let code = r#"
    struct Pos(x: int, y: int);
    struct Entity(pos: Pos, hp: int);

    fn main() -> int {
        let p = Pos(3, 7);
        let e = Entity(p, 100);

        e.pos.x += 2;
        e.hp    -= e.pos.y;

        return e.pos.x + e.hp; // 5 + 93 = 98
    }
    "#;
    test_simple_code("main", code, 98);
}

#[test]
fn test_struct_returned_from_fn() {
    let code = r#"
    struct Range(lo: int, hi: int) {
        fn size() -> int {
            return this.hi - this.lo;
        }

        fn contains(n: int) -> bool {
            return n >= this.lo && n < this.hi;
        }
    }

    fn make_range(lo: int, hi: int) -> Range {
        return Range(lo, hi);
    }

    fn main() -> int {
        let r = make_range(5, 15);
        let result = r.size(); // 10

        if r.contains(7)  { result += 1; }
        if r.contains(5)  { result += 2; }
        if r.contains(14) { result += 4; }
        if r.contains(15) { result -= 100; } // must NOT trigger

        return result; // 17
    }
    "#;
    test_simple_code("main", code, 17);
}

// ─── Nullables ───────────────────────────────────────────────────────────────

#[test]
fn test_nullable_chain() {
    let code = r#"
    struct Node(val: int, next: Node?);

    fn sum_list(n: Node?) -> int {
        if n == null {
            return 0;
        }
        return n!!.val + sum_list(n!!.next);
    }

    fn main() -> int {
        let c = Node(3, null);
        let b = Node(2, c);
        let a = Node(1, b);

        return sum_list(a);
    }
    "#;
    test_simple_code("main", code, 6);
}

#[test]
fn test_nullable_default_chain() {
    // `??` is right-associative: a ?? b ?? c = a ?? (b ?? c)
    let code = r#"
    fn maybe(n: int, threshold: int) -> int? {
        if n < threshold {
            return null;
        }
        return n;
    }

    fn main() -> int {
        let a = maybe(1, 5);  // null
        let b = maybe(3, 5);  // null
        let c = maybe(7, 5)!!;  // 7

        let x = a ?? b ?? c;  // should be 7
        let y = a ?? 42;      // should be 42

        return x + y; // 49
    }
    "#;
    test_simple_code("main", code, 49);
}

#[test]
fn test_safe_call_chain() {
    let code = r#"
    struct Inner(value: int) {
        fn doubled() -> int {
            return value * 2;
        }
    }

    struct Outer(inner: Inner?) {
        fn get_inner() -> Inner? {
            return this.inner;
        }
    }

    fn main() -> int {
        let result = 0;

        let o1 = Outer(Inner(5));
        let o2 = Outer(null);

        // safe call on non-null inner → 10
        let v1: int? = o1.inner?.doubled();
        if v1 != null {
            result += v1!!;
        }

        // safe call on null inner → null (should not add anything)
        let v2: int? = o2.inner?.doubled();
        if v2 != null {
            result += 999; // must NOT fire
        }

        return result; // 10
    }
    "#;
    test_simple_code("main", code, 10);
}

#[test]
fn test_nullable_reassign_and_deref() {
    let code = r#"
    fn main() -> int {
        let x: int? = null;
        let total = 0;

        let i = 1;
        while i <= 5 {
            x = i * i;         // x is reassigned each iteration
            total += x!!;
            i++;
        }

        // 1 + 4 + 9 + 16 + 25 = 55
        return total;
    }
    "#;
    test_simple_code("main", code, 55);
}

// ─── Arrays ──────────────────────────────────────────────────────────────────

#[test]
fn test_array_read_write() {
    let code = r#"
    fn main() -> int {
        let arr = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        // double every even-indexed element
        let i = 0;
        while i < 10 {
            if i % 2 == 0 {
                arr[i] = arr[i] * 2;
            }
            i++;
        }

        // sum everything: 2+2+6+4+10+6+14+8+18+10 = 80
        let sum = 0;
        let j = 0;
        while j < 10 {
            sum += arr[j];
            j++;
        }

        return sum;
    }
    "#;
    test_simple_code("main", code, 80);
}

#[test]
fn test_array_passed_to_fn() {
    let code = r#"
    fn array_sum(arr: [int], len: int) -> int {
        let total = 0;
        let i = 0;
        while i < len {
            total += arr[i];
            i++;
        }
        return total;
    }

    fn array_max(arr: [int], len: int) -> int {
        let m = arr[0];
        let i = 1;
        while i < len {
            if arr[i] > m {
                m = arr[i];
            }
            i++;
        }
        return m;
    }

    fn main() -> int {
        let data = [4, 7, 2, 9, 1, 5, 8, 3, 6];

        let s = array_sum(data, 9);  // 45
        let m = array_max(data, 9);  // 9

        return s + m; // 54
    }
    "#;
    test_simple_code("main", code, 54);
}

#[test]
fn test_array_of_structs() {
    let code = r#"
    struct Point(x: int, y: int);

    fn manhattan(p: Point) -> int {
        return p.x + p.y;
    }

    fn main() -> int {
        let pts = [Point(1, 2), Point(3, 4), Point(5, 6)];

        let total = 0;
        let i = 0;
        while i < 3 {
            total += manhattan(pts[i]);
            i++;
        }

        // (1+2) + (3+4) + (5+6) = 3+7+11 = 21
        return total;
    }
    "#;
    test_simple_code("main", code, 21);
}

#[test]
fn test_array_index_expression() {
    // Index can be any int expression.
    let code = r#"
    fn main() -> int {
        let arr = [10, 20, 30, 40, 50];
        let base = 1;

        return arr[base + 1] + arr[base * 2]; // arr[2] + arr[2] = 60
    }
    "#;
    test_simple_code("main", code, 60);
}

// ─── Boolean Logic & Lazy Evaluation ─────────────────────────────────────────

#[test]
fn test_short_circuit_or() {
    // If left side of || is true the right should not be evaluated.
    let code = r#"
    struct Flag(hit: bool);

    fn side_effect(f: Flag) -> bool {
        f.hit = true;
        return true;
    }

    fn main() -> int {
        let f = Flag(false);

        // true || side_effect(f)  — right side must NOT run
        if true || side_effect(f) {
            if f.hit {
                return 1; // bad: side effect ran
            }
            return 0; // good
        }

        return 2;
    }
    "#;
    test_simple_code("main", code, 0);
}

#[test]
fn test_short_circuit_and() {
    // If left side of && is false the right should not be evaluated.
    let code = r#"
    struct Flag(hit: bool);

    fn side_effect(f: Flag) -> bool {
        f.hit = true;
        return false;
    }

    fn main() -> int {
        let f = Flag(false);

        if false && side_effect(f) {
            return 2;
        }

        if f.hit {
            return 1; // bad: side effect ran
        }

        return 0; // good
    }
    "#;
    test_simple_code("main", code, 0);
}

#[test]
fn test_complex_bool_expr() {
    let code = r#"
    fn main() -> int {
        let a = true;
        let b = false;
        let c = true;

        let result = 0;

        if a && (b || c)        { result += 1; }
        if !b && !(!a)          { result += 2; }
        if (a || b) && (b || c) { result += 4; }
        if !(a && b)            { result += 8; }
        if (a && c) || b        { result += 16; }

        return result; // all five → 31
    }
    "#;
    test_simple_code("main", code, 31);
}

// ─── Constants ───────────────────────────────────────────────────────────────

#[test]
fn test_const_access() {
    let code = r#"
    const MULTIPLIER: int = 7;
    const BASE: int = 10;

    fn scaled(n: int) -> int {
        return n * MULTIPLIER + BASE;
    }

    fn main() -> int {
        return scaled(5) + scaled(0);
        // scaled(5) = 45, scaled(0) = 10 → 55
    }
    "#;
    test_simple_code("main", code, 55);
}

#[test]
fn test_const_array() {
    let code = r#"
    const PRIMES: [int] = [2, 3, 5, 7, 11];

    fn main() -> int {
        let sum = 0;
        let i = 0;
        while i < 5 {
            sum += PRIMES[i];
            i++;
        }
        return sum; // 28
    }
    "#;
    test_simple_code("main", code, 28);
}

// ─── Autoboxing / Type Promotion ─────────────────────────────────────────────

#[test]
fn test_autobox_struct() {
    let code = r#"
    struct Token(id: int);

    fn process(t: Token?) -> int {
        if t == null {
            return -1;
        }
        return t!!.id * 2;
    }

    fn main() -> int {
        let t1 = Token(5);
        let t2: Token? = Token(3);

        // t1 (non-nullable) should be auto-promoted when passed to process()
        return process(t1) + process(t2) + process(null);
        // 10 + 6 + (-1) = 15
    }
    "#;
    test_simple_code("main", code, 15);
}

// ─── Integration / Larger Programs ───────────────────────────────────────────

/// A small stack implemented with an array + a length counter.
#[test]
fn test_manual_stack() {
    let code = r#"
    struct Stack(data: [int], size: int) {
        fn push(val: int) {
            data[size] = val;
            this.size += 1;
        }

        fn pop() -> int {
            this.size -= 1;
            return data[size];
        }

        fn peek() -> int {
            return data[size - 1];
        }

        fn is_empty() -> bool {
            return this.size == 0;
        }
    }

    fn main() -> int {
        // pre-allocate enough space
        let buf = [0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let s = Stack(buf, 0);

        s.push(1);
        s.push(2);
        s.push(3);

        let total = 0;
        total += s.pop();  // 3
        total += s.peek(); // 2  (peek doesn't remove)
        total += s.pop();  // 2
        total += s.pop();  // 1

        if s.is_empty() {
            total += 10;
        }

        return total; // 3+2+2+1+10 = 18
    }
    "#;
    test_simple_code("main", code, 18);
}

/// Sieve of Eratosthenes up to 30 — count the primes.
#[test]
fn test_sieve_count() {
    let code = r#"
    fn main() -> int {
        // sieve[i] == 0 means i is prime candidate
        let sieve = [
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        ];

        // mark 0 and 1 as not-prime
        sieve[0] = 1;
        sieve[1] = 1;

        let i = 2;
        while i < 30 {
            if sieve[i] == 0 {
                let j = i * 2;
                while j < 30 {
                    sieve[j] = 1;
                    j += i;
                }
            }
            i++;
        }

        let count = 0;
        let k = 0;
        while k < 30 {
            if sieve[k] == 0 {
                count++;
            }
            k++;
        }

        // primes < 30: 2,3,5,7,11,13,17,19,23,29 → 10
        return count;
    }
    "#;
    test_simple_code("main", code, 10);
}

/// Binary search on a sorted array.
#[test]
fn test_binary_search() {
    let code = r#"
    fn bsearch(arr: [int], len: int, target: int) -> int {
        let lo = 0;
        let hi = len - 1;

        while lo <= hi {
            let mid = (lo + hi) / 2;

            if arr[mid] == target {
                return mid;
            } else if arr[mid] < target {
                lo = mid + 1;
            } else {
                hi = mid - 1;
            }
        }

        return -1;
    }

    fn main() -> int {
        let sorted = [1, 3, 5, 7, 9, 11, 13, 15, 17, 19];

        let result = 0;

        result += bsearch(sorted, 10, 1);   // idx 0
        result += bsearch(sorted, 10, 11);  // idx 5
        result += bsearch(sorted, 10, 19);  // idx 9
        result += bsearch(sorted, 10, 6);   // -1  (not found)

        return result; // 0+5+9-1 = 13
    }
    "#;
    test_simple_code("main", code, 13);
}

/// Linked-list node count using nullable recursive structs.
#[test]
fn test_linked_list_length() {
    let code = r#"
    struct ListNode(val: int, next: ListNode?);

    fn length(node: ListNode?) -> int {
        if node == null {
            return 0;
        }
        return 1 + length(node!!.next);
    }

    fn nth(node: ListNode?, n: int) -> int {
        if n == 0 {
            return node!!.val;
        }
        return nth(node!!.next, n - 1);
    }

    fn main() -> int {
        let n4 = ListNode(40, null);
        let n3 = ListNode(30, n4);
        let n2 = ListNode(20, n3);
        let n1 = ListNode(10, n2);

        let len = length(n1); // 4

        // nth(list, 2) = 30
        let third = nth(n1, 2);

        return len + third; // 34
    }
    "#;
    test_simple_code("main", code, 34);
}

/// State-machine-style struct: a simple counter that caps at a max value.
#[test]
fn test_capped_counter() {
    let code = r#"
    struct CappedCounter(value: int, max: int) {
        fn tick() {
            if this.value < this.max {
                this.value += 1;
            }
        }

        fn reset() {
            this.value = 0;
        }

        fn is_full() -> bool {
            return this.value == this.max;
        }
    }

    fn main() -> int {
        let c = CappedCounter(0, 5);

        let i = 0;
        while i < 10 {
            c.tick(); // only first 5 ticks count
            i++;
        }

        let result = c.value; // 5

        if c.is_full() {
            result += 10;
        }

        c.reset();

        if c.value == 0 {
            result += 1;
        }

        return result; // 16
    }
    "#;
    test_simple_code("main", code, 16);
}