//! Regression tests for generic *type resolution* — the phase that expands
//! `SymbolType{Name, [args]}` into a concrete (monomorphized) struct.
//!
//! Bugs these guard against:
//!
//!   1. Self-referential generic structs (`Node<T>` whose field is `Node<T>?`)
//!      sending the resolver into infinite recursion / stack overflow.
//!
//!   2. The *second* occurrence of an already-resolved generic instantiation —
//!      especially a **bare** (un-nullable) struct field — staying stuck as an
//!      unresolved `SymbolType`, surfacing later as a member access on a
//!      `$symbol-Name$` type. The instantiation cache returned the cached entry
//!      without resolving the new field's own entry, and the deferred field
//!      pass resolves entries *in place* (it ignores the returned entry).
//!
//!   3. A *nested* generic field (`Inner<T>` inside `Outer<T>`, or the recursive
//!      `Node<T>`) staying abstract when the concrete type is produced through a
//!      constructor call, because `box_arg` only fills the direct generic-param
//!      slots. Member access then saw an abstract `T` and tripped an
//!      `IllegalBinaryExpression`.
//!
//! A resolution failure shows up as a `COMPILETIME([...])` error out of
//! `run_code(...).unwrap()`.

use tropaion::run_code;

fn run_int(code: &str, expected: i32) {
    let mut blob = run_code(code.to_string(), "main")
        .unwrap_or_else(|e| panic!("compilation/runtime failed: {e:?}"));
    let value = blob.next_int().unwrap();
    blob.expect_end().unwrap();
    assert_eq!(value, expected);
}

// ── Repeated generic instantiations (the `$symbol-Vec$` class) ───────────────

#[test]
fn same_generic_field_in_two_structs() {
    // `Box<int>` appears as a bare field in two different structs. The second
    // one used to stay an unresolved SymbolType, so `b.b.value` blew up with a
    // member access on `$symbol-Box$`.
    let code = r#"
    struct Box<T>(value: T);

    struct A(b: Box<int>);
    struct B(b: Box<int>);

    fn main() -> int {
        let a = A(Box(1));
        let b = B(Box(2));

        return a.b.value + b.b.value;
    }
    "#;
    run_int(code, 3);
}

#[test]
fn same_generic_field_twice_in_one_struct() {
    // Two bare fields of the same instantiation inside a single struct — the
    // second field is the one that hits the instantiation cache.
    let code = r#"
    struct Box<T>(value: T);

    struct Pair(first: Box<int>, second: Box<int>);

    fn main() -> int {
        let p = Pair(Box(10), Box(32));

        return p.first.value + p.second.value;
    }
    "#;
    run_int(code, 42);
}

#[test]
fn repeated_generic_field_with_struct_type_arg() {
    // The type argument is itself a user struct (`Box<Point>`), repeated as a
    // bare field, and both copies are accessed.
    let code = r#"
    struct Point(x: int, y: int);
    struct Box<T>(value: T);

    struct Holder(a: Box<Point>, b: Box<Point>);

    fn main() -> int {
        let h = Holder(Box(Point(1, 2)), Box(Point(3, 4)));

        return h.a.value.x + h.a.value.y + h.b.value.x + h.b.value.y;
    }
    "#;
    run_int(code, 10);
}

#[test]
fn repeated_nullable_generic_field() {
    // Same instantiation as a *nullable* field in two places. The nullable
    // wrapper takes a different code path than a bare field, so cover both.
    let code = r#"
    struct Box<T>(value: T);

    struct A(b: Box<int>?);
    struct B(b: Box<int>?);

    fn main() -> int {
        let a = A(Box(7));
        let b = B(null);

        let x = a.b?.value ?? 0;
        let y = b.b?.value ?? 100;

        return x + y;
    }
    "#;
    run_int(code, 107);
}

#[test]
fn distinct_instantiations_same_template_do_not_collide() {
    // `Box<int>` and `Box<bool>` share a template but must resolve to distinct
    // concrete types — the instantiation cache key must include the structural
    // form of the argument, not just the template.
    let code = r#"
    struct Box<T>(value: T);

    struct Mixed(i: Box<int>, b: Box<bool>);

    fn main() -> int {
        let m = Mixed(Box(5), Box(true));

        if !m.b.value {
            return 0;
        }

        return m.i.value;
    }
    "#;
    run_int(code, 5);
}

#[test]
fn generic_instantiation_as_field_and_as_local() {
    // The same instantiation is used both as a struct field and as a standalone
    // local variable type.
    let code = r#"
    struct Box<T>(value: T);

    struct Holder(inner: Box<int>);

    fn main() -> int {
        let standalone: Box<int> = Box(20);
        let h = Holder(Box(22));

        return standalone.value + h.inner.value;
    }
    "#;
    run_int(code, 42);
}

// ── Self-referential generic structs (the infinite-loop / LinkNode class) ────

#[test]
fn self_referential_generic_constructs_without_looping() {
    // The original bug: resolving a self-referential generic struct never
    // terminated. This builds a chain and reads a direct field — enough to prove
    // resolution terminates and construction works. (Reading *through* the
    // recursive field from a local is covered — and currently broken — below.)
    let code = r#"
    struct Node<T>(value: T, next: Node<T>?);

    fn main() -> int {
        let tail = Node(5, null);
        let head = Node(10, tail);

        return head.value;
    }
    "#;
    run_int(code, 10);
}

#[test]
fn self_referential_generic_recursive_field_via_struct_field() {
    // Accessing the recursive `next` field works when the `Node<int>` is reached
    // through a declared struct field (`List.head`), because the deferred field
    // pass monomorphizes it fully. Combines the recursion + repeated-field cases.
    let code = r#"
    struct Node<T>(value: T, next: Node<T>?);

    struct List(head: Node<int>, count: int);

    fn main() -> int {
        let tail = Node(40, null);
        let head = Node(2, tail);
        let list = List(head, 2);

        return list.head.value + list.head.next!!.value + list.count;
    }
    "#;
    run_int(code, 44);
}

#[test]
fn self_referential_generic_distinct_instantiations() {
    // Two distinct instantiations of the same self-referential template must not
    // be conflated by the recursion-breaking cache.
    let code = r#"
    struct Node<T>(value: T, next: Node<T>?);

    fn main() -> int {
        let i: Node<int> = Node(7, null);
        let b: Node<bool> = Node(true, null);

        if !b.value {
            return 0;
        }

        return i.value;
    }
    "#;
    run_int(code, 7);
}

// ── Nested generic substitution via the constructor / value path ─────────────
//
// A *nested* generic field (`Inner<T>` inside `Outer<T>`, or the recursive
// `Node<T>` inside `Node<T>`) must be monomorphized when the concrete type is
// produced through a constructor call — not just through a declared struct
// field. `box_arg` only fills the direct generic-parameter slots, so the call
// path re-resolves the struct's fields against the inferred args afterwards;
// without that, `o.inner.v` / `head.next!!.value` came back as the abstract `T`
// and tripped an `IllegalBinaryExpression`.

#[test]
fn nested_generic_field_via_local_value() {
    let code = r#"
    struct Inner<T>(v: T);
    struct Outer<T>(inner: Inner<T>, direct: T);

    fn main() -> int {
        let o: Outer<int> = Outer(Inner(5), 10);

        return o.inner.v + o.direct;
    }
    "#;
    run_int(code, 15);
}

#[test]
fn self_referential_recursive_field_via_local() {
    let code = r#"
    struct Node<T>(value: T, next: Node<T>?);

    fn main() -> int {
        let tail = Node(5, null);
        let head = Node(10, tail);

        return head.value + head.next!!.value;
    }
    "#;
    run_int(code, 15);
}

#[test]
fn self_referential_chain_traversal_via_local() {
    let code = r#"
    struct Node<T>(value: T, next: Node<T>?);

    fn main() -> int {
        let n3 = Node(3, null);
        let n2 = Node(2, n3);
        let n1 = Node(1, n2);

        return n1.value * 100 + n1.next!!.value * 10 + n1.next!!.next!!.value;
    }
    "#;
    run_int(code, 123);
}
