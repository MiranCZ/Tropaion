# General

- statements must end with `;`
- scope is defined with `{}`
    - empty scope without any statement before it is always executed
- comments start with `//`
- multiline comments can be done using `/*` and `*/`
- all logic must be inside some function (except for a `const`)

# Datatypes

`bool`, `int` (32 bits), `float` (32 bits), `tuple`, `array`* ,`string`


```rust
const ARR: [int; 5] = [0, 1, 2, 3, 4];

fn main() {
    ARR.pop(); // error
    ARR.push(10); // error

    ARR[3] = 10; // error

    print(ARR[4]); // inline `4`
}
```

## Overflows
- `int.MAX + 1` is wrapped to `int.MIN`
- `int.MIN - 1` is wrapped to `int.MAX`

<br>

- `float` wrapps to the respective infinity

# Tuples
- declared using `()`
- =TODO= mutabble/immutable?
- can have values of different types
- indexed using `[]`
- `let my_tuple: (string, int) = ("hello", 5);`
- `let value = my_tuple[0];`

# Arrays
- need to have the SAME type
- size MUST be known at compile time
- declaration `let my_arr: [int; 3] = [0, 1, 2];`
- you can also specify EXACTLY one element and the whole array will be filled with it:
  - `let my_arr: [int; 3] = [6];` => `[6, 6, 6]`

=TODO= some ppl might complain that size must be known,
so maybe dynamically allocate arrays with comp-time unknown sizes?
Downside of this is someone making a mistake, allocating on heap, and not realizing

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

- do NOT have to include `()`

## For

``for (let i = 0; i < 10; i++)``

``for (let x in arr)``


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

`&` (AND), `|` (OR), `^` (XOR), `>>` (SHR*), `<<` (SHR), `~` (NOT)

\* =TODO= does right shift include sign in the shift??

# Type annotations

- variable types are implicit, can be defined with `:`

- `let x: <type> = ...`

- function arg types are explicit

## Conversion

- conversion functions (`int()`, ...)
- `bool` is evaluated to `1` if `true`, to `0` if `false`
- `bool(0)` and `bool(NaN)` is `false`, rest is `true`

# Naming

- function and variable names can contain alphanumeric symbols and '_'.
- the name must not start with a number

# Variables

- `let`
- must be scoped
- always mutable
- gets dropped after scope end
- can be reassigned with new `let`


# Constants

- `const`
- immutable
- can be in global scope
- cannot be a result of a function call (at least right now)


# Memory model
*all of this is very WIP, for now all values might as well be stored on the heap and optimizations created later*

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

- all primitives, arrays (which have a known size at compile-time) and "simple structs" 
are allocated on the stack which makes it trivial to free them
  - if a function returns a reference to some of these values (`fn foo() -> &int`) the variable gets allocated on the heap upon creation (where it can be later garbage collated)
- if the compiler cannot prove when an object stops being used it does not drop it and the object is later dropped by the GC


# Functions

- `fn`
- `return` keyword (NOT implicit)
- CAN have two functions with same name and different data types
- order of creation does not matter
- can NOT be nested (for now I guess)
- type of arguments must be defined
- return type must be defined or is implicitly `void`
- to pass a primitive argument by reference you add `&` before the arg type definition
- to pass a **non-primitive** argument by value you add the `*` operator before the arg type definition
  - this is semantically the same as calling `.clone()` on the argument once it enters the function,
but the `*` might get optimized by the compiler more easily (eq. not cloning when unnecessary)
  - therefor the `*` **must** be used only on simple structs

<br>

```rust
// `arg1` is passed by reference
// `arg2` is passed by value
fn foo(arg1: &int, arg2: *string) -> float {
    // function definition
    // ...
}
```

<br>

- might want anonymous functions if we wanna have sth like `vec.map`, but not defined for now I guess


# Structs
 
- `struct` keyword
- fields are ordered and type must be specified
- fields are declared in `()` after the struct name
- struct methods can be then declared in `{}`
  - =TODO= do the methods get passed `self`?
  - would probably rather like java-style with optional `this` to reference self params

- structs with methods do not need a `;` after
- struct without methods do need `;`

- =TODO= can structs hold references?
```
struct NameHolder(name: String);

let holder = NameHolder("jeff");
holder.name = "tom";

print(holder.name);
```


```

struct Rect(width: float, height: float) {
 
  fn circumference() -> float {
    return width + height; 
  } 
  
  fn area() -> float {
    return this.width * this.height;
  }

}

let a = Rect(5, 10);
let area = a.area();
```

