<!-- markdownlint-disable-file -->
---
title: RLM-0002 Type-System Research
description: Evidence and decision analysis for the Realm primitive and type-system specification
author: GitHub Copilot
ms.date: 2026-08-25
ms.topic: reference
keywords:
  - realm
  - type system
  - semantics
  - ABI
  - LLVM
---

## Research Status

Complete. Repository and official platform evidence have been reviewed. The
recommendations remain proposals until the language owner answers the questions
near the end of this artifact.

## Scope and Questions

This research answers the decision questions owned by `RLM-0002` without
implementing a type checker or changing accepted repository sources.

* What primitive widths, value domains, literal defaults, arithmetic behavior,
  casts, identity rules, inference limits, coercions, and aggregate semantics
  must a normative `type-system.md` specify?
* Which rules are language semantics, which are target ABI or storage choices,
  and which belong to later ownership or generic work?
* What accepted text constrains each decision, and where do contradictions or
  owner approvals remain?
* What judgment notation and bounded validation can establish specification
  completeness during `RLM-0002` without implementing a type checker?

## Repository Evidence

### Controlling Scope

* docs/planning/implementation-roadmap.md assigns `RLM-0002` primitive widths,
  character and string encoding, literal defaults, overflow, casts, nominal
  identity, local inference, coercions, never, unit, arrays, tuples, slices,
  and public-signature rules. Acceptance requires normative rules, judgment
  examples for `REQ-008`, `REQ-009`, and `REQ-016`, and no implicit backend
  layout.
* docs/planning/requirements-and-assumptions.md requires signed and unsigned
  fixed-width integers, `bool`, a defined character type, `f32`, `f64`, unit,
  never, tuples, fixed arrays, slices, and owned strings. Additional numeric
  widths, including `f16`, are deferred until target and ABI semantics are
  specified.
* `REQ-008` requires nominal identity for user-defined types even when their
  structure is equal. `REQ-009` limits inference to function bodies and forbids
  inferred public signatures. `REQ-016` requires build-profile semantics for
  integer faults, casts, floating point, characters, and strings before code
  generation is accepted.
* `ASM-002` fixes the initial target to `x86_64-pc-windows-msvc`. The project
  explicitly does not promise a stable Realm-native ABI, C++ or Rust ABI
  interoperability, other targets, or source compatibility with README
  examples.

### Accepted RLM-0001 Constraints

The current RLM-0001 records show explicit owner acceptance after 21 checker
tests and two independent reviews. Relevant records are:

* .copilot-tracking/changes/2026-08-25/phase-0-rlm-0001-changes.md
* .copilot-tracking/details/2026-08-25/phase-0-rlm-0001-details.md
* .copilot-tracking/plans/2026-08-25/phase-0-rlm-0001-plan.instructions.md
* .copilot-tracking/plans/logs/2026-08-25/phase-0-rlm-0001-log.md
* .copilot-tracking/research/2026-08-25/phase-0-research.md
* .copilot-tracking/research/subagents/2026-08-25/rlm-0001-grammar-review.md
* .copilot-tracking/research/subagents/2026-08-25/rlm-0001-syntax-research.md
* .copilot-tracking/reviews/2026-08-25/phase-0-rlm-0001-plan-review.md

Their accepted constraints are:

* `()` is the unit type and expression shape; `!` is the never type syntax.
* `(T)` is `T`, `(T,)` is a one-element tuple, and larger tuples are ordered
  products.
* `[T; N]` is a fixed array, `[T]` is a borrowed slice, and owned vectors are
  nominal library types deferred to generic design.
* String literals are Unicode scalar sequences encoded as UTF-8. Owned strings
  are nominal library values, not primitives. Direct string indexing and
  bracket slicing are type errors.
* Character literals contain exactly one Unicode scalar after escape
  processing. Surrogates and values above U+10FFFF are invalid.
* Literal signs are unary operators. The lexer emits a numeric token followed
  by a separate identifier; RLM-0002 owns suffix recognition and defaults.
* Integer overflow, division or remainder by zero, signed minimum divided by
  negative one, negation overflow, and invalid shift counts use the same
  non-catchable, cleanup-preserving Realm panic in every build profile.
* Checked, wrapping, and saturating operations may be explicit later, but the
  ordinary operator matrix cannot change by optimization level.
* Slices are borrowed views. Shared versus exclusive mutability and loan
  conflicts remain assigned to `RLM-0005`.

### Architecture and Ownership Boundaries

* docs/planning/architecture.md says type checking is per body against
  collected signatures. It permits local literal and binding inference, while
  requiring explicit public parameter and return types. It requires every
  typed-HIR expression to record its type and any inserted coercion.
* docs/planning/project-structure.md assigns canonical target-neutral types,
  effects, substitutions, ownership categories, and target-neutral layout
  facts to `realm-types`; inference, unification, coercion, literal checking,
  and typed-HIR production to `realm-typeck`; and concrete primitive and
  aggregate layouts plus calling conventions to `realm-runtime-abi`.
* LLVM lowering consumes validated monomorphic CFG and a target ABI
  description. It cannot infer source types or repair semantic failures.
* `RISK-002` requires normative numeric semantics before implementation.
  `RISK-012` requires native behavior to agree with the semantic CFG at both
  `-O0` and optimized settings. `RISK-029` warns against leaking Windows layout
  details into target-neutral language IR.
* ADR-001, ADR-002, ADR-004, ADR-005, and ADR-013 remain `Proposed`, not
  accepted. RLM-0002 may rely on their planning direction only where the
  accepted roadmap and requirements independently impose the same constraint.

### README Contradictions

README.md remains non-normative. Its `f16` claim conflicts with the accepted
deferred-feature list; `string` is used but omitted from its primitive list;
`[bool, 5]`, `[i32]`, and `['d', 3]` conflict with the accepted array, slice,
and repetition grammar; and it leaves literal inference, tuple access, casts,
overflow, and aggregate representation undefined. The accepted grammar already
supersedes these spellings. RLM-0002 must not preserve them accidentally.

## Official Platform Evidence

The platform sources constrain lowering, not Realm value domains.

* [LLVM Language Reference](https://llvm.org/docs/LangRef.html) distinguishes
  source-language guarantees from IR behavior. Integer `add`, `sub`, and `mul`
  wrap unless overflow flags are attached; invalid `nsw` or `nuw` assumptions
  produce poison. Division by zero, signed minimum divided by negative one,
  and oversized shifts cannot be emitted unchecked when Realm requires a
  deterministic panic. Float-to-integer conversion also needs a source-level
  range and NaN check before an LLVM conversion if Realm chooses checked casts.
* [LLVM DataLayout](https://llvm.org/docs/LangRef.html#data-layout) is
  target-specific. LLVM distinguishes bit width, store size, allocation size,
  and ABI alignment. In particular, `i1` has one value bit but an eight-bit
  store and allocation size in the documented size examples. Realm therefore
  cannot equate LLVM `i1` SSA representation with its addressable `bool`
  storage or public calling representation.
* [LLVM floating-point types](https://llvm.org/docs/LangRef.html#floating-point-types)
  provide `half`, `float`, and `double`, but an available IR type does not prove
  a native Windows x64 ABI or instruction strategy for a Realm `f16` primitive.
* [LLVM constrained floating-point intrinsics](https://llvm.org/docs/LangRef.html#constrained-floating-point-intrinsics)
  exist for code that exposes rounding-mode or floating-exception behavior.
  Realm can use ordinary strict operations if it fixes the environment and
  does not expose dynamic rounding or traps; fast-math flags must remain
  forbidden unless a later opt-in semantic mode defines them.
* [Microsoft x64 ABI conventions](https://learn.microsoft.com/en-us/cpp/build/x64-software-conventions?view=msvc-170)
  specify 1, 2, 4, and 8-byte integer storage, 4-byte `float`, 8-byte `double`,
  8-byte pointers, natural scalar alignment, element-aligned arrays, and
  maximum-member-aligned aggregates with tail padding.
* [Microsoft x64 calling convention](https://learn.microsoft.com/en-us/cpp/build/x64-calling-convention?view=msvc-170)
  passes the first four integer or pointer arguments in `RCX`, `RDX`, `R8`, and
  `R9`, and floating arguments in positional `XMM0` through `XMM3`. Aggregates
  of 8, 16, 32, or 64 bits can be passed as same-sized integers; other
  aggregates are passed by reference. Arrays and strings are not passed as
  immediate values.
* The same Microsoft calling document sets the default `MXCSR` environment to
  masked exceptions, round-to-nearest, denormals-are-zero off, and
  flush-to-zero off. A callee that changes nonvolatile control bits must
  restore them.
* [Microsoft data type ranges](https://learn.microsoft.com/en-us/cpp/cpp/data-type-ranges?view=msvc-170)
  documents a one-byte C++ `bool` and one-byte implementation-dependent plain
  `char`. These are ABI facts for C/C++ interoperation, not reasons to make
  Realm `char` an eight-bit code unit or to admit noncanonical Boolean values.

## Semantic Decision Set

The following set is the minimum complete normative surface. Each recommendation
is a proposed owner decision, not an accepted language change.

### Decision 1: Integer Primitives and Value Domains

| Option | Pros | Cons | Complexity | ABI and backend implications |
|---|---|---|---|---|
| A. Fixed `i8`/`i16`/`i32`/`i64` and `u8`/`u16`/`u32`/`u64` only | Matches MVP text and README subset; target-independent source widths | No pointer-sized convenience type | Low | Direct LLVM `i8` through `i64`; Windows sizes and alignments are 1/1, 2/2, 4/4, and 8/8 bytes |
| B. Add `isize` and `usize` | Convenient for lengths and pointer arithmetic | Leaks target width into source semantics; creates cross-target compatibility work | Medium | Must map to target pointer width and enter mangling, constants, overloads, and C ABI rules |
| C. Add arbitrary-width integers | Matches LLVM capability | Greatly expands literal fitting, ABI, arithmetic runtime, and generic constraints | High | Widths beyond legal machine scalars need legalization or runtime support |

Recommendation: Select option A. Define signed range as
`-2^(N-1)..=2^(N-1)-1` and unsigned range as `0..=2^N-1`. Do not add
`isize`, `usize`, `i128`, `u128`, or `f16` in RLM-0002. Any integer primitive
may type an index or slice bound; the semantic index operation checks negativity
and magnitude without first converting to a narrower common type. Array lengths
are nonnegative constant values bounded by `u64` and by target allocation
limits.

Accepted constraints: The minimum viable language requires signed and unsigned
fixed-width integers. The deferred list explicitly includes additional widths
and `f16`. LLVM's arbitrary integer widths are an implementation capability,
not a Realm requirement.

### Decision 2: `f16`

| Option | Pros | Cons | Complexity | ABI and backend implications |
|---|---|---|---|---|
| A. Keep `f16` deferred | Follows accepted scope; avoids software and ABI ambiguity | README claim remains superseded | Low | No `half` values, suffix, calling rule, or runtime helper in MVP |
| B. Storage-only `f16` with arithmetic promoted to `f32` | Useful for data formats | Surprising promotion and cast rules; still needs ABI policy | Medium | Two-byte storage, conversion on every operation, indirect or custom argument classification |
| C. Full binary16 arithmetic primitive | Compact numerical type | Native support varies; requires exact operation, constant, ABI, and debugging behavior | High | LLVM `half` exists, but Windows x64 documentation does not define a standard scalar `half` calling class |

Recommendation: Select option A. The type name and `f16` suffix are unavailable,
not reserved semantic aliases. A later numerical task may add binary16 after a
target probe and ABI decision.

Accepted constraints: docs/planning/requirements-and-assumptions.md explicitly
defers `f16`; docs/specifications/grammar.md assigns the README claim to
RLM-0002 and target-layout review.

### Decision 3: `bool`, `char`, `unit`, and `never`

#### `bool`

| Option | Pros | Cons | Complexity | ABI and backend implications |
|---|---|---|---|---|
| A. Two-value domain, one-byte canonical storage, LLVM `i1` SSA | Clear validity invariant; interoperable storage size | Requires memory/SSA conversion | Low | Store `0` or `1` in `i8`; load then compare/truncate to `i1`; use explicit zero-extension attributes where the selected ABI requires them |
| B. One-bit packed storage everywhere | Dense arrays | Addressability and ABI become target-specific; ordinary pointers cannot select a bit | High | Requires bit packing, read-modify-write, and custom aggregate layout |
| C. Four-byte storage | Simple register promotion | Wastes arrays and disagrees with Microsoft C++ `bool` storage | Low | LLVM `i32` storage and integer ABI class |

Recommendation: Select option A. `bool` has exactly `false` and `true`. Any
noncanonical byte entering through unsafe code or FFI is invalid. `bool` is one
byte with alignment one in addressable storage and aggregates, while conditions
and SSA computations may use LLVM `i1`. Boolean arrays are byte arrays in the
MVP, not bitsets.

#### `char`

| Option | Pros | Cons | Complexity | ABI and backend implications |
|---|---|---|---|---|
| A. Unicode scalar value in four-byte storage | Matches accepted literals and UTF-8 policy; constant-time validation | Uses more space than UTF-8 code units | Low | LLVM `i32`; four-byte size/alignment; C FFI needs an explicit `u32` contract, not C `char` or `wchar_t` |
| B. UTF-8 code unit | One byte | Cannot represent one accepted character literal generally | Low | LLVM `i8`, but contradicts the scalar literal contract |
| C. UTF-16 code unit | Windows-adjacent | Surrogates conflict with the accepted scalar-value rule | Medium | LLVM `i16`; not equivalent to Microsoft `wchar_t` sequences or Realm characters |

Recommendation: Select option A. The value domain is U+0000 through U+D7FF and
U+E000 through U+10FFFF. Storage is an `i32`-shaped four-byte value with
alignment four; surrogate values are invalid. `char` is not an integer subtype.

#### Unit and never

| Option | Pros | Cons | Complexity | ABI and backend implications |
|---|---|---|---|---|
| A. Unit is a zero-sized singleton; never is an empty type | Conventional semantics; no runtime payload | Address identity for zero-sized places must be handled later | Low | Unit return lowers to LLVM `void`; unit arguments and fields consume no payload; never-returning paths end in `unreachable` after required cleanup |
| B. Unit occupies one byte | Every place has distinct storage more readily | Inflates tuples and arrays; unit byte has no semantic value | Low | LLVM `i8` placeholder; ABI slots become observable internally |
| C. Unit and never both lower to `void` without semantic types | Simplest IR | Loses typing distinctions and never coercion | Low | Violates typed-HIR requirement that every expression has a type |

Recommendation: Select option A. `()` has one value and zero target storage;
`!` has no values and no layout. A public function may explicitly return either.
Unit parameters are semantically present but omitted from the current internal
calling sequence; unit returns use `void`. Never parameters are uninhabited and
have no callable native entry from safe Realm. Address identity and borrowing of
zero-sized places remain RLM-0005 work.

Accepted constraints: The grammar fixes `()` and `!`. Architecture requires
typed HIR and validated CFG, so neither type may disappear before lowering.

### Decision 4: Owned String Boundary

| Option | Pros | Cons | Complexity | ABI and backend implications |
|---|---|---|---|---|
| A. String literals default to nominal owned `String` with `{ptr, len, capacity}` representation | Matches MVP owned-string goal and README local inference | Literal evaluation may allocate or copy; constants need restriction | Medium | On Windows x64 the 24-byte aggregate is passed and returned indirectly under the default aggregate rules |
| B. String literals have an implicit static borrowed-string type | No allocation for literals; const-friendly | Introduces hidden lifetime and coercion rules before ownership design | Medium | Fat reference is typically two words; aggregate calling classification must be explicit |
| C. Opaque runtime handle | Hides layout and permits runtime changes | Adds indirection and runtime dependency; risks implicit shared ownership | Medium | One pointer in ABI, but allocation, cloning, and thread behavior move into runtime |

Recommendation: Select option A, with a strict boundary. `String` is a nominal
prelude library type, not a primitive, but `STRING` expressions default to it
when no expected type applies. Its semantic invariant is owned, contiguous,
valid UTF-8 bytes with `0 <= len <= capacity`; indexing and bracket slicing
remain rejected. For the initial target layout, store an eight-byte data pointer,
an eight-byte byte length, and an eight-byte capacity in that order, with
alignment eight. Empty strings use the library's approved non-null aligned
sentinel. Allocation, moves, destruction, and a future borrowed string view
remain RLM-0005 and standard-library work.

String-valued `const` items should remain rejected in RLM-0002. This avoids
inventing immortal ownership or a hidden static-borrow type. A later ownership
task can add static string constants without changing literal UTF-8 semantics.

Accepted constraints: Owned strings are nominal library values, source and
literal contents are UTF-8, direct string indexing is a type error, implicit
shared ownership is a non-goal, and no stable Realm-native ABI is promised.

### Decision 5: Numeric Suffixes and Defaults

| Option | Pros | Cons | Complexity | ABI and backend implications |
|---|---|---|---|---|
| A. Contextual literals with exact suffixes; fallback `i32` and `f64` | Familiar; keeps literals locally inferred | Requires a parser rule for adjacent suffix tokens | Medium | Constants are parsed at arbitrary precision, then converted once to the selected LLVM type |
| B. No suffixes, defaults only | No grammar amendment | Large and narrow literals need verbose casts; request explicitly asks for suffix behavior | Low | Fewer literal forms but more cast checks |
| C. Infer the smallest fitting type | Accepts large literals conveniently | Small source edits can change overload and ABI identity; signedness is surprising | Medium | Type selection becomes value-dependent and destabilizes mangling |
| D. Default every integer to `i64` and float to `f64` | Simple on x64 | Conflicts with README's pervasive `i32` examples and inflates storage | Low | More truncation at narrow APIs; four-byte Windows `int` is no longer the common default |

Recommendation: Select option A. Exact suffixes are `i8`, `i16`, `i32`, `i64`,
`u8`, `u16`, `u32`, `u64`, `f32`, and `f64`. Integer-shaped syntax may use an
integer or floating suffix; decimal-point or exponent syntax may use only
`f32` or `f64`. Context may select a compatible numeric primitive before the
fallback. An unconstrained integer literal defaults to `i32`; an unconstrained
floating literal defaults to `f64`. A fallback literal outside that type is a
diagnostic that asks for a suffix or expected type, not an automatic widening.

Literal parsing uses unbounded mathematical integers and correctly rounded
decimal conversion. Unary sign is applied before the final representability
check, so `-2147483648` may inhabit `i32` while positive `2147483648` may not.
Contextual literal typing is not an implicit numeric coercion.

The accepted token contract creates a blocking grammar detail: the lexer emits
an immediately adjacent suffix as a separate `IDENTIFIER`, while the grammar's
`literal` production consumes only one numeric token and otherwise permits
trivia between tokens. The recommended repair is a narrow parser production
that combines a numeric token and an immediately contiguous exact suffix token;
whitespace forbids suffix interpretation. Owner approval must authorize this
amendment to docs/specifications/grammar.md during RLM-0002.

### Decision 6: Ordinary Integer Arithmetic

| Option | Pros | Cons | Complexity | ABI and backend implications |
|---|---|---|---|---|
| A. Checked in every build | Matches accepted panic category; deterministic | Adds branches or overflow intrinsics | Medium | Use `llvm.*.with.overflow` or equivalent checks, then branch to cleanup-preserving panic before consuming invalid LLVM operations |
| B. Checked in debug, wrapping when optimized | Conventional in some toolchains | Explicitly conflicts with accepted build-independent semantics | Low | Optimizer profile changes observable behavior |
| C. Always wrapping | Fast and simple | Hides defects and conflicts with accepted arithmetic panic | Low | Plain LLVM integer operations suffice |
| D. Undefined on overflow | Maximum optimizer freedom | Violates Realm safety and determinism goals | Low | `nsw`/`nuw` poison can become undefined behavior at side effects |

Recommendation: Select option A. Define this complete matrix:

| Operation | Operand typing | Result and failure rule |
|---|---|---|
| Unary `+` | Any integer `T` | Identity in `T` |
| Unary `-` | Signed integer `T` | Mathematical negation; panic on `MIN_T` |
| `+`, `-`, `*` | Same integer type `T` | Mathematical result if representable in `T`; otherwise panic |
| `/` | Same integer type `T` | Quotient truncated toward zero for signed values; panic on zero and on `MIN_T / -1` |
| `%` | Same integer type `T` | Remainder with zero or dividend's sign and `a = (a / b) * b + a % b`; same panic cases as division |
| `<<` | Integer left operand `T`; any integer count | Panic for negative count, count at least bit width, or unrepresentable mathematical `lhs * 2^count`; otherwise result in `T` |
| `>>` | Integer left operand `T`; any integer count | Panic for negative count or count at least bit width; logical right shift for unsigned, arithmetic sign-extending right shift for signed |
| `&`, `|`, `^` | Same integer type `T` | Bitwise result in `T` with no arithmetic overflow |
| Unary `!` | Integer `T` or `bool` | Bitwise complement for integers; logical negation for `bool` |
| `&`, `|`, `^` on `bool` | Two `bool` operands | Eager Boolean operations; unlike `&&` and `||`, both operands evaluate |

Operands evaluate left to right as already accepted. Compound assignment
evaluates its place once and uses the corresponding checked operation. Ordinary
integer operators never perform implicit widening or signedness conversion.
Compile-time failure in a required constant is a diagnostic; the same operation
at runtime invokes the accepted non-catchable panic. Debug and optimized
behavior is identical.

Integer equality and ordering require identical operand types, use that type's
signed or unsigned interpretation, and produce `bool`. Boolean `&&` and `||`
accept only `bool`, evaluate the right operand conditionally, and produce
`bool`. Boolean values support equality but not ordering. Characters support
equality and Unicode-scalar-value ordering. Unit equality is always true and
unit inequality is always false; unit has no ordering. Assignment and every
compound assignment produce `()` after updating a mutable compatible place, so
assignment chaining is rejected by ordinary type checking.

LLVM implications: check division and remainder preconditions before emitting
`sdiv`, `udiv`, `srem`, or `urem`; check shift counts before LLVM shifts; do not
model a source panic as poison or undefined behavior. After a successful guard,
the backend may attach provable flags locally.

### Decision 7: IEEE-754 Floating-Point Semantics

| Option | Pros | Cons | Complexity | ABI and backend implications |
|---|---|---|---|---|
| A. Strict binary32/binary64 in the default environment | Predictable across optimization levels; matches LLVM and Windows defaults | Restricts reassociation and contraction | Medium | Ordinary LLVM FP operations without fast-math flags; preserve default `MXCSR` around Realm and foreign calls |
| B. Fast math in optimized builds | Higher optimization potential | NaN, infinity, signed-zero, and reassociation behavior changes by profile | Low | Fast-math flags permit otherwise unsafe transformations and poison assumptions |
| C. Dynamic rounding and observable FP exceptions | Maximum numerical control | Expands language state, optimizer constraints, unwind, and FFI contracts | High | Requires constrained FP intrinsics throughout affected functions |

Recommendation: Select option A. `f32` is IEEE-754 binary32 and `f64` is
IEEE-754 binary64. Operations use round-to-nearest, ties-to-even; exceptions are
masked; subnormals are preserved; overflow produces signed infinity; gradual
underflow applies; and division by floating zero follows IEEE-754 instead of
the integer panic rule. NaNs, infinities, and signed zeros are values.

`+`, `-`, `*`, and `/` use the corresponding format. `%` uses truncated-quotient
remainder equivalent to `fmod`/LLVM `frem`, not IEEE `remainder`. Unary `-`
changes the sign, including signed zero and NaN sign where the target operation
preserves it. `==` treats positive and negative zero as equal and is false for
NaN; `!=` is true for NaN; ordered comparisons are false when either operand is
NaN. NaN payload and NaN sign are unspecified, but a NaN result remains NaN.

Floating comparisons require identical operand types and produce `bool`; there
is no implicit `f32`/`f64` promotion. String, tuple, and array equality remains
unavailable until the library and generic-constraint task define its element
requirements.

Do not reassociate, assume finite values, discard signed zeros, approximate
division, or contract multiply-add in ordinary code. A fused operation may be
added only as an explicit library or intrinsic operation. Compile-time
evaluation must match target-format rounding and comparison, not host Rust
accidentally. The runtime owns preservation of the documented default `MXCSR`
control bits across Realm calls. A later explicitly unsafe FFI contract must
state how foreign code that changes the environment is isolated.

Accepted constraints: `REQ-016` demands both debug and optimized semantics;
`RISK-012` requires native behavior to match semantic CFG; Microsoft documents
the recommended default environment; LLVM says fast-math flags enable otherwise
unsafe transformations and constrained intrinsics are required for non-default
rounding or observed exceptions.

### Decision 8: Explicit Cast Syntax and Matrix

| Option | Pros | Cons | Complexity | ABI and backend implications |
|---|---|---|---|---|
| A. `expression as Type` with checked value-changing casts | Uses an existing keyword; visible and safe | Requires an accepted grammar amendment; needs runtime checks | Medium | Maps to extension, truncation, FP conversion, or validation plus a panic edge |
| B. Constructor-style `Type(expression)` | Uses call grammar | Conflates nominal construction, conversion, and calls | Medium | Name resolution must distinguish type calls from functions before type checking |
| C. Intrinsic `cast<Type>(expression)` | Semantically explicit | Depends on generic syntax blocked on RLM-0004 | Medium | Adds an intrinsic and generic parsing special case |
| D. Wrapping C-like casts | Familiar to C users | Silent narrowing conflicts with the checked-safety direction | Low | Direct LLVM truncation can hide data loss |

Recommendation: Select option A. Add `cast_expression ::= expression "as"
type` at a documented precedence between unary/postfix expressions and
multiplicative operators, or use an equivalent nonrecursive Pratt rule. Casts
do not chain without explicit grouping if the grammar would otherwise be
ambiguous.

The accepted grammar reserves `as` but currently uses it only for import aliases
and has no cast expression. RLM-0002 cannot claim a usable cast matrix without
owner approval to amend docs/specifications/grammar.md.

Recommended explicit cast matrix:

| Source | Target | Semantics | Failure |
|---|---|---|---|
| Integer `S` | Integer `T` | Preserve the mathematical value when representable, regardless of width or signedness | Panic if not representable; constant diagnostic in a required constant |
| Integer | `f32` or `f64` | Round to nearest, ties to even, in the target format | No failure for MVP integer widths |
| `f32` or `f64` | Integer `T` | Truncate toward zero, then require the mathematical integer to be representable | Panic for NaN, infinity, or out-of-range result; constant diagnostic in a required constant |
| `f32` | `f64` | Exact widening | None |
| `f64` | `f32` | IEEE target-format rounding, including infinity on overflow and signed zero on underflow | None |
| `char` | Integer `T` | Convert the Unicode scalar numeric value if representable | Panic only when `T` is too narrow or signed range is insufficient |
| Integer | `char` | Require a Unicode scalar value | Panic for negative, surrogate, or above U+10FFFF |
| Any type | Same type | Identity; accepted but SHOULD be diagnosed as redundant | None |
| `bool` | Numeric or `char` | Not permitted | Static type error |
| Numeric or `char` | `bool` | Not permitted | Static type error |
| References, strings, arrays, tuples, unit, nominal types | Unrelated type | Not permitted in safe RLM-0002 casts | Static type error |
| Never | Any type | Uses implicit never coercion, not `as` | None |

Checked cast panic behavior is identical in debug and optimized builds. Later
APIs may expose named wrapping, saturating, bit-pattern, pointer, or unchecked
casts. Those are not alternate meanings of `as`.

LLVM implications: `trunc`, `zext`, `sext`, `fptrunc`, `fpext`, `fptosi`,
`fptoui`, `sitofp`, and `uitofp` are lowering tools, not the source contract.
Float-to-integer checks must precede LLVM conversion because an out-of-range
conversion produces poison. Pointer casts and `bitcast` remain unavailable to
safe Realm and belong to the RLM-0005 unsafe boundary.

### Decision 9: Nominal and Structural Identity

| Option | Pros | Cons | Complexity | ABI and backend implications |
|---|---|---|---|---|
| A. Each `struct` and `enum` declaration has nominal identity; tuples, arrays, references, and function types are structural | Satisfies `REQ-008`; conventional and composable | Package identity must later become stable | Low | Distinct nominal types may share a layout but must retain distinct type IDs and mangled identities |
| B. All types are structural | Fewer explicit conversions | Directly violates `REQ-008`; accidental compatibility across modules | Medium | Layout becomes semantic identity and blocks independent evolution |
| C. Nominalize aliases and all aggregate occurrences | Maximum separation | No alias syntax exists; tuple and array usability suffers | High | Proliferates wrapper layouts and conversion operations |

Recommendation: Select option A. A user-defined nominal type is identified by
its declaration `DefId`, not its name spelling or field/variant structure. Two
declarations remain distinct even when every field, variant, visibility, and
layout is equal. Renaming a field does not create structural compatibility.

Tuple identity is the ordered list of element types. Array identity is element
type plus constant length. Reference identity is mutability plus referent type.
Function type identity is parameter sequence, return type, and declared
`throws` effect; `async` identity remains RLM-0007 work. Primitive types have
language-defined intrinsic identity. The nominal identity of `String` is its
prelude declaration, not its three-word representation.

The stable cross-package component of `DefId` cannot be finalized until package
content identity is specified. RLM-0002 should state the semantic rule and use
session-local declaration identity in examples. Generic substitutions and
constraint identity remain RLM-0004 work.

Accepted constraints: `REQ-008` is direct; HIR and type architecture require
stable semantic IDs; no stable Realm-native binary ABI is promised.

### Decision 10: Local Inference and Public Signatures

| Option | Pros | Cons | Complexity | ABI and backend implications |
|---|---|---|---|---|
| A. Per-body bidirectional inference for locals and literals only | Satisfies `REQ-009`; signatures are collectable before bodies | Some local annotations remain necessary | Medium | Backend sees fully typed HIR; no ABI depends on body inference |
| B. Infer private function returns across bodies | Less annotation | Introduces dependency-order, recursion, and diagnostic instability | High | Signature and mangling can depend on body analysis order |
| C. Infer public signatures | Concise APIs | Directly violates `REQ-009`; changing an implementation silently changes API/ABI | High | Exported identity and downstream checking become unstable |
| D. No inference | Simplest checker | Contradicts the planned local inference and README examples | Low | No backend issue, but poor language ergonomics |

Recommendation: Select option A with these boundaries:

* Function parameters are always explicitly typed, as required by the grammar.
* An omitted return type means `()` for a private function; it is not inferred
  from body tails.
* A `pub fn` must write an explicit return type, including `-> ()` for unit.
  Its parameter types and declared effects are also explicit.
* Structure fields, enum payloads, function-type components, and constant items
  are explicitly typed.
* A local `let` or `var` may omit its annotation because the grammar requires an
  initializer. Inference is confined to that function body.
* Expected types flow into literals, array elements, tuple elements, returns,
  assignments, and call arguments. Equality constraints flow back within the
  same body. Only the explicit coercion set below may change a type.
* Local bindings are monomorphic. No let-generalization, cross-body inference,
  or inference from downstream callers occurs.
* Empty array literals require an expected array type. Other unresolved type
  variables are diagnostics after numeric defaults are applied.
* Recursive and mutually recursive calls use collected explicit signatures,
  never body-derived return inference.
* Generic argument inference, if any, remains RLM-0004 work and cannot broaden
  these public-signature rules.

Accepted constraints: `REQ-009` forbids inferred public signatures;
architecture says bodies are checked against collected signatures and permits
local literal and binding inference; grammar gives every parameter a type and
requires every binding initializer.

### Decision 11: Implicit Coercions and References

| Option | Pros | Cons | Complexity | ABI and backend implications |
|---|---|---|---|---|
| A. Closed coercion set: never, mutability weakening, array-reference to slice-reference, and function item to pointer | Predictable and reviewable; no hidden numeric loss | Requires explicit casts more often | Medium | Typed HIR records every coercion; fat-reference construction is explicit in lowering |
| B. Add implicit numeric widening | Convenient arithmetic | Mixed signedness and overload behavior expand rapidly; ABI may change with context | Medium | Inserts extension operations throughout typed HIR |
| C. General subtyping and user conversions | Flexible | Contradicts deferred user-defined implicit conversion and broadens solver scope | High | Requires dynamic or static conversion lookup and coherence |

Recommendation: Select option A. The complete RLM-0002 coercion set is:

* Identity conversion
* Never coercion from `!` to any expected type on a control-flow path that does
  not continue
* Shared reborrow from `&mut T` to `&T`
* Exclusive array-to-slice coercion from `&mut [T; N]` to `&mut [T]`
* Shared array-to-slice coercion from `&[T; N]` to `&[T]`, including the
  composition `&mut [T; N]` to `&[T]`
* Function-item to matching noncapturing `fn(...) -> R` pointer coercion

There are no implicit integer widenings, signedness changes, integer/float
conversions, Boolean conversions, nominal-wrapper conversions, string
allocations, dereference chains, or ownership clones. Contextual numeric literal
typing is not a coercion because the literal has no prior concrete type.

RLM-0002 should fix reference representation boundaries without deciding loans:
`&T` and `&mut T` are non-null, aligned, dereferenceable one-word references for
sized `T` and occupy eight bytes with alignment eight on the initial target.
Their validity duration, exclusivity, reborrow lifetime, variance, move/copy
eligibility, and cleanup interaction remain RLM-0005 work. A function pointer is
one eight-byte code pointer; closures are outside this task.

Accepted constraints: Shared and exclusive reference syntax is accepted;
user-defined implicit conversions are deferred; typed HIR must record every
coercion; RLM-0005 owns detailed loan behavior.

### Decision 12: Arrays and Tuples

| Option | Pros | Cons | Complexity | ABI and backend implications |
|---|---|---|---|---|
| A. Structural arrays/tuples with source-order target layout | Predictable identity and field offsets; easy validation | Exposes current internal layout to debugging, though not as stable ABI | Low | LLVM arrays and structs; target DataLayout supplies padding/alignment |
| B. Permit field reordering for tuples | Potential packing improvements | Source projection and debugger layout become unstable | Medium | Requires remapping every projection and ABI signature |
| C. Make arrays nominal and tuples library types | Uniform nominality | Contradicts accepted grammar and ordinary aggregate use | High | Requires generated nominal declarations and constructors |

Recommendation: Select option A.

An array `[T; N]` has exactly `N` elements of the same `T`; `N` is part of type
identity and is a nonnegative constant representable as `u64`. `N = 0` is valid.
Array values use contiguous element allocation stride and the element's
alignment. The target size is `N * alloc_size(T)`, checked for `u64` and target
address-space overflow during compilation. Arrays with zero-sized elements have
zero data size while retaining their semantic length. Array equality and
ordering remain unavailable until generic constraints define the required
element operations.

An array-list expression checks every element against one expected `T`; without
an expected type, it infers one `T` from all elements using equality plus the
closed coercion set. A repeat expression checks its value against `T` and its
count as a constant array length. An empty array expression always needs an
expected `[T; 0]` type.

A tuple `(T0, ..., Tn)` is a structural ordered product. Elements retain source
order and use zero-based integer projections. `()` is unit, `(T)` is `T`, and
`(T,)` is a one-element tuple as already accepted. The target layout places
each element at the next offset satisfying its alignment and rounds total size
to maximum element alignment. Empty/unit and all-zero-sized tuples occupy zero
bytes with alignment one. No field reordering occurs in optimized builds.

Windows x64 calling classification is a target ABI concern: aggregate values of
1, 2, 4, or 8 bytes may use an integer class under the default ABI; other
aggregate sizes are indirect. RLM-0002 should record this mapping but must not
make calling class part of tuple or array type identity. Explicit C-layout
aggregates remain FFI work.

Accepted constraints: Array and tuple grammar is fixed; `RISK-029` prohibits
target leakage into semantic types; Microsoft and LLVM both make aggregate
size/alignment dependent on target layout.

### Decision 13: Slice Type Meaning and Representation

| Option | Pros | Cons | Complexity | ABI and backend implications |
|---|---|---|---|---|
| A. `[T]` is unsized; values are `&[T]` or `&mut [T]` | Mutability and borrowing are explicit; clean array coercions | Requires correcting accepted prose/example that uses `[i32]` directly | Medium | Fat reference `{ptr, u64 len}` is 16 bytes/alignment 8 and indirect under default Windows aggregate calling rules |
| B. `[T]` is a shared by-value borrowed view; `&mut [T]` is exclusive | Preserves the accepted `[i32]` parameter example | Hides the shared borrow while spelling exclusive borrow; `&[T]` becomes ambiguous or redundant | Medium | Shared slice is a 16-byte fat value; exclusive form needs a separate type rule |
| C. `[T]` is an owned dynamic array | Matches old README list spelling | Directly contradicts accepted grammar and owned-vector deferral | High | Requires allocation, capacity, moves, destruction, and generic ownership now |

Recommendation: Select option A and obtain explicit owner approval to correct
the accepted grammar prose and recovery example. The type `[T]` denotes an
unsized contiguous sequence and cannot appear by value in a binding, parameter,
return, field, tuple, or array. A shared slice value has type `&[T]`; an
exclusive mutable slice has type `&mut [T]`. Slicing constructs one of those
reference types according to the accepted receiver rule.

Both slice references use a non-null aligned data pointer plus `u64` element
length. The pointer may use an approved aligned dangling sentinel for an empty
slice and must never be dereferenced when length is zero. Length counts elements,
not bytes. The representation is 16 bytes with alignment eight on Windows x64.
Element validity, bounds normalization, and panic behavior are already accepted;
lifetime and alias validity remain RLM-0005 work.

Option A resolves a genuine accepted-text conflict. docs/specifications/grammar.md
says `[T]` is a borrowed slice, uses `fn recover(input: [i32])`, also admits
`&[T]`, and says exclusive slicing may produce an exclusive mutable slice. It
does not say whether `[T]` is unsized, a shared fat value, or the referent of a
fat reference. RLM-0002 must not leave all three interpretations possible.

### Decision 14: Constants and Public Type Surfaces

| Option | Pros | Cons | Complexity | ABI and backend implications |
|---|---|---|---|---|
| A. Explicitly typed constants with a bounded structural evaluator | Enough for array lengths and scalar/aggregate constants; deterministic | Function calls and owned strings are initially excluded | Medium | Constant lowering uses target-independent values, then target layout; failures are diagnostics, never poison globals |
| B. Permit arbitrary pure function calls | Expressive | Purity, recursion, termination, effects, and ownership are unresolved | High | Requires an interpreter over functions before CFG/runtime semantics stabilize |
| C. Literals only | Easiest implementation | Array lengths and useful aggregates become awkward | Low | Minimal constant folding but inadequate for ordinary declarations |

Recommendation: Select option A. Every `const`, public or private, retains the
grammar-required explicit type. A public constant's type is part of its API and
cannot be inferred. Its initializer may contain:

* Boolean, character, integer, and floating literals
* Unit, tuple, fixed-array, and array-repeat construction
* References to previously resolved constants without initialization cycles
* Parentheses and the ordinary unary, binary, comparison, and explicit cast
  operations defined by RLM-0002

Calls, loops, mutable bindings, assignment, `throw`, `await`, owned strings,
references, slices, structures, enums, and unsafe operations are not
constant-evaluable in RLM-0002. Conditional and match evaluation should remain
deferred until a later constant-evaluation expansion and ADT exhaustiveness
rules exist. A rejected construct is a constant-evaluation diagnostic, not
proof that the runtime expression is invalid.

Constant arithmetic uses the same value semantics as runtime arithmetic. A
panic condition becomes a compile-time diagnostic because a required constant
cannot be published with a latent failure. Floating constants are rounded in
the declared target format. Array lengths additionally require a nonnegative
integer value representable as `u64`, target allocation-size validation, and no
dependency cycle.

Public signatures include all `pub fn`, `pub const`, public structure fields,
public enum payloads, and public function-typed values. Every component is
explicitly typed and contains no unresolved inference variable. Whether a type
is accessible enough for a public API is a resolver/visibility check, not type
inference.

Accepted constraints: Constant items already require a type annotation;
`REQ-009` forbids inferred public signatures; `REQ-031` permits only constants
and expressions proven pure at top level; arbitrary top-level effects are
rejected.

### Recommended Initial Target Representation

This table prevents backend layout from remaining implicit while keeping
calling classification outside semantic identity.

| Realm type | Value domain | Windows x64 storage | LLVM value boundary | Default Windows x64 call boundary |
|---|---|---|---|---|
| `i8`/`u8` | Fixed 8-bit signed/unsigned | 1 byte, align 1 | `i8` | Integer class, right-justified |
| `i16`/`u16` | Fixed 16-bit signed/unsigned | 2 bytes, align 2 | `i16` | Integer class, right-justified |
| `i32`/`u32` | Fixed 32-bit signed/unsigned | 4 bytes, align 4 | `i32` | Integer class, right-justified |
| `i64`/`u64` | Fixed 64-bit signed/unsigned | 8 bytes, align 8 | `i64` | Integer class |
| `f32` | IEEE binary32 | 4 bytes, align 4 | `float` | Positional XMM register or stack |
| `f64` | IEEE binary64 | 8 bytes, align 8 | `double` | Positional XMM register or stack |
| `bool` | `false`, `true` | Canonical `i8`, 1 byte, align 1 | `i1` in SSA; `i8` in memory | One-byte integer class with explicit canonicalization/extension contract |
| `char` | Unicode scalar | `i32`, 4 bytes, align 4 | `i32` | Four-byte integer class |
| `()` | Singleton | 0 bytes, align 1 | No payload/`void` return | Omitted payload |
| `!` | Empty | No layout | `unreachable` control flow | No callable value |
| `String` | Owned valid UTF-8 | `{ptr, u64, u64}`, 24 bytes, align 8 | Backend struct or scalarized internals | Indirect argument and return |
| `&T`/`&mut T` for sized `T` | Valid reference capability | 8-byte pointer, align 8 | `ptr` | Pointer class |
| `&[T]`/`&mut [T]` | Reference plus element count | `{ptr, u64}`, 16 bytes, align 8 | Backend struct or two internal scalars | Indirect aggregate under default ABI |
| `[T; N]` | Homogeneous structural product | Contiguous `N * stride(T)` | LLVM array when representable | Size-dependent aggregate class |
| Tuple | Ordered structural product | Source-order padded aggregate | LLVM struct when representable | Size-dependent aggregate class |
| `fn(P...) -> R` | Noncapturing code address | 8-byte code pointer | `ptr` plus semantic signature | Pointer class; call uses lowered parameter/result ABI |

The table describes the first internal target contract, not a stable external
Realm ABI. Explicit C ABI exposure must use separately approved ABI-safe types
and layouts under `REQ-049`.

### Language, ABI, and Downstream Boundary

| Owned by normative type-system semantics | Owned by target ABI/backend | Explicitly downstream |
|---|---|---|
| Primitive names, domains, operator typing, literal fitting/defaults, panic conditions, IEEE results, cast legality/results, nominal identity, inference, coercions, aggregate identity, constant eligibility | Byte size, alignment, padding, scalar register class, indirect aggregate passing, LLVM type selection, overflow-check lowering, panic edge lowering, verification before/after optimization | Generic parameters/constraints/substitution (`RLM-0004`); moves, copy eligibility, loans, lifetimes, drops, zero-sized-place identity, unsafe pointer casts (`RLM-0005`); exception ABI (`RLM-0006`); async type identity (`RLM-0007`); sendability (`RLM-0008`); explicit C layout and FFI (`REQ-049`) |

The type-system document should state representation invariants and link to the
initial target table, but it should not put Windows register classes into
`TypeId` equality or source-language overload resolution.

## Contradictions and Owner Approvals

### Blocking Accepted-Text Conflicts

1. Numeric suffixes are assigned to RLM-0002, but the lexer emits an adjacent
   suffix as a separate identifier, the grammar consumes only one numeric token
   as a literal, and grammar notation otherwise permits trivia between any two
   tokens. Owner approval is required for a contiguous-token suffix production
   or for a no-suffix decision.
2. `REQ-016` and RLM-0002 require cast semantics, and `as` is a keyword, but the
   accepted expression grammar has no cast production. Owner approval is
   required to add `expression as Type` or choose a different existing grammar
   shape.
3. docs/specifications/grammar.md simultaneously calls `[T]` a borrowed slice,
   uses `[i32]` directly as a parameter, admits `&[T]`, and refers to shared and
   exclusive slice results. It does not determine whether `[T]` is unsized, a
   shared fat value, or a reference referent. Owner approval is required before
   normative slice typing and layout can be complete.
4. docs/planning/architecture.md requires explicit parameter and return types
   in public signatures, while the grammar permits a missing return type for
   every function. Owner approval is required for the recommendation that
   private omission means unit and `pub fn` must spell `-> ()`.
5. Accepted planning calls strings nominal owned library values but requires no
   backend layout to remain implicit. Owner approval is required for the
   proposed three-word `{ptr, len, capacity}` initial representation and the
   rejection of string-valued constants in RLM-0002.

### Semantic Choices Not Settled by Accepted Text

* Whether ordinary left shift checks discarded/sign-changing bits or only the
  shift count. The recommendation treats loss as overflow and panics.
* Whether `!` and `& | ^` operate on integers as well as Boolean values. The
  recommendation follows a typed dual meaning because no unary `~` exists.
* Whether float `%` is available and, if so, whether it means `fmod`/LLVM
  `frem` or IEEE `remainder`. The recommendation selects `fmod` semantics.
* Exact NaN payload/sign guarantees, fused contraction, subnormal handling,
  floating-environment exposure, and optimizer flags. The recommendation
  preserves strict default-environment behavior while leaving NaN payload/sign
  unspecified.
* Whether checked explicit narrowing is the sole meaning of `as`. The
  recommendation rejects silent wrapping and reserves named wrapping or
  bit-pattern operations for later APIs.
* Whether unit and all-zero-sized aggregates occupy zero bytes, and how their
  places are addressed. The recommendation fixes zero semantic storage but
  delegates place identity to RLM-0005.
* How nominal identity names a package across sessions. RLM-0002 can specify
  declaration identity, but stable package content identity remains package
  design work.
* Whether zero-length arrays and arrays of zero-sized elements are permitted.
  The recommendation permits both and preserves logical length independently
  of byte size.
* The permitted constant-expression subset. The grammar deliberately parses
  every expression as a candidate, but no accepted document defines semantic
  constant eligibility.

### README Disposition

No README conflict needs to block RLM-0002. The README is non-normative, source
compatibility is a non-goal, and RLM-0001 is accepted. RLM-0002 should record:

* `f16` remains deferred
* `string` is replaced by the nominal prelude spelling `String` in normative
  type examples
* `[bool, 5]` is `[bool; 5]`
* `[i32]` does not mean an owned list
* `['d', 3]` is `['d'; 3]` with type `[char; 3]`
* Unsuffixed `10` defaults to `i32`, `3.141` defaults to `f64`, and each may be
  contextually typed before fallback

## Validation Proposal

RLM-0002 can produce strong evidence without implementing name resolution,
unification, a general type checker, or production LLVM lowering.

### Judgment Notation

Use a small declarative notation in the normative specification:

```text
Γ ⊢ e ⇒ T ⇝ e'                 expression e synthesizes T and elaborates to e'
Γ ⊢ e ⇐ T ⇝ e'                 expression e checks against expected T
Σ ⊢ T ≡ U                      T and U have semantic type identity
T ↝ U                          one permitted implicit coercion exists
v ∈ T                          mathematical value v inhabits T
op_T(v1, ..., vn) ⇓ v          typed operation returns v
op_T(v1, ..., vn) ⇑ panic      typed operation invokes Realm panic
cast(S, T, v) ⇓ v'             explicit cast succeeds
cast(S, T, v) ⇑ panic          explicit cast fails at runtime
Γ ⊢const e ⇓ v : T             constant expression evaluates successfully
Γ ⊢const e ⇑ diagnostic        required constant is invalid
layout_target(T) = (s, a, r)   target size, alignment, and representation
abi_target(T, position) = c    target calling classification
```

`layout` and `abi` judgments must never appear as premises for source-level
type identity or coercion. Ownership judgments such as move, loan, region, and
drop are deliberately absent and enter in RLM-0005.

Representative derivations should include:

```text
Γ(x) = i32
----------------------------- synth-local
Γ ⊢ x + 1 ⇒ i32 ⇝ add.checked.i32(x, 1)

Γ ⊢ return value ⇒ ! ⇝ terminate(value)
----------------------------------------- never-coerce
Γ ⊢ return value ⇐ T ⇝ terminate(value)

Γ ⊢ a ⇒ &mut [T; N]
----------------------------------------- array-slice-shared
Γ ⊢ a ⇐ &[T] ⇝ slice.shared(a.data, N)

Σ(A) = DefId(pkg, module, 1)    Σ(B) = DefId(pkg, module, 2)
---------------------------------------------------------------- nominal-distinct
Σ ⊬ A ≡ B
```

### Normative Tables

The future type-system specification should contain machine-reviewable tables
for:

* Every primitive's name, category, width, value domain, default, suffix,
  storage size/alignment, LLVM value shape, and Windows call class
* Every unary and binary operator by operand/result type and failure condition
* Every explicit source/target cast pair, including rejected pairs
* Every implicit coercion, with no catch-all row
* Structural identity for tuples, arrays, references, slices, and functions
* Constant-eligible and constant-ineligible expression categories
* Public API surfaces that require explicit types
* Aggregate and reference layout examples, including zero-sized cases

### Disposable Table Oracle

If RLM-0002 adds executable evidence, place a standalone, disposable oracle
under a new spike directory and keep it outside the production workspace. It
should model only closed tables and mathematical operations, not parse Realm or
infer types.

Bound the oracle to these cases:

1. For each of eight integer types, test `MIN-1`, `MIN`, `MIN+1`, `-1`, `0`,
   `1`, `MAX-1`, `MAX`, and `MAX+1` where mathematically meaningful.
2. For `+`, `-`, `*`, unary `-`, `/`, and `%`, test ordinary values plus every
   overflow, zero-divisor, and `MIN / -1` boundary.
3. For shifts, cross the counts `-1`, `0`, `1`, `width-1`, `width`, and
   `width+1` with `0`, `1`, `MAX`, signed `MIN`, and `-1` where applicable.
4. For every integer-to-integer cast class, test one in-range value and both
   nearest out-of-range boundaries. Cover signed-to-unsigned and
   unsigned-to-signed explicitly.
5. For float-to-integer casts, cover positive and negative zero, fractions on
   both sides of zero, exact bounds, just-outside bounds, infinity, and NaN.
6. For `f32` and `f64`, use fixed bit patterns for positive/negative zero,
   minimum/maximum subnormal, minimum normal, maximum finite, both infinities,
   and representative quiet NaNs. Check comparisons, division by zero,
   `frem`, widening, and narrowing.
7. For characters, check U+0000, U+D7FF, both surrogate boundaries, U+E000,
   U+10FFFF, and U+110000 through the cast matrix.
8. For layout, assert the representation table plus arrays and tuples that
   force internal and tail padding. Include `[(); 0]`, `[(); 4]`, `[u8; 0]`,
   `(u8, u64, u16)`, `String`, and a slice reference.
9. For nominal identity and coercions, consume a static case table of at least
   12 accepted and 12 rejected judgments. The oracle compares expected table
   rows; it does not resolve source names or solve inference variables.
10. Run the same semantic case inventory under debug and optimized compilation
    of the oracle. The expected rows must be byte-for-byte identical.

Use a host-independent wide integer model for arithmetic boundaries and
explicit IEEE bit patterns for floats. Do not treat host Rust overflow mode or
host casts as the oracle. A small dependency for arbitrary-precision integer
parsing is acceptable only if pinned and reviewed; `i128` is sufficient for
testing the proposed eight integer domains but not for accepting arbitrary
source literal text.

### Independent Review

Acceptance should include two read-only reviews:

* One language review traces every `REQ-008`, `REQ-009`, and `REQ-016` clause to
  a normative rule, positive example, negative example, and table row.
* One backend review traces every representation to Microsoft x64 size,
  alignment, and calling constraints and traces every dangerous LLVM operation
  to a guard or semantics-preserving lowering rule.

The backend review should explicitly reject plans that rely on LLVM poison,
undefined behavior, fast-math flags, host-language casts, or `i1` memory as a
substitute for Realm semantics.

## Sources and References

### Workspace Sources

* README.md
* docs/planning/README.md
* docs/planning/architecture.md
* docs/planning/decision-log.md
* docs/planning/implementation-roadmap.md
* docs/planning/project-structure.md
* docs/planning/requirements-and-assumptions.md
* docs/planning/risk-register.md
* docs/specifications/grammar.md
* docs/specifications/lexical-grammar.md
* .copilot-tracking/changes/2026-08-25/phase-0-rlm-0001-changes.md
* .copilot-tracking/details/2026-08-25/phase-0-rlm-0001-details.md
* .copilot-tracking/plans/2026-08-25/phase-0-rlm-0001-plan.instructions.md
* .copilot-tracking/plans/logs/2026-08-25/phase-0-rlm-0001-log.md
* .copilot-tracking/research/2026-08-25/phase-0-research.md
* .copilot-tracking/research/subagents/2026-08-25/rlm-0001-grammar-review.md
* .copilot-tracking/research/subagents/2026-08-25/rlm-0001-syntax-research.md
* .copilot-tracking/reviews/2026-08-25/phase-0-rlm-0001-plan-review.md

### Official LLVM Sources

* [LLVM integer type](https://llvm.org/docs/LangRef.html#integer-type)
* [LLVM floating-point types](https://llvm.org/docs/LangRef.html#floating-point-types)
* [LLVM data layout](https://llvm.org/docs/LangRef.html#data-layout)
* [LLVM `add`](https://llvm.org/docs/LangRef.html#add-instruction)
* [LLVM `sdiv`](https://llvm.org/docs/LangRef.html#sdiv-instruction)
* [LLVM `srem`](https://llvm.org/docs/LangRef.html#srem-instruction)
* [LLVM `shl`](https://llvm.org/docs/LangRef.html#shl-instruction)
* [LLVM `fadd`](https://llvm.org/docs/LangRef.html#fadd-instruction)
* [LLVM `frem`](https://llvm.org/docs/LangRef.html#frem-instruction)
* [LLVM `fptosi`](https://llvm.org/docs/LangRef.html#fptosi-to-instruction)
* [LLVM `fptoui`](https://llvm.org/docs/LangRef.html#fptoui-to-instruction)
* [LLVM fast-math flags](https://llvm.org/docs/LangRef.html#fast-math-flags)
* [LLVM constrained floating point](https://llvm.org/docs/LangRef.html#constrained-floating-point-intrinsics)
* [LLVM arithmetic with overflow intrinsics](https://llvm.org/docs/LangRef.html#arithmetic-with-overflow-intrinsics)

### Official Microsoft Sources

* [Microsoft x64 ABI conventions](https://learn.microsoft.com/en-us/cpp/build/x64-software-conventions?view=msvc-170)
* [Microsoft x64 calling convention](https://learn.microsoft.com/en-us/cpp/build/x64-calling-convention?view=msvc-170)
* [Microsoft C++ data type ranges](https://learn.microsoft.com/en-us/cpp/cpp/data-type-ranges?view=msvc-170)

## Recommended Owner Questions

1. Approve the fixed primitive set with no `f16`, pointer-sized integer, or
   128-bit primitive in the MVP?
2. Approve contiguous suffix token pairs with exact suffixes, contextual
   literal typing, and `i32`/`f64` fallback, including the narrow grammar
   amendment?
3. Approve checked ordinary integer arithmetic and checked `as` casts in every
   build, including checked left-shift value overflow?
4. Approve `expression as Type` and the required cast-expression grammar
   amendment?
5. Approve strict binary32/binary64 semantics with the default floating
   environment, preserved subnormals, no ordinary fast-math flags or
   contraction, `frem` semantics for `%`, and unspecified NaN payload/sign?
6. Approve one-byte canonical Boolean storage, four-byte Unicode-scalar `char`,
   zero-sized unit, and layoutless never?
7. Approve nominal declaration identity with structural tuples, arrays,
   references, slices, and function types, while deferring stable package
   identity details?
8. Approve per-body local inference only, omitted private return as unit, and a
   required explicit `-> ()` on public unit-returning functions?
9. Approve the closed coercion set with no implicit numeric conversions and
   with never, reference weakening, array-to-slice, and function-item
   coercions only?
10. Resolve slices as unsized `[T]` behind `&[T]` or `&mut [T]`, authorizing
    correction of the accepted direct `[i32]` parameter example?
11. Approve the initial `String` representation as `{ptr, u64 len, u64 cap}`
    and reject string-valued constants until ownership/static-borrow design?
12. Approve the bounded constant-expression subset, zero-length arrays, and
    zero-byte storage for all-zero-sized aggregates?

## Acceptance Checklist

* [ ] Language owner answers all 12 recommended questions or records a selected
  alternative with consequences
* [ ] docs/specifications/type-system.md defines every primitive's exact value
  domain, suffix/default, and operator set
* [ ] Boolean, character, unit, never, string, reference, slice, array, tuple,
  and function-pointer representation boundaries are explicit
* [ ] Debug and optimized integer overflow, negation, division, remainder,
  shift, and cast behavior are identical and use the accepted panic category
* [ ] IEEE-754 formats, rounding, subnormal, infinity, NaN, signed-zero,
  comparison, remainder, contraction, and floating-environment rules are
  normative
* [ ] The complete explicit cast matrix and closed implicit coercion set are
  normative, with rejected pairs listed
* [ ] Nominal and structural identity rules cover declaration, tuple, array,
  reference, slice, function, and primitive types
* [ ] Local inference boundaries and every public type surface are explicit
* [ ] Constant eligibility, evaluation failures, cycles, array-length rules,
  and target-size checks are explicit
* [ ] Numeric suffix, cast syntax, slice meaning, public unit return, and string
  representation conflicts are reconciled in accepted sources
* [ ] Generic constraints, ownership/loans/drop, unsafe pointers, async,
  sendability, exception ABI, and C-layout work remain explicitly downstream
* [ ] Every `REQ-008`, `REQ-009`, and `REQ-016` clause maps to a normative rule,
  positive case, negative case, and judgment or table row
* [ ] Bounded table/oracle validation passes with identical debug and optimized
  expected rows without implementing a parser or type checker
* [ ] Independent language and Windows x64/LLVM reviews report no implicit
  semantic or backend layout decision
* [ ] README contradictions are marked superseded rather than preserved
