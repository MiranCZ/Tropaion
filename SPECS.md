# General

- statements must end with `;`
- scope is defined with `{}`
    - empty scope without any statement before it is always executed
- comments start with `//`
- multiline comments can be done using `/*` and `*/`

# Datatypes

`bool`, `int` (32 bits), `float` (32 bits),`array`* ,`string`

\* should array be ungrowable? (imo it should NOT growable)


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


# Functions

- `fn`
- `return` keyword (NOT implicit)
- CAN have two functions with same name and different data types
- order of creation does not matter
- can NOT be nested (for now I guess)
- type of arguments must be defined
- return type must be defined or is implicitly `void`
- to pass an argument by reference you add `&` before the arg definition

<br>

```rust
fn foo(arg1: &int, arg2: string) -> float {
    // function definition
    // ...
}
```

<br>

- might want anonymous functions if we wanna have sth like `vec.map`, but not defined for now I guess
