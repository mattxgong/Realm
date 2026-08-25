---
title: Realm Surface Grammar
description: Normative declaration, statement, expression, indexing, slicing, semicolon, and recovery rules for Realm
ms.date: 2026-08-25
ms.topic: reference
---

## Status and Scope

This specification defines the Realm surface grammar required by `RLM-0001`
and resolves `REQ-003` through `REQ-007`. It consumes tokens and trivia from
[lexical-grammar.md](lexical-grammar.md). Generic constraints, ownership
validity, exception ABI, and concurrency behavior remain assigned to their
later Phase 0 tasks, but their reserved words cannot be used as identifiers.

The grammar and conformance spike remain under review until the `RLM-0001`
acceptance gate is complete. The terms MUST, MUST NOT, SHOULD, SHOULD NOT, and
MAY describe requirements on conforming Realm implementations.

## Grammar Notation

Grammar productions use this notation:

```ebnf
choice       ::= first | second
optional     ::= element?
repetition   ::= element*
one_or_more  ::= element+
group        ::= (first second)
terminal     ::= "token spelling"
```

`IDENTIFIER`, `INTEGER`, `FLOAT`, `CHARACTER`, and `STRING` are tokens from the
lexical grammar. Keywords appear in quotation marks. Trivia may occur between
any two tokens and is retained in the concrete syntax tree.

The EBNF describes accepted token sequences. The precedence table resolves the
expression shorthand that would otherwise be ambiguous. Semantic checks may
reject a syntactically valid program without changing its syntax tree.

## Source Files and Items

A source file defines one module. The module path is derived from its
package-relative file path by the modules-and-packages specification.

```ebnf
source_file       ::= item* EOF
item              ::= import_item
                    | const_item
                    | function_item
                    | structure_item
                    | enum_item

visibility        ::= "pub"
path              ::= IDENTIFIER ("::" IDENTIFIER)*

import_item       ::= "import" path import_alias? ";"
import_alias      ::= "as" IDENTIFIER

const_item        ::= visibility? "const" IDENTIFIER type_annotation
                      "=" expression ";"

function_item     ::= visibility? "async"? "fn" IDENTIFIER
                      "(" parameter_list? ")" throws_clause?
                      return_type? block
parameter_list    ::= parameter ("," parameter)* ","?
parameter         ::= pattern ":" type
throws_clause     ::= "throws"
return_type       ::= "->" type

structure_item    ::= visibility? "struct" IDENTIFIER
                      "{" field_declaration* "}"
field_declaration ::= visibility? IDENTIFIER ":" type ","

enum_item         ::= visibility? "enum" IDENTIFIER
                      "{" variant_declaration* "}"
variant_declaration ::= IDENTIFIER variant_payload? ","
variant_payload   ::= "(" type_list? ")"
                    | "{" field_declaration* "}"
type_list         ::= type ("," type)* ","?
```

Imports and constants end with semicolons. Functions, structures, and enums end
with their closing brace and MUST NOT have a trailing semicolon. Structure
fields and enum variants require trailing commas, including the last member.
This rule keeps recovery stable when members are reordered or added.

Realm version 0 does not permit executable top-level statements. A later
modules-and-packages specification determines which constant initializers are
pure enough for module initialization.

## Deferred Declaration Extensions

Generic parameter and constraint syntax is blocked on `RLM-0004`. Exception
payload and catch syntax is blocked on `RLM-0006`. Structured task and scope
syntax is blocked on `RLM-0007` and `RLM-0008`. These tasks MAY add productions
at the marked declaration and expression boundaries, but they MUST NOT alter
token identity, semicolon inference, indexing syntax, or slice syntax without
reopening `RLM-0001`.

The reserved words `catch`, `spawn`, `where`, and related future words prevent
otherwise valid identifiers from occupying those extension points.

## Types

```ebnf
type              ::= path_type
                    | tuple_type
                    | parenthesized_type
                    | array_type
                    | slice_type
                    | shared_reference_type
                    | exclusive_reference_type
                    | function_type
                    | never_type

path_type         ::= path
tuple_type        ::= "(" ")"
                    | "(" type "," ")"
                    | "(" type "," type ("," type)* ","? ")"
parenthesized_type ::= "(" type ")"
array_type        ::= "[" type ";" const_expression "]"
slice_type        ::= "[" type "]"
shared_reference_type ::= "&" type
exclusive_reference_type ::= "&" "mut" type
function_type     ::= "fn" "(" type_list? ")" throws_clause? return_type?
never_type        ::= "!"
type_annotation   ::= ":" type
const_expression  ::= expression
```

`()` is the unit type. `(T,)` is a one-element tuple type, while `(T)` is a
parenthesized type and therefore equivalent to `T`. Fixed arrays use `[T; N]`.
Borrowed slices use `[T]`. Owned vectors and strings are nominal library types,
not alternate bracket grammar.

Whether an expression is valid in an array length is a semantic constant-check
performed after parsing.

## Blocks and Statements

```ebnf
block             ::= "{" statement* tail_expression? "}"
statement         ::= binding_statement ";"
                    | return_statement ";"
                    | break_statement ";"
                    | continue_statement ";"
                    | throw_statement ";"
                    | expression_statement ";"

binding_statement ::= ("let" | "var") pattern type_annotation?
                      "=" expression
return_statement  ::= "return" expression?
break_statement   ::= "break" expression?
continue_statement ::= "continue"
throw_statement   ::= "throw" expression
expression_statement ::= expression
tail_expression   ::= expression
```

`let` creates an immutable binding and `var` creates a mutable binding. Both
forms require an initializer in this grammar. Partial initialization remains a
later ownership-design concern and cannot be introduced by omitting `=`.

Every statement production ends with an explicit semicolon. Newlines never
terminate statements. The only expression in a block that may omit a semicolon
is the final tail expression immediately followed by `}`. A semicolon converts
that expression into an expression statement, so it no longer supplies the
block value.

Block-bodied items do not use semicolons. Member declarations use commas. The
semicolon rules are therefore:

| Construct                            | Terminator                 |
|--------------------------------------|----------------------------|
| Import or constant item              | Required semicolon         |
| Function, structure, or enum item    | Closing brace              |
| Structure field or enum variant      | Required comma             |
| Binding or control-transfer statement| Required semicolon         |
| Discarded expression statement       | Required semicolon         |
| Final block value                    | No semicolon               |

```realm
fn choose(flag: bool) -> i32 {
    let fallback = 2;
    if flag { 1 } else { fallback }
}
```

```realm
fn invalid() -> i32 {
    let value = 1
    value
}
```

The invalid function is missing the semicolon after the binding. The newline
does not repair it.

## Expressions

```ebnf
expression        ::= assignment_expression
assignment_expression ::= binary_expression assignment_tail?
assignment_tail   ::= assignment_operator assignment_expression
assignment_operator ::= "=" | "+=" | "-=" | "*=" | "/=" | "%="
                      | "&=" | "|=" | "^=" | "<<=" | ">>="

binary_expression ::= unary_expression
                      (binary_operator unary_expression)*
binary_operator   ::= "||" | "&&" | "|" | "^" | "&"
                    | "==" | "!=" | "<" | "<=" | ">" | ">="
                    | "<<" | ">>" | "+" | "-" | "*" | "/" | "%"

unary_expression  ::= unary_operator unary_expression | postfix_expression
unary_operator    ::= "!" | "+" | "-" | "&" | "&" "mut"
                    | "move" | "await"

postfix_expression ::= primary_expression postfix_part*
postfix_part      ::= call_suffix | field_suffix | index_suffix | slice_suffix
call_suffix       ::= "(" argument_list? ")"
argument_list     ::= expression ("," expression)* ","?
field_suffix      ::= "." (IDENTIFIER | INTEGER)
index_suffix      ::= "[" expression "]"
slice_suffix      ::= "[" expression? ":" expression? "]"

primary_expression ::= literal
                     | path
                     | parenthesized_expression
                     | tuple_expression
                     | array_expression
                     | structure_expression
                     | block
                     | if_expression
                     | loop_expression
                     | while_expression
                     | for_expression
                     | match_expression

literal           ::= INTEGER | FLOAT | CHARACTER | STRING | "true" | "false"
parenthesized_expression ::= "(" expression ")"
tuple_expression  ::= "(" expression "," ")"
                    | "(" expression "," expression
                      ("," expression)* ","? ")"
array_expression  ::= "[" "]"
                    | "[" expression ("," expression)* ","? "]"
                    | "[" expression ";" const_expression "]"

structure_expression ::= path "{" field_initializer* "}"
field_initializer ::= IDENTIFIER (":" expression)? ","

if_expression     ::= "if" expression block
                      ("else" (if_expression | block))?
loop_expression   ::= "loop" block
while_expression  ::= "while" expression block
for_expression    ::= "for" pattern "in" expression block

match_expression  ::= "match" expression "{" match_arm* "}"
match_arm         ::= pattern match_guard? "=>" expression ","
match_guard       ::= "if" expression
```

An expression used as a receiver is evaluated before its postfix arguments.
Call arguments and collection bounds evaluate once from left to right. Semantic
analysis determines whether assignment targets are places, tuple projections
are valid, calls are callable, and control-flow expressions have compatible
types.

The dangling `else` belongs to the nearest unmatched `if`. Braces are mandatory
for every `if`, `else`, loop, `while`, and `for` body.

## Operator Precedence

The parser MUST apply these levels from lowest binding power to highest:

| Level | Associativity | Forms                                      |
|------:|---------------|--------------------------------------------|
|     1 | Right         | Assignments and compound assignments       |
|     2 | Left          | `||`                                       |
|     3 | Left          | `&&`                                       |
|     4 | Left          | `|`                                        |
|     5 | Left          | `^`                                        |
|     6 | Left          | `&`                                        |
|     7 | None          | `==`, `!=`                                 |
|     8 | None          | `<`, `<=`, `>`, `>=`                       |
|     9 | Left          | `<<`, `>>`                                 |
|    10 | Left          | `+`, `-`                                   |
|    11 | Left          | `*`, `/`, `%`                              |
|    12 | Right         | Prefix `!`, `+`, `-`, `&`, `& mut`, `move`, `await` |
|    13 | Left          | Calls, fields, indexing, and slicing       |

`await` has prefix precedence in version 0. Equality and comparison operators
do not chain. `a < b < c` is a syntax error; write `a < b && b < c`.

Assignment is right-associative, so `a = b = value` parses as
`a = (b = value)`. Semantic analysis may reject it when assignment has unit
type. All other binary operators at the same level group left.

## Patterns

```ebnf
pattern           ::= or_pattern
or_pattern        ::= primary_pattern ("|" primary_pattern)*
primary_pattern   ::= "_"
                    | literal
                    | IDENTIFIER
                    | tuple_pattern
                    | variant_pattern
                    | structure_pattern

tuple_pattern     ::= "(" pattern "," ")"
                    | "(" pattern "," pattern ("," pattern)* ","? ")"
variant_pattern   ::= path "(" pattern_list? ")"
pattern_list      ::= pattern ("," pattern)* ","?
structure_pattern ::= path "{" field_pattern* "}"
field_pattern     ::= IDENTIFIER (":" pattern)? ","
```

Name resolution distinguishes a binding identifier from a unit variant path.
Patterns do not include regular expressions, numeric ranges, rest patterns, or
guards outside match-arm syntax in version 0.

## Indexing

Indexing uses `receiver[index]`. A non-negative index is zero-based. A negative
index counts from the end of the sequence.

For sequence length `n` and signed index `i`, indexing succeeds exactly when
`0 <= i < n` or `-n <= i < 0`. A successful negative index normalizes to
`n - magnitude(i)`. The implementation MUST compare the unsigned magnitude
before subtraction so the most-negative signed integer cannot overflow during
normalization.

```text
length 4:  0  1  2  3
negative: -4 -3 -2 -1
```

An index equal to `n`, less than `-n`, or applied to an empty sequence fails.
Failure raises the dedicated non-catchable Realm panic, runs initialized
cleanups, and terminates deterministically in every build mode. It does not
satisfy or require a declared `throws` effect.

```realm
let first = values[0];
let last = values[-1];
```

## Slicing

Slicing uses `receiver[start:end]`. The start is inclusive and the end is
exclusive. Version 0 has exactly one colon and no stride or step expression.

For sequence length `n`:

* An omitted start normalizes to `0`
* An omitted end normalizes to `n`
* A non-negative bound remains unchanged
* A negative bound from `-n` through `-1` normalizes to
  `n - magnitude(bound)`
* Both normalized bounds must satisfy `0 <= start <= end <= n`

Equal bounds produce an empty slice. Bounds do not clamp. Reversed or
out-of-range bounds raise the same cleanup-preserving panic as invalid indexing.

```realm
let all = values[:];
let prefix = values[:2];
let suffix = values[-2:];
let middle = values[1:-1];
let empty = values[2:2];
```

```realm
let reversed = values[3:1];
let too_far = values[:-5];
```

The invalid examples fail for a sequence of length four.

A slice is a borrowed view, not an implicit copy. Slicing an exclusively
borrowed mutable sequence may produce an exclusive mutable slice; all other
valid receivers produce a shared slice. Detailed loan conflicts and lifetime
rules are defined by `RLM-0005`.

Direct `string[index]` is a type error. Direct bracket slicing on an owned UTF-8
string is also a type error. String APIs expose explicit byte and Unicode-scalar
iteration and may expose byte-range slicing that checks both endpoints are UTF-8
code-point boundaries. Grapheme segmentation belongs to a Unicode library.

## Failure Category Boundary

Invalid collection bounds use a non-catchable Realm panic in debug and optimized
builds. The panic runs deterministic cleanup before termination and remains
distinct from declared exceptions, cancellation, and compiler internal errors.

`RLM-0002` specifies the full arithmetic matrix. It MUST preserve the selected
build-independent failure category for integer overflow, division or remainder
by zero, signed minimum divided by negative one, negation overflow, and invalid
shift counts. Explicit checked, wrapping, and saturating operations may return
values instead of panicking.

## Recoverable Parsing

Parsing MUST produce a lossless concrete syntax tree plus ordered diagnostics
even when input is malformed. Tokens, trivia, lexical errors, skipped tokens,
and inserted missing-token markers remain represented in source order.
Inserted markers have empty ranges and do not invent source bytes.

Every recovery action MUST do one of the following:

* Consume at least one token into an error node
* Insert exactly one expected token marker when the current token belongs to
  the expected token's follow set
* Return to a caller that immediately consumes or inserts

The parser MUST assert progress in debug and fuzz configurations. A parser loop
cannot repeat at the same token position with the same recovery state.

### Synchronization Sets

At source-file scope, synchronize on an item starter (`import`, `const`, `pub`,
`async`, `fn`, `struct`, or `enum`) or end of file.

At item scope, synchronize on `;`, the matching `}`, an item starter, or end of
file. Consume a synchronizing semicolon that belongs to the malformed item, but
leave a candidate next-item starter for the source-file parser.

At statement scope, synchronize on `;`, `}`, a statement starter (`let`, `var`,
`return`, `break`, `continue`, or `throw`), or an expression starter. Consume
the malformed statement's semicolon and leave `}` for the block parser.

Within comma-separated lists, synchronize on `,` or the matching closing
delimiter. Within expressions, synchronize on `,`, `;`, `)`, `]`, `}`, `=>`,
or the slice colon when the parser is inside brackets.

An unexpected closing delimiter is retained in an error node owned by the
nearest construct that cannot accept it. An unclosed delimiter produces an
empty missing-token marker at the recovery point.

### Diagnostic Cascade Limit

A parser emits at most eight syntax diagnostics while recovering within one
delimited construct. On the ninth defect it emits one
`PARSE_DIAGNOSTICS_SUPPRESSED` diagnostic and consumes to that construct's
closing delimiter or outer synchronization set. Lexical diagnostics do not
count toward this syntax limit.

Diagnostics are ordered by primary byte-range start, then range end, then code.
Missing-token diagnostics at the same offset precede diagnostics for a token
consumed at that offset.

## Recovery Example

This source contains three independent syntax errors:

```realm
fn recover(input: [i32]) -> i32 {
    let first = input[0]
    let middle = input[1:3;
    return first + ;
}
```

A conforming parser reports the missing binding semicolon, missing slice closing
bracket, and missing right operand. It retains all source bytes, reaches the
function closing brace, and emits a function item containing error nodes rather
than abandoning the file.

## README Reconciliation

The root README is non-normative. This specification resolves its bracket
ambiguity as follows:

```realm
var flags: [bool; 5] = [true, false, true, false, true];
var values: Vec<i32> = [1, 2, 3, 4];
var repeated: [char; 3] = ['d'; 3];
let fourth = values[3];
let last = values[-1];
let interior = values[1:-1];
```

`Vec<i32>` is illustrative nominal library syntax whose generic spelling remains
blocked on `RLM-0004`; it is not accepted by this grammar until that task is
approved. The fixed-array and repeat forms are normative. `f16` is deferred and
owned strings are nominal library values rather than primitives.

| README concern | Disposition |
|---|---|
| `[bool, 5]` fixed-array type | Replaced by `[bool; 5]` |
| `[i32]` used for an owned list | Reserved for borrowed slices; `Vec<i32>` is deferred to `RLM-0004` |
| `['d', 3]` repetition | Replaced by the expression `['d'; 3]` and type `[char; 3]` |
| `f16` primitive claim | Deferred to `RLM-0002` and target-layout review |
| Implicit string primitive | Owned strings remain nominal library values |
| Semicolon, dotted index, output `5`, and `/n` claims | Superseded README revision; current source already uses `;`, `[3]`, `4`, and `\n` |

## Requirement Coverage

| Requirement | Normative coverage | `RLM-0001` checker evidence | Downstream implementation gate |
|---|---|---|---|
| `REQ-001` | Source encoding and invalid-byte recovery | Malformed UTF-8 class matrix and byte partition | `RLM-0102`, `RLM-0104` |
| `REQ-002` | Identifier identity and security policy | Unicode 17 table assertions, NFC collision, invisible, and representative UTS 39 fixtures | `RLM-0104` with complete version-matched data |
| `REQ-003` | Block, statement, and semicolon rules | Valid tail expression and missing-semicolon fixtures | `RLM-0105`, `RLM-0106` |
| `REQ-004` | Recovery, progress, and cascade policy | Three-error recovery, EOF delimiter recovery, and eight-diagnostic cap | `RLM-0003`, `RLM-0107` |
| `REQ-005` | Lossless source-order syntax contract | Token, trivia, and invalid-element byte round trip | `RLM-0003` measured CST and `RLM-0105` snapshots |
| `REQ-006` | Index grammar, normalization, and panic behavior | Empty, positive, negative, boundary, and minimum-integer cases | `RLM-0405` |
| `REQ-007` | Slice grammar, bounds, borrowing, and panic behavior | Omitted, negative, empty, reversed, and out-of-range cases | `RLM-0405` |

The disposable checker is a bounded grammar-review aid, not a production parser
or the measured CST prototype assigned to `RLM-0003`. Its 21 tests exercise at
least one case for every `REQ-001` through `REQ-007` rule and the selected
lexical/recovery edge classes. It intentionally does not implement a complete
UTS 39 engine, a hierarchical lossless tree, typed syntax access, or every
grammar production. Those implementation gates remain assigned to
`RLM-0003`, `RLM-0104` through `RLM-0107`, and `RLM-0405`.

## Validation Evidence

The isolated checker is not a member of the root package or a future production
workspace. On stable `x86_64-pc-windows-msvc`, these commands pass:

```powershell
cargo fmt --manifest-path spikes/syntax-grammar/Cargo.toml -- --check
cargo clippy --manifest-path spikes/syntax-grammar/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path spikes/syntax-grammar/Cargo.toml
cargo run --manifest-path spikes/syntax-grammar/Cargo.toml
```

The test run reports 21 passed tests. The executable reports that its bounded
checks pass while full production conformance remains pending.
