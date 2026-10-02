---
title: Realm Type System
description: Normative value, typing, conversion, constant, and initial target-representation rules for Realm
ms.date: 2026-08-27
ms.topic: reference
---

## Status and Scope

This specification defines the Realm version 0 type-system contract owned by
`RLM-0002`. It resolves `REQ-008`, `REQ-009`, and `REQ-016` for primitive value
domains, literals, operations, casts, identity, body-local inference,
coercions, aggregates, constants, public type surfaces, and the initial
`x86_64-pc-windows-msvc` representation boundary.

The language owner accepted this contract and its bounded validation evidence
on 2026-08-27.

The terms MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY describe requirements on
conforming Realm implementations. Source behavior is identical in debug and
optimized builds unless this specification explicitly says otherwise.

This contract does not implement or claim conformance of a production type
checker or LLVM backend. Generic constraints, ownership, exception ABI, async
identity, sendability, explicit C layout, and a stable external ABI remain
downstream. The Phase C disposable oracle is non-normative, table-level
evidence and does not establish production conformance.

## Judgment Notation

The normative rules use these judgments:

```text
Gamma |- e => T ~> e'                 e synthesizes T and elaborates to e'
Gamma |- e <= T ~> e'                 e checks against expected T
Sigma |- T == U                       T and U have semantic type identity
T ~> U                                one permitted implicit coercion exists
v in T                                mathematical value v inhabits T
op_T(v1, ..., vn) => v                typed operation returns v
op_T(v1, ..., vn) => panic            typed operation invokes Realm panic
cast(S, T, v) => v'                   explicit cast succeeds
cast(S, T, v) => panic                explicit cast fails at runtime
Gamma |-const e => v : T              constant expression evaluates to v
Gamma |-const e => diagnostic         required constant is invalid
layout_target(T) = (size, align, rep)  target storage and representation
abi_target(T, position) = class       target calling classification
```

`Gamma` contains body-local bindings and collected item signatures. `Sigma`
contains canonical type and declaration identities. Layout and ABI judgments
MUST NOT be premises of source-level identity, inference, overload resolution,
or coercion. Move, loan, lifetime, drop, and place-identity judgments are
deliberately absent and belong to `RLM-0005`.

Representative derivations are:

```text
Gamma(x) = i32
-------------------------------- synth-checked-add
Gamma |- x + 1 => i32 ~> add.checked.i32(x, 1)

Gamma |- return value => ! ~> terminate(value)
----------------------------------------------- never-coerce
Gamma |- return value <= T ~> terminate(value)

Gamma |- a => &mut [T; N]
----------------------------------------------- array-slice-shared
Gamma |- a <= &[T] ~> slice.shared(a.data, N)

Sigma(A) = DefId(package, module, 1)
Sigma(B) = DefId(package, module, 2)
----------------------------------------------- nominal-distinct
Sigma !|- A == B
```

## Primitive Types and Value Domains

Realm version 0 has exactly these primitive types:

| Type  | Category         | Value domain                                      | Width | Exact suffix | Fallback | Windows x64 storage |
|-------|------------------|---------------------------------------------------|------:|--------------|----------|---------------------|
| `i8`  | Signed integer   | $-2^7$ through $2^7-1$                            |     8 | `i8`         | No       | 1 byte, align 1     |
| `i16` | Signed integer   | $-2^{15}$ through $2^{15}-1$                      |    16 | `i16`        | No       | 2 bytes, align 2    |
| `i32` | Signed integer   | $-2^{31}$ through $2^{31}-1$                      |    32 | `i32`        | Integer  | 4 bytes, align 4    |
| `i64` | Signed integer   | $-2^{63}$ through $2^{63}-1$                      |    64 | `i64`        | No       | 8 bytes, align 8    |
| `u8`  | Unsigned integer | $0$ through $2^8-1$                               |     8 | `u8`         | No       | 1 byte, align 1     |
| `u16` | Unsigned integer | $0$ through $2^{16}-1$                            |    16 | `u16`        | No       | 2 bytes, align 2    |
| `u32` | Unsigned integer | $0$ through $2^{32}-1$                            |    32 | `u32`        | No       | 4 bytes, align 4    |
| `u64` | Unsigned integer | $0$ through $2^{64}-1$                            |    64 | `u64`        | No       | 8 bytes, align 8    |
| `f32` | Floating point   | IEEE 754 binary32                                 |    32 | `f32`        | No       | 4 bytes, align 4    |
| `f64` | Floating point   | IEEE 754 binary64                                 |    64 | `f64`        | Float    | 8 bytes, align 8    |
| `bool`| Boolean          | Exactly `false` and `true`                        |     1 | None         | N/A      | 1 byte, align 1     |
| `char`| Character        | U+0000..U+D7FF and U+E000..U+10FFFF               |    32 | None         | N/A      | 4 bytes, align 4    |
| `()`  | Unit             | One value, also spelled `()`                      |     0 | None         | N/A      | 0 bytes, align 1    |
| `!`   | Never            | No values                                         |   N/A | None         | N/A      | No layout           |

The fixed integer ranges are mathematical ranges. Realm version 0 has no
`isize`, `usize`, `i128`, `u128`, or `f16`. These names are not aliases, and
there is no `f16` suffix. An available LLVM integer or floating type does not
add a Realm source type.

`bool` has only two valid values. Addressable storage contains canonical byte
`0` or `1`; any other byte introduced through a later unsafe or foreign
boundary is invalid. Boolean arrays are byte arrays, not bitsets.

`char` denotes a Unicode scalar value, not a UTF-8 byte, UTF-16 code unit, or
integer subtype. Surrogate values and values above U+10FFFF are invalid.

Unit is a zero-sized singleton. Never is an uninhabited semantic type with no
layout. A path with type `!` can satisfy any expected type only through the
never coercion.

## Literal Typing

### Numeric Token Composition

The lexer emits a numeric token and any following identifier as separate
tokens. The parser composes the pair into one suffixed literal only when their
source ranges are immediately contiguous and the identifier spelling is one of
the exact suffixes in the primitive table. Trivia of any kind prevents
composition.

Integer-shaped syntax accepts an integer or floating suffix. Decimal syntax
with a point or exponent accepts only `f32` or `f64`. Therefore `1u8`, `1f32`,
and `1.0f32` are valid, while `1.0i32`, `1 i32`, and `1widget` are not suffixed
numeric literals.

### Context, Fallback, and Fitting

Numeric literal text is parsed as an unbounded mathematical integer or an exact
decimal input before a concrete type is selected. An expected compatible
numeric primitive selects the literal type first. An exact suffix overrides
fallback and must agree with any expected type. If no expected type or suffix
selects a type, an integer-shaped literal defaults to `i32` and a decimal-point
or exponent literal defaults to `f64`.

The selected integer value MUST fit its exact domain. Decimal conversion MUST
round correctly to the selected IEEE format using round-to-nearest,
ties-to-even. A fallback value that does not fit is a diagnostic asking for an
expected type or suffix; the compiler MUST NOT widen it automatically.

Unary `+` and `-` are not part of a numeric token. The sign is applied to the
mathematical value before the final representability check. Consequently,
`-2147483648` can inhabit `i32`, while positive `2147483648` cannot.

Contextual literal typing does not convert an already typed value and is not an
implicit numeric coercion.

```realm
let default_integer = 10;          // i32
let default_float = 3.141;         // f64
let byte: u8 = 255;                // expected u8
let wide = 4_294_967_295u32;       // exact u32 suffix
let rounded = 0.1f32;              // correctly rounded binary32
let minimum: i32 = -2147483648;    // sign applied before fitting
```

```realm
let too_large = 2147483648;        // does not fit fallback i32
let bad: u8 = 256;                 // does not fit expected u8
let split = 1 i32;                 // whitespace forbids suffix composition
let bad_float = 1.0i32;            // integer suffix on float syntax
```

Character literals check that escape processing produces exactly one Unicode
scalar value. String literals are Unicode scalar sequences encoded as UTF-8
and synthesize the nominal prelude type `String` when no other rule rejects the
context. Realm version 0 has no implicit borrowed-string literal type.

## Integer and Boolean Operations

Ordinary integer operators never widen, change signedness, or mix concrete
integer types. Except for shift counts, binary integer operands MUST have the
same type `T`, and the result remains `T`. All operands evaluate left to right.

| Operation       | Operand rule                                | Result or failure |
|-----------------|---------------------------------------------|-------------------|
| Unary `+`       | Integer `T`                                 | Identity in `T` |
| Unary `-`       | Signed integer `T`                          | Mathematical negation; panic on `MIN_T` |
| `+`, `-`, `*`   | Same integer `T`                            | Mathematical result in `T`; panic if unrepresentable |
| `/`             | Same integer `T`                            | Quotient, truncated toward zero when signed; panic on zero and `MIN_T / -1` |
| `%`             | Same integer `T`                            | Truncated-quotient remainder; same panic cases as `/` |
| `<<`            | Integer `T`, any integer count              | Mathematical $lhs * 2^{count}$ in `T`; panic on invalid count or unrepresentable result |
| `>>`            | Integer `T`, any integer count              | Logical when unsigned and sign-extending when signed; panic on invalid count |
| `&`, `|`, `^`   | Same integer `T`                            | Bitwise result in `T` |
| Unary `!`       | Integer `T`                                 | Bitwise complement in `T` |
| `==`, `!=`      | Same integer `T`                            | `bool` using `T` values |
| `<`, `<=`, `>`, `>=` | Same integer `T`                      | `bool` using signed or unsigned ordering of `T` |
| `&&`, `||`      | Two `bool` values                           | Short-circuit Boolean result |
| `&`, `|`, `^`   | Two `bool` values                           | Eager Boolean result; both operands evaluate |
| Unary `!`       | `bool`                                      | Logical negation |
| `==`, `!=`      | Two `bool` values                           | Boolean equality result |

For signed division and remainder, `a = (a / b) * b + a % b`; a nonzero
remainder has the dividend's sign. A shift count is invalid when negative or at
least the bit width of `T`. Left shift also panics when its mathematical result
does not fit `T`, including discarded or sign-changing bits.

Compound assignment evaluates its target place once and applies the
corresponding checked operation. Assignment and compound assignment produce
`()` after updating a mutable compatible place. Assignment chaining therefore
fails ordinary type checking because the inner assignment has unit type.

Any integer type may be used as an index or slice bound. Bounds logic checks
negativity and mathematical magnitude directly; it MUST NOT first narrow the
index into a common integer type.

At runtime, integer overflow, negation overflow, division or remainder by zero,
`MIN_T / -1`, `MIN_T % -1`, and invalid shifts invoke the accepted
non-catchable, cleanup-preserving Realm panic. In a required constant, the same
condition is a compile-time diagnostic. These results MUST be identical in
debug and optimized builds.

## Floating-Point Operations

`f32` and `f64` use strict IEEE 754 binary32 and binary64 behavior in the
default floating environment:

* Rounding is round-to-nearest, ties-to-even
* Floating exceptions are masked and are not Realm effects
* Subnormals are preserved; gradual underflow applies
* Overflow produces signed infinity
* NaNs, infinities, and signed zeros are values
* Floating division by zero follows IEEE 754 and does not invoke integer panic

Binary arithmetic and comparisons require identical operand types. There is no
implicit `f32` to `f64` promotion.

| Operation       | Semantics |
|-----------------|-----------|
| `+`, `-`, `*`, `/` | Target-format IEEE operation with one result rounding |
| `%`             | Truncated-quotient remainder equivalent to `fmod` or LLVM `frem`, not IEEE `remainder` |
| Unary `+`       | Identity in the same format |
| Unary `-`       | Change the sign, including signed zero; NaN sign follows the target operation |
| `==`            | True for equal non-NaN values and both zero signs; false if either operand is NaN |
| `!=`            | Logical inverse of `==`; true if either operand is NaN |
| `<`, `<=`, `>`, `>=` | False if either operand is NaN; otherwise IEEE ordered comparison |

NaN payload and sign are unspecified, but any NaN result remains NaN. Ordinary
code MUST NOT reassociate operations, assume finite inputs, discard signed
zero, approximate division, or contract multiply-add. LLVM fast-math flags are
forbidden for ordinary Realm operations. A fused operation requires a future
explicit intrinsic or library API.

Compile-time evaluation MUST reproduce target-format rounding and comparison;
host Rust behavior is not normative evidence. The runtime owns preservation of
the documented default Windows `MXCSR` control bits across Realm calls. Dynamic
rounding modes and observable floating exceptions are not language features.

## Other Built-In Operations

Characters support equality and ordering by Unicode scalar numeric value. They
do not support arithmetic or bitwise operators. Unit equality is always true,
and unit inequality is always false; unit has no ordering. Boolean ordering is
unavailable.

String, tuple, and array equality or ordering is unavailable in version 0.
Generic element constraints and library operations may add those capabilities
without changing the built-in scalar rules.

No operator overloading or user-defined implicit conversion exists in version
0. A syntactically valid operator with no row in this specification is a static
type error.

## Explicit Casts

An explicit cast is written `expression as Type`. It binds less tightly than
prefix and postfix operations and more tightly than multiplicative operators.
Direct cast chaining is a syntax error; use `(value as T) as U`.

The following table is the complete safe `as` cast set. Source and target
categories not admitted by a row are static type errors.

| Source              | Target              | Success rule | Failure rule |
|---------------------|---------------------|--------------|--------------|
| Integer `S`         | Integer `T`         | Preserve the mathematical value when representable in `T` | Panic if not representable |
| Integer             | `f32` or `f64`      | Round to nearest, ties-to-even, in the target format | No failure for version 0 integer widths |
| `f32` or `f64`      | Integer `T`         | Truncate toward zero, then preserve the mathematical integer in `T` | Panic for NaN, infinity, or an out-of-range truncated result |
| `f32`               | `f64`               | Exact widening | None |
| `f64`               | `f32`               | IEEE target-format rounding, including infinity on overflow and signed zero on underflow | None |
| `char`              | Integer `T`         | Preserve the scalar numeric value when representable in `T` | Panic when `T` cannot represent the value |
| Integer             | `char`              | Produce the scalar with that numeric value | Panic for negative, surrogate, or above U+10FFFF |
| Any non-never `T`   | The same `T`        | Identity; the compiler SHOULD diagnose a redundant cast | None |

The complete rejected set is:

* `bool` to any numeric type or `char`, and the reverse direction
* Any cast to or from `()`, except the redundant identity cast
* Any cast involving `!`; never uses its implicit coercion instead
* Any cast involving references, slices, arrays, tuples, `String`, function
  items, function pointers, or nominal declarations, except redundant identity
* Any cast between distinct unrelated nominal or structural aggregate types
* Pointer, bit-pattern, wrapping, saturating, and unchecked conversions

A checked cast failure invokes the same non-catchable, cleanup-preserving Realm
panic in every runtime build profile. In required constant evaluation, the same
case is a diagnostic. Wrapping, saturating, bit-pattern, pointer, or unchecked
operations require future named APIs and are never alternate meanings of `as`.

LLVM conversion instructions are lowering tools, not source semantics.
Float-to-integer range and NaN checks MUST occur before `fptosi` or `fptoui`
because invalid LLVM conversions can produce poison. Integer range checks MUST
precede value-losing truncation. No source panic may be represented as poison or
undefined behavior.

## Type Identity

Primitive types have language-defined intrinsic identity. Every `struct` and
`enum` declaration has nominal identity determined by its declaration `DefId`,
not its spelling, fields, variants, visibility, or target layout. Distinct
declarations remain distinct even when structurally identical.

`String` is nominally identified by its prelude declaration, not by its
three-word representation. The stable cross-package component of `DefId`
remains package-design work; version 0 examples use session-local declaration
identity.

Built-in aggregate identities are structural:

| Type form             | Identity components |
|-----------------------|---------------------|
| Tuple                 | Ordered element-type sequence |
| Array `[T; N]`        | Element type `T` and constant length `N` |
| Shared reference      | Shared mutability and referent type |
| Exclusive reference   | Exclusive mutability and referent type |
| Slice referent `[T]`  | Element type `T` |
| Function pointer      | Parameter sequence, return type, and declared `throws` effect |

Calling class, size, alignment, field offsets, and LLVM shape are never identity
components. Generic substitutions and constraint identity belong to
`RLM-0004`; async function identity belongs to `RLM-0007`.

```realm
struct Left { value: i32, }
struct Right { value: i32, }

fn reject(value: Left) -> Right {
    value
}
```

The return is a static type error because `Left` and `Right` have distinct
nominal identities despite equal fields and initial layouts.

## Local Inference

Type checking operates per function body against previously collected item
signatures. It may infer local binding and literal types, propagate expected
types, and solve equality constraints within that body. It MUST NOT infer
across function bodies or from downstream callers.

The version 0 inference boundary is:

* Every function parameter is explicitly typed
* A private function with no written return type returns `()`; its body does
  not infer another return type
* Every public function writes its return type, including `-> ()`
* Structure fields, enum payloads, function-type components, and constants are
  explicitly typed
* A local `let` or `var` may omit its annotation because it has an initializer
* Expected types flow into literals, array and tuple elements, returns,
  assignments, and call arguments
* Equality constraints flow backward only inside the current body
* Local bindings are monomorphic; there is no let-generalization
* Empty array literals require an expected `[T; 0]` type
* Numeric defaults apply only after contextual constraints are exhausted
* Any other unresolved inference variable is a diagnostic
* Recursive and mutually recursive calls use collected explicit signatures

Generic argument inference is not defined here and MUST NOT broaden this
boundary when `RLM-0004` specifies it.

```realm
fn local() -> i32 {
    let value = 1;
    value
}

pub fn reset() -> () {
    ()
}
```

```realm
pub fn missing_public_return() {
    ()
}

fn unresolved() -> () {
    let empty = [];
}

fn unit_by_default() {
  ()
}

fn caller_cannot_refine() -> i32 {
  unit_by_default()
}
```

The first invalid function omits a public return type. The second lacks an
expected type for the empty array. The final caller cannot refine the preceding
function's omitted return type from `()` to `i32`.

## Implicit Coercions

The implicit coercion relation is closed. A conforming implementation permits
exactly these cases and records every inserted coercion in typed HIR:

| Source          | Target          | Elaboration |
|-----------------|-----------------|-------------|
| `T`             | Identical `T`   | Identity conversion |
| `!`             | Any expected `T`| Noncontinuing never coercion |
| `&mut T`        | `&T`            | Shared reborrow |
| `&mut [T; N]`   | `&mut [T]`      | Exclusive array-to-slice view |
| `&[T; N]`       | `&[T]`          | Shared array-to-slice view |
| `&mut [T; N]`   | `&[T]`          | Mutability weakening followed by shared array-to-slice view |
| Function item   | Matching `fn(P...) -> R` | Noncapturing function-pointer value |

There are no implicit integer widenings, signedness changes, integer/float
conversions, Boolean conversions, nominal-wrapper conversions, string
allocations, ownership clones, general dereference chains, or user-defined
coercions. Literal contextual typing is not a coercion because a literal has no
prior concrete type.

Reference reborrow validity, duration, exclusivity, variance, copy eligibility,
and cleanup interaction remain `RLM-0005` responsibilities. This table fixes
only type compatibility and elaboration shape.

## Public Type Surfaces

Public type surfaces include every `pub fn`, `pub const`, public structure
field, public enum payload, and public function-typed value. Every component of
such a surface MUST be explicitly typed and contain no unresolved inference
variable. Public functions explicitly state parameter types, return type, and
declared effects; unit return is written `-> ()`.

Whether a referenced type is visible enough for a public API is a resolver and
visibility check, not an inference rule. This specification does not promise
that a public Realm signature has a stable external binary ABI.

## Constants

Every `const` has the explicit type required by the grammar. Its initializer is
checked against that type and may contain only:

* Boolean, character, integer, and floating literals
* Unit, tuple, fixed-array, and array-repeat construction
* References to previously resolved constants without initialization cycles
* Parentheses
* The unary, binary, comparison, and explicit cast operations defined here

Calls, loops, mutable bindings, assignment, `throw`, `await`, owned strings,
references, slices, structures, enums, unsafe operations, conditional
expressions, and match expressions are not constant-evaluable in version 0.
A rejected construct is a constant-evaluation diagnostic, not evidence that the
same expression is invalid at runtime.

Constant arithmetic and casts use the same value semantics as runtime
operations. A runtime panic condition becomes a compile-time diagnostic.
Floating constants use target-format rounding. An initialization cycle is a
diagnostic.

Array lengths additionally require a nonnegative integer constant representable
as `u64`. Compilation MUST reject a length whose target allocation size
overflows `u64` or the target address space. String-valued constants are
rejected because version 0 does not define immortal ownership or a hidden
static-borrow string type.

```realm
const LIMIT: u64 = (1u64 << 20u8) - 1u64;
const EMPTY: [u8; 0] = [];
const POINT: (i32, i32) = (1, 2);
```

```realm
const BAD_DIVISION: i32 = 1 / 0;
const MESSAGE: String = "hello";
const CALLED: i32 = compute();
```

## Arrays and Tuples

An array `[T; N]` has exactly `N` elements of one type `T`. Its length is part
of type identity and follows the constant restrictions above. `N = 0` is valid.
An array with zero-sized elements preserves its semantic length while occupying
zero data bytes.

Array-list elements check against one expected `T`. Without an expected type,
the body-local checker infers one `T` using equality and only the closed
coercion set. An array-repeat expression checks its value against `T` and its
count as an array length. `[]` requires an expected `[T; 0]` type.

Array storage is contiguous with element allocation stride and alignment. The
target size is `N * alloc_size(T)`, checked for target overflow during
compilation.

A tuple is a structural ordered product. Elements retain source order and use
zero-based integer projections. `()` is unit, `(T)` is `T`, and `(T,)` is a
one-element tuple. Each target element begins at the next offset satisfying its
alignment, and total size rounds up to maximum element alignment. The compiler
MUST NOT reorder tuple elements in optimized builds. The empty tuple has size
zero and alignment one. A nonempty tuple containing only zero-sized elements
has size zero and retains the maximum member alignment.

Array and tuple equality and ordering remain unavailable until generic
constraints define their element requirements.

## Slices and References

`[T]` is an unsized contiguous sequence referent. It cannot appear by value in
a binding, parameter, return, field, tuple, or array. Shared and exclusive slice
values have type `&[T]` and `&mut [T]`.

```realm
fn inspect(values: &[i32]) -> i32 {
    values[0]
}

fn replace(values: &mut [i32]) -> () {
    values[0] = 1;
}
```

For sized `T`, `&T` and `&mut T` have one non-null, aligned, dereferenceable
pointer in the initial representation. Slice references have a non-null,
element-aligned data pointer and a `u64` element count. Length counts elements,
not bytes. Empty slices may use an approved aligned dangling sentinel, which is
never dereferenced while the length is zero.

Slicing constructs a shared or exclusive slice reference according to the
receiver rule in [grammar.md](grammar.md). Array-reference coercions construct
the same representation using the array data pointer and constant length.

Bounds normalization and panic behavior are defined by the surface grammar.
Reference validity duration, alias conflicts, reborrow lifetimes, and
zero-sized-place identity remain `RLM-0005` work.

## Owned String

`String` is a nominal prelude library type, not a primitive or structural tuple.
Its values own contiguous valid UTF-8 bytes and maintain
`0 <= len <= capacity`. Length and capacity count bytes. String literals
synthesize `String` when admitted by their context, which may require allocation
or copying under the future ownership and library contract.

The initial Windows x64 storage is `{ptr, u64 len, u64 capacity}` in that order,
with size 24 and alignment 8. The data pointer is non-null and suitably aligned;
an empty value uses the library's approved non-null aligned sentinel.

Direct string indexing and bracket slicing are type errors. Explicit library
APIs provide byte and Unicode-scalar iteration and may provide checked UTF-8
boundary slicing. Allocation, move, clone, destruction, and future borrowed
string views remain `RLM-0005` and standard-library work.

## Initial Target Representation

This table closes the initial internal target boundary for
`x86_64-pc-windows-msvc`. It does not define a stable Realm-native ABI, C ABI
exposure, cross-target layout, or user-visible type identity.

| Realm type             | Windows x64 storage                         | LLVM boundary                    | Default Windows x64 call boundary |
|------------------------|---------------------------------------------|----------------------------------|-----------------------------------|
| `i8`, `u8`             | 1 byte, align 1                             | `i8`                             | Integer class, right-justified |
| `i16`, `u16`           | 2 bytes, align 2                            | `i16`                            | Integer class, right-justified |
| `i32`, `u32`           | 4 bytes, align 4                            | `i32`                            | Integer class, right-justified |
| `i64`, `u64`           | 8 bytes, align 8                            | `i64`                            | Integer class |
| `f32`                  | 4 bytes, align 4                            | `float`                          | Positional XMM register or stack |
| `f64`                  | 8 bytes, align 8                            | `double`                         | Positional XMM register or stack |
| `bool`                 | Canonical `i8`, 1 byte, align 1             | `i1` in SSA, `i8` in memory      | One-byte integer class with explicit canonicalization and zero-extension |
| `char`                 | `i32`, 4 bytes, align 4                     | `i32`                            | Four-byte integer class |
| `()`                   | 0 bytes, align 1                            | No payload or `void` return      | Omitted payload |
| `!`                    | No layout                                   | `unreachable` control flow       | No callable value |
| `String`               | `{ptr, u64, u64}`, 24 bytes, align 8        | Backend struct or internal scalars | Indirect argument; hidden result pointer return |
| `&T`, `&mut T`         | 8-byte pointer, align 8                     | `ptr`                            | Pointer class |
| `&[T]`, `&mut [T]`     | `{ptr, u64}`, 16 bytes, align 8             | Backend struct or internal scalars | Indirect argument; hidden result pointer return |
| `[T; N]`               | Contiguous `N * alloc_size(T)`              | LLVM array when representable    | Omitted at size zero; otherwise size-dependent |
| Tuple                  | Source-order padded aggregate               | LLVM struct when representable   | Omitted at size zero; otherwise size-dependent |
| `fn(P...) -> R`        | 8-byte code pointer                         | `ptr` plus semantic signature    | Pointer class; call uses lowered signature ABI |

Without a hidden result pointer, the first four argument positions use the
Windows x64 positional integer or XMM classes as appropriate. Boolean values
are canonicalized and zero-extended when crossing the call boundary. A
zero-sized aggregate payload is omitted and requires no address or sentinel.
A nonzero aggregate argument of 1, 2, 4, or 8 bytes may use a same-sized integer
class; other aggregate arguments are indirect through a caller-created,
appropriately aligned temporary.

A return eligible for the same-sized 1-, 2-, 4-, or 8-byte aggregate class uses
the integer return register. Other nonzero aggregate returns use a hidden
caller-provided, appropriately aligned result pointer. The hidden pointer
occupies the first integer argument position and shifts declared argument
positions. `String` and slice-reference returns use this hidden-pointer rule.
These classifications are backend facts and MUST NOT enter source type
identity.

Arrays use element alignment and allocation stride. Tuples use source order,
internal padding, maximum-member alignment, and tail padding. Required layout
examples are:

| Type                | Size | Alignment | Notes |
|---------------------|-----:|----------:|-------|
| `[(); 0]`           |    0 |         1 | Logical length zero |
| `[(); 4]`           |    0 |         1 | Logical length four, zero-sized elements |
| `[u8; 0]`           |    0 |         1 | Zero-length byte array |
| `[u64; 0]`          |    0 |         8 | Zero length retains element alignment |
| `([u64; 0],)`       |    0 |         8 | Nonempty zero-sized tuple retains member alignment |
| `(u8, u64, u16)`    |   24 |         8 | Offsets 0, 8, and 16 with tail padding |
| `String`            |   24 |         8 | Pointer, byte length, capacity |
| `&[T]`              |   16 |         8 | Pointer and element length |

LLVM lowering MUST guard integer division, remainder, shifts, overflow, and
float-to-integer conversion before emitting operations whose invalid inputs can
produce poison or undefined behavior. Ordinary floating operations carry no
fast-math flags. LLVM modules set the selected target triple and data layout and
remain subject to verification before and after optimization under `REQ-037`.

## Positive and Negative Conformance Examples

These examples summarize the closed rules and do not claim that a production
checker currently executes them.

| Case | Example | Required result |
|------|---------|-----------------|
| Positive | `let x: i16 = 32767;` | Checks as `i16` |
| Negative | `let x: i16 = 32768;` | Literal-fit diagnostic |
| Positive | `let x = 1u8 + 2u8;` | Synthesizes `u8` with checked addition |
| Negative | `let x = 1u8 + 2u16;` | Static type error; no numeric coercion |
| Positive | `let x = 255u16 as u8;` | Produces `255u8` |
| Negative | `let x = 256u16 as u8;` | Runtime panic or constant diagnostic |
| Positive | `let c = 0x10ffffu32 as char;` | Produces U+10FFFF |
| Negative | `let c = 0xd800u32 as char;` | Runtime panic or constant diagnostic |
| Positive | `let view: &[i32] = &array;` | Shared array-to-slice coercion |
| Negative | `let view: [i32] = array;` | Unsized-by-value type error |
| Positive | `let shared: &i32 = exclusive;` | Shared reborrow from `&mut i32` |
| Negative | `let wide: i64 = narrow_i32;` | Static type error without `as` |
| Positive | `pub fn done() -> () { () }` | Explicit public unit signature |
| Negative | `pub fn done() { () }` | Public return-type diagnostic |
| Positive | `pub struct Callbacks { pub run: fn(i32) -> (), }` | Every public function-type component is explicit |
| Negative | `pub struct Callbacks { pub run: fn(i32), }` | Public function-type return diagnostic |
| Positive | `const A: i32 = 40 + 2;` | Evaluates to `42i32` |
| Negative | `const A: i32 = 1 / 0;` | Constant-evaluation diagnostic |

## README Reconciliation

The root README is non-normative and source compatibility with its examples is
an explicit non-goal. Realm version 0 resolves its type claims as follows:

| README claim | Normative disposition |
|--------------|-----------------------|
| `f16` primitive | Deferred; no version 0 type or suffix |
| Lowercase `string` | Replaced by nominal prelude `String` |
| `[bool, 5]` | Fixed array type is `[bool; 5]` |
| `[i32]` as an owned list | `[i32]` is an unsized slice referent; owned `Vec<i32>` remains generic-library work |
| `['d', 3]` | Repeat expression is `['d'; 3]` with type `[char; 3]` |
| Unsuffixed `10` | Contextual first, then `i32` fallback |
| Unsuffixed `3.141` | Contextual first, then `f64` fallback |
| Unspecified casts and tuple layout | Replaced by the checked cast and target-representation rules in this specification |

## Ownership Boundaries

| Normative type-system semantics | Initial target and backend facts | Explicitly downstream |
|----------------------------------|----------------------------------|-----------------------|
| Primitive domains, literal fitting and defaults, operator typing, panic conditions, IEEE results, cast legality, nominal identity, body-local inference, coercions, aggregate identity, constant eligibility, public type surfaces | Byte size, alignment, padding, register or indirect class, LLVM value shape, overflow guards, panic-edge lowering, and verification | Generic constraints and substitutions (`RLM-0004`); moves, copy eligibility, loans, lifetimes, drops, unsafe pointer casts, and zero-sized-place identity (`RLM-0005`); exception ABI (`RLM-0006`); async identity (`RLM-0007`); sendability (`RLM-0008`); C layout and FFI (`REQ-049`) |

The target table is an initial compiler/runtime integration contract. It does
not promise stable external ABI, complete type-checker conformance, generic or
ownership semantics, or finished LLVM lowering.

## Requirement and Evidence Mapping

Phase B provides normative documentary evidence. Phase C adds 901 ordered
table-oracle rows with matching debug and release digest
`fnv1a64:32262fbb72a93d29`. The oracle independently models integer ranges,
casts, IEEE conversions and classes, identity, coercions, and layouts. Its
small host floating-operation sample is a bounded target observation against
fixed expected bits, not a normative semantic source. Production type-checker
tests and LLVM conformance remain pending their owning phases.

| Requirement clause | Normative rule | Positive case | Negative case | Phase B evidence | Downstream evidence gate |
|--------------------|----------------|---------------|---------------|------------------|--------------------------|
| `REQ-008`: nominal user-defined identity | Each `struct` and `enum` uses declaration `DefId`; layout and structure do not determine identity | Distinct declaration used only at its own type | Structurally equal `Left` returned as `Right` | Type Identity section and nominal derivation | Phase C identity table; production resolver and type-checker tests |
| `REQ-009`: inference remains body-local | Per-body constraints use collected signatures and monomorphic locals | Local unsuffixed literal infers `i32` | Caller or another body cannot determine a signature | Local Inference section and examples | Production body-checking tests |
| `REQ-009`: public signatures are not inferred | Every public surface component is explicit, including `-> ()` | `pub fn done() -> ()` | `pub fn done()` | Public Type Surfaces section and grammar amendment | Parser and visibility/type-checker compile-fail tests |
| `REQ-016`: integer overflow and division | Complete integer table uses checked, profile-independent panic semantics | In-range `u8` addition | Overflow, zero division, `MIN / -1`, and invalid shifts | Integer and Boolean Operations section | Phase C boundary table and debug/release digest; native tests |
| `REQ-016`: casts | Closed checked `as` matrix and blanket rejection | In-range integer and scalar casts | Narrowing overflow, NaN-to-integer, surrogate-to-`char`, and unrelated types | Explicit Casts section | Phase C bounded cast rows; production legality and guarded LLVM lowering tests |
| `REQ-016`: floating point | Strict binary32/binary64, default environment, no fast-math | Signed zero, subnormal, infinity, and ordered finite cases | NaN ordering and implicit mixed-format operation | Floating-Point Operations section | Phase C fixed-bit cases; optimized native comparison |
| `REQ-016`: character representation | Unicode scalar domain and four-byte initial storage | U+0000 and U+10FFFF | Surrogates and U+110000 | Primitive table, casts, and target table | Phase C scalar boundaries; backend layout probe |
| `REQ-016`: string encoding | Nominal owned valid UTF-8 `String` with three-word initial storage | UTF-8 string literal in runtime context | Direct indexing and string-valued constant | Owned String and target sections | Ownership/library tests and backend layout probe |
| `RLM-0002`: no implicit backend layout | Every scoped primitive and aggregate has an initial Windows x64 boundary | Listed scalar, aggregate, zero-sized, and indirect cases | Target facts used as source identity or coercion | Initial Target Representation and Ownership Boundaries sections | Independent Windows x64 and LLVM review |

## Downstream Acceptance Gates

The following work remains required before implementation claims can expand:

* A language review traces every scoped requirement to a rule and case
* A Windows x64 and LLVM review checks layouts, calling classes, poison guards,
  floating flags, and aggregate lowering
* Production type checking records a type and inserted coercion for every typed
  HIR expression
* Native debug and optimized executions agree with semantic CFG behavior

None of the remaining gates weakens the source rules in this specification.
