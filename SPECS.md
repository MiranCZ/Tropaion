# General

- statements must end with `;`
  - this might get dropped depending on how hard it is to parse it without
- scope is defined with `{}`
    - empty scope without any statement before it is always executed
- comments start with `//`
- multiline comments can be done using `/*` and `*/`
- all logic must be inside some function (except for a `const`)

# Datatypes

`bool`, `int` (32 bits, signed), `float` (32 bits), `tuple`, `array` ,`string`


## Overflows
- `int.MAX + 1` is wrapped to `int.MIN`
- `int.MIN - 1` is wrapped to `int.MAX`

<br>

- `float` wrapps to the respective infinity

# Tuples
- declared using `()`
- immutable
- can have values of different types
- indexed using `[]`


```
let my_tuple: (string, int) = ("hello", 5);
let value: string = my_tuple[0];
```

# Arrays
- need to have the SAME type
- size DOES NOT have to be known at compile time
- declaration `let my_arr: [int] = [5, 10, 15];`
- allocated on the heap (future optimization might be a stack allocation for known sizes)
- zero-indexed using `[]`, (eq `my_arr[0]` - which returns `5`)
- an index can be any expression evaluated into an int
- providing index that is out of bounds or negative will result in an error

# Strings

Indexing, slicing and adding strings is not supported.
(this might be added in the future, not a priority atm tho)

To add a string representation of some variable into a string the `$` character can be used

to format evaluated expressions `${<expr>}` can be used.

`"${1+2}"` = `"3"`

Structs are formatted as `Name(name1: value1, name2: value2...)`

```
let n: int = 5;
let text = "$n is cool!"; // "5 is cool"
```

```
let obj = Rect(1, 2);

let str = "rectangle: $obj omg"; // "rectangle: Rect(a: 1, b: 2) omg"
```

# Control flow

`if`, `while`, `for`

- do NOT have to include `()` (except `for(let ...)` I guess? that seems it would be messy)

=TODO= decide how `fori` is syntaxed, not a fan of ranges, but it being the only one with parentheses also seems weird

## For

``for (let i = 0; i < 10; i++)``

``for x in arr``


# Math ops

- must always be same types

`+`, `-`, `*`, `/`, `%` (modulo)

`++`, `--`

- `float` division by zero is `inf`
- `int` division by zero is *error*TM

<br>

- modulo is only defined for integers, `% 0` is *error*TM
- =TODO= negative modulo spec



## Boolean ops
`&&`, `||`, `!`

- lazy evaluation


## Bitwise ops

`&` (AND), `|` (OR), `^` (XOR), `>>` (SHR*), `<<` (SHL), `~` (NOT)

\* =TODO= does right shift include sign in the shift??

# Type annotations

- variable types are implicit, can be defined with `:`

- `let x: <type> = ...`

- function arg types must be explicit and struct field types

## Conversion

- conversion functions (`int()`, ...)
- `bool` is evaluated to `1` if `true`, to `0` if `false`
- `bool(0)` and `bool(NaN)` is `false`, rest is `true`

# Naming

- function and variable names can contain alphanumeric symbols and '_'
- the name must not start with a number

# Variables

- `let`
- must be scoped
- always mutable
- gets dropped after scope end
- can be shadowed with new `let`


# Constants

- `const`
- immutable
- can be in global scope
- cannot be a result of a function call (at least for now)

```rust
const ARR: [int; 5] = [0, 1, 2, 3, 4];

fn main() {
    ARR.pop(); // error
    ARR.push(10); // error

    ARR[3] = 10; // error

    print(ARR[4]);
}
```

# Null

the `null` keyword can be used to represent the absence of something.

## Basics
- If a type can be null a `?` suffix must be used after its type (eq. `let x: int? = null;`).
- Can be deconstructed into non-nulls using `??` (eq. `let y: int = x ?? 27;` - evaluates into x with value or `27`)
- the `?.` operator for func calls, either calls if not-null or returns null (eq. `some_struct?.some_func()`)
- the `!!` operator for making a nullable obj not-null (eq. `let z: int = x!!;`), the obj being null results in a runtime error

## Promotion
- any non-null type can be implicitly promoted to a nullable one

```
fn do_stuff(x: int?) {
 // ...
}

do_stuff(12); // '12' gets promoted to type `int?` from `int` at compile time
```

## Narrowing
- this is "*slightly*" hard but it would be cool to have type narrowing
```
let x: int? = some_fn();

if x != null {
  // x is known to not be null
  let y: int = 2 * x; // valid since 'x' cannot be null
  
  x = null; // ALSO VALID! 'x' is still nullable
  
  let z = 2 * x; // invalid since 'x' is no longer guaranteed to be non-null
}
```


# Memory model
*all of this is very WIP, really need to think about all of this more*

quick definition: lets call a "simple struct" any struct without circular dependencies (graph of its values types forms a tree).

=TODO= string specification

- all primitives (`bool`, `int,` `float`) are **copied by value** by default
- if you want to move a primitive by reference a `&` operator can be used (eq. `&int`)
- all other types are **passed by reference** by default
- a `.clone()` method gets generated for all **simple** `struct`s at compile-time, this method can be used to get a deep* copy of some value
  - *deep copy means that `Struct(T, E, F).clone()` is evaluated to `Struct(T.clone(), E.clone(), F.clone())`
- a `.copy()` method gets generated for **all** `struct`s at compile-time, this method can be used to get a shallow copy of some value

```
let x = 5;
let y = x; // `x` gets copied to `y`
```
<br>

```
let v = Vec();
let r = s; // `r` holds a reference to `v`
```


## Freeing memory
None of this affects `const` variables, these are immutable and are present for the whole run of the program

- all primitives, tuples and "simple structs" 
are allocated on the stack which makes it trivial to free them
  - if a function returns a reference to some of these values (`fn foo() -> &int`) the variable gets allocated on the heap upon creation (where it can be later garbage collated)
- if the compiler cannot prove when an object stops being used it does not drop it and the object is later dropped by the GC

## Notes

Still not entirely sure how much I want to care about references in a god-damn scripting language, most of this might get dropped...

- also would like some operator/keyword to copy "simple structs" by value? sth like `cpy Struct` or maybe `*Struct` in function definitions?

# Functions

- `fn`
- `return` keyword (NOT implicit)
- CAN have two functions with same name and different data types
- order of creation does not matter
- can NOT be nested (for now I guess)
- type of arguments must be defined
- return type must be defined or is implicitly `void`
- to pass a primitive argument by reference you add `&` before the arg type definition

<br>

```rust
// `arg1` is passed by value
// `arg2` is passed by reference
fn foo(arg1: int, arg2: string) -> float {
    // ... code 
  
    return some_float;
}
```

<br>

- might want anonymous functions if we wanna have sth like `vec.map`, but not defined for now I guess


# Structs
 
- `struct` keyword
- fields are ordered and type must be specified
- fields are declared in `()` after the struct name
- struct methods can be then declared in `{}`

- all functions inside a struct are instance-methods (for now)
- can access fields and methods of the struct either implicitly or by explicitly using `this.<statement>`

- structs with methods do not need a `;` after
- struct without methods do need `;`

- =TODO= can structs hold references?

- =TODO= some keyword/way of defining secondary constructors?
  - the `constructor` keyword from Kotlin seems rather long

```
// a struct without methods
struct NameHolder(name: String);

let holder = NameHolder("jeff");
holder.name = "tom";

print(holder.name); // tom
```


```
// a struct with methods
struct Rect(width: float, height: float) {
 
  fn circumference() -> float {
    return 2 * (width + height); 
  } 
  
  fn area() -> float {
    return this.width * this.height;
  }

}

let a = Rect(5, 10);
let area = a.area();
```

# Enums 

- values get replaced by `int`s at compile-time


```
enum Direction(UP, DOWN) {
    fn opposite() {
        if this == UP {
           return DOWN; 
        } else {
           return UP; 
        }
    }
}
```

# Comparing values

- the `==` operator compares the rhs and lhs **by value**
- it keeps a track of already visited types, so even circular structs can be compared

```
struct A(b: B?, i: int);
struct B(a: A);

fn create_a() -> A {
    let a = A(null, 5);
    let b = B(a);

    a.b = b;

    return a;
}


fn main() -> bool {
    let a1 = create_a();
    let a2 = create_a();

    return a1 == a2; // true
}
```

- =TODO= the `===` operator might be introduced to compare by reference in the future

# Generics

- multiple structs/functions with the specified used types are generated

```
struct Box<T>(value: T);
```