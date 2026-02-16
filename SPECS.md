# General

- statements must end with `;`
- scope is defined with `{}`
    - empty scope without any statement before it is always executed
- comments start with `//`
- multiline comments can be done using `/*` and `*/`

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

