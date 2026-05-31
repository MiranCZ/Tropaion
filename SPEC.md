# General

- statements must end with `;`
    - this might get dropped depending on how hard it is to parse it without
- scope is defined with `{}`
    - empty scope without any statement before it is always executed
- comments start with `//`
- multiline comments can be done using `/*` and `*/`

# Naming

- symbol names can contain alphanumeric symbols and '_'
- the name must not start with a number

# Datatypes

`bool`, `int` (32 bits, signed), `float` (32 bits), `tuple`, `array` ,`string`

## Overflows
- `int.MAX + 1` is wrapped to `int.MIN`
- `int.MIN - 1` is wrapped to `int.MAX`
  
<br>

- `float` wrapps to the respective infinity

*note/TODO: Special value accessors (int.MAX, float.INFINITY, float.NaN, ...) are currently not implemented*

## Strings
- string constants are declared using `"<text>"`
- can be contacted using `+` only with other strings
- no more operations are currently supported
- the `str(<arg>)` function can be used to convert numbers to a string


## Tuples
- declared using `()`
- immutable
- can have values of different types
- indexed using `.<index>`

```
let my_tuple: (string, int) = ("hello", 5);
let value: string = my_tuple.0;
```

# Math ops

- must always be same types
- the short `<op>=` syntax can be used for appropriate types where `x <op>= y` is semantically same as `x = x <op> y`
  - eq. `x += y` is same as `x = x + y`

`+`, `-`, `*`, `/`, `%` (modulo)

`++`, `--`

- `float` division by zero is `inf`
- `int` division by zero is *error*TM

*note/TODO: this is probably not finished*


## Bitwise ops

`&` (AND), `|` (OR), `^` (XOR), `>>` (SHR*), `<<` (SHL), `~` (NOT)

## Boolean ops
`&&`, `||`, `!`

- lazy evaluation

# Control flow

`if`, `while`, `for`

- do NOT have to include `()`

``if <condition> {...}``

``for let i = 0; i < 10; i++ {...}``

# Type annotations

- variable types are implicit, can be defined with `:`

- `let x: <type> = ...;`

- function arg types and struct field type smust be explicit 

## Conversion

- conversion functions (`int()`, ...)
- `bool` is evaluated to `1` if `true`, to `0` if `false`
- `bool(0)` and `bool(NaN)` is `false`, rest is `true`

# Variables

- `let`
- must be scoped
- always mutable
- can be shadowed with new `let`

# Constants

*note/TODO: CONSTANTS ARE CURRENTLY NOT IMPLEMENTED*

- `const`
- immutable
- can be in global scope
- cannot be a result of a function call (at least for now)

# Null & Nullable

the `null` keyword can be used to represent the absence of something.

## Basics
- If a type can be `null` a `?` suffix must be used after its type (eq. `let x: int? = null;`).
- Can be deconstructed into non-nulls using `??` (eq. `let y: int = x ?? 42;` - evaluates into `x` with value or `42`)
- the `?.` operator for function calls, either calls if not-null or returns null
  - eq. `let v = obj?.member();` is semantically same as 
  ```
  let v = null;
  if obj != null {
    v = obj.member();
  }
  ```
- the `!!` operator for asserting a nullable obj not-null (eq. `let z: int = x!!;`), the obj being null results in a runtime error

## Promotion
- any non-null type can be implicitly promoted to a nullable one

```
fn do_stuff(x: int?) {
 // ...
}

do_stuff(12); // '12' gets promoted to type `int?` from `int` at compile time
```




# Memory model

- garbage collection
- uhh, details are *implementation specific* (:D)


# Functions

- `fn`
- to return a value the `return` keyword must be used
- CAN have two functions with same name and different parameters
- order of creation does not matter
- can NOT be nested (for now I guess)
- type of arguments must be defined
- return type must be defined or implicitly does not return anything


```
fn add(a: int, b: int) -> int {
    return a + b;
}
```

<br>

# Structs

- `struct` keyword
- fields are ordered and type must be specified
- fields are declared in `()` after the struct name
- struct methods can be then declared in `{}`

- all functions inside a struct are instance-methods (for now at least I guess)
- can access fields and methods of the struct either implicitly or by explicitly using `this.<statement>`

- structs with methods do not need a `;` after
- struct without methods do need `;` after their fields declaration
```
// a struct without methods
struct NameHolder(name: String);

fn main() {
    let holder = NameHolder("jeff");
    holder.name = "tom";
    
    print(holder.name); // tom
}
```

- if you want a function inside a struct to be visible outside the structs scope the `pub` keyword must be used

- constructor is always implicit by the structs fields or other constructors can be defined using the `init` keyword
    - please note that for the constructor to be public `pub` must be used
    ```
    struct IntBox(value: int?) {
        pub init() {
            this(null); 
        }
    }
    ```

- the `priv` modifier can be used to hide the default constructor and the structs fields (although you cannot hide only fields/only constructor)
```
struct Vec<T> priv(<secret internal pointer magic>) {
    
    // constructor for `Vec<T>()`
    pub init() {
        this(<pointer magic initialization)
    }
    
}
```

example:
```
// an example struct with methods
struct Rect(width: float, height: float) {
 
  pub fn circumference() -> float {
    return 2 * (width + height); 
  } 
  
  pub fn area() -> float {
    return this.width * this.height;
  }

}

fn main() {
    let a = Rect(5, 10);
    let area = a.area();
}
```

# Enums
 
- `enum` keyword
- values get replaced by `int`s at compile-time


```
enum Direction(UP, DOWN) {
    fn opposite() -> Direction {
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


fn main() {
    let a1 = create_a();
    let a2 = create_a();

    print(a1 == a2); // true
}
```

*note/TODO*: the `===` operator might be introduced to compare by reference in the future


# Generics

- multiple structs/functions with the specified used types are generated
- generics are resolved into concrete types at compile-time (like `C`s templates)
- you can annotate generics types by putting the generic type into `<>`, eq. `let x: Vec<int> = ...`

```
struct Box<T>(value: T);
```

# Panicking

- `panic` function (can optionally take a string as input)
- non-recoverable


# Built-in helpers

*note/TODO: these are ideally gonna grow in the future*

- `Vec<T>`: Growable array
  - `push(element: T)`, adds element `T` at the end of the vector
  - `set(index: int, element:T)`, sets element at index to the specific element
  - `get(index: int) -> T`, returns `T` or panic if OOB
  - `pop() -> T`, returns and removes last element or panics if empty
  - `size() -> int` return size of vector
- `print`
- `panic`
- `str(float/int)`
- `float(int)`
- `int(float)`
