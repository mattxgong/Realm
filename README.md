<h1 align="center">Realm</h1>

<p align="center">
    <strong>A programming language with concurrency written in Rust.<strong>
</p>

## Characteristics

### Compilation
  * Ahead-of-Time (AOT)
  * Targets LLVM Intermediate Representation (IR) with Inkwell

### Typing
  * Statically Typed
  * Strongly Typed
  * Nominal Typing
  * Supports Generic
  * Primitive Types: `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `f16`, `f32`, `f64` `bool`, `char`

## Psuedo Code Example

```realm
var x = 10;     // Declare a mutable variable x
let y = "Hello";        // Declare an immutable variable y

var tuple: (i32, f64, string) = (42, 3.141, "Answer");      // Declare a mutable tuple with an integer, a float, and a string

var array: [bool, 5] = [true, false, true, false, true];     // Declare a mutable array of booleans with a fixed size of 5
var list: [i32] = [1, 2, 3, 4];      // Declare a mutable list of integers with dynamic size
var duplicateArray: ['d', 3];        // Declare a mutable array with duplicate values: ['d', 'd', 'd']

print(list[3]);       // Output: 4
println(y);     // Output: Hello\n
```