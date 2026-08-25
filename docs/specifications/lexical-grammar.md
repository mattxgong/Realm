---
title: Realm Lexical Grammar
description: Normative source decoding, trivia, identifier, literal, token, and lexical recovery rules for Realm
ms.date: 2026-08-25
ms.topic: reference
---

## Status and Scope

This specification is the normative lexical contract for Realm source files.
It resolves the lexical parts of `RLM-0001`, `REQ-001`, `REQ-002`, and
`REQ-005`. The grammar and conformance spike remain under review until the
`RLM-0001` acceptance gate is complete.

The terms MUST, MUST NOT, SHOULD, SHOULD NOT, and MAY describe requirements on
conforming Realm implementations. Source ranges are zero-based, half-open byte
ranges into the original source file.

## Source Encoding

A Realm source file is a finite byte sequence encoded as UTF-8. A compiler MUST
retain the original bytes for diagnostics and lossless syntax reconstruction.
It MUST NOT silently replace malformed input with U+FFFD.

The compiler MUST apply the Unicode maximal-subpart replacement boundary rules
to identify each ill-formed UTF-8 subsequence. It MUST emit one ordered
diagnostic for each identified subsequence, retain that byte range as an
`INVALID_UTF8` lexical element, and continue decoding at the next boundary. An
invalid element never combines with adjacent valid text into an identifier,
literal, comment, or operator.

An optional UTF-8 byte-order mark (`EF BB BF`) is permitted only at byte offset
zero. It is retained as trivia and has no semantic effect. U+FEFF elsewhere is
an invalid source character.

U+0000 is invalid outside an escape sequence. A compiler MUST diagnose it,
retain its byte range as `INVALID_CHARACTER`, and continue.

## Unicode Version

Realm version 0 source rules use Unicode 17.0. A compiler MUST use one
consistent Unicode version for normalization, identifier properties, and
confusable analysis. Changing this version is a language compatibility change.

Implementations MUST validate their Unicode data against the version-matched
Unicode Character Database and normalization test suite.

## Lines and Whitespace

The following sequences are line terminators and form `NEWLINE` trivia:

* U+000A LINE FEED
* U+000D CARRIAGE RETURN
* U+000D U+000A as one line terminator
* U+000B LINE TABULATION
* U+000C FORM FEED
* U+0085 NEXT LINE
* U+2028 LINE SEPARATOR
* U+2029 PARAGRAPH SEPARATOR

A carriage return followed by a line feed is one trivia element covering both
code points. Other adjacent terminators remain separate elements.

U+0009 CHARACTER TABULATION, U+0020 SPACE, and Unicode characters with the
`Pattern_White_Space` property are whitespace trivia when they are not already
classified as line terminators. U+200E LEFT-TO-RIGHT MARK and U+200F
RIGHT-TO-LEFT MARK are permitted only as standalone trivia between tokens.

Whitespace and newlines separate tokens but do not insert or imply semicolons.
Every required semicolon is an explicit U+003B character.

## Comments

`//` begins a line comment. It consumes source through, but not including, the
next line terminator or end of file.

`/*` begins a block comment and `*/` closes it. Block comments nest. Each nested
`/*` increments the nesting depth and each `*/` decrements it. A block comment
ends when its depth returns to zero.

An unclosed block comment consumes through end of file and produces one
`LEX_UNTERMINATED_BLOCK_COMMENT` diagnostic at the unmatched opening delimiter.
Nested unmatched openings MAY be attached as secondary labels to that
diagnostic.

Comments are trivia and MUST be retained exactly. Realm version 0 does not
assign documentation semantics to any comment spelling.

```realm
// One line comment
let ready = true; /* outer /* nested */ outer */
```

```realm
/* invalid because the outer comment never closes
let hidden = 1;
```

## Identifiers

An identifier starts with U+005F LOW LINE (`_`) or a code point with the Unicode
`XID_Start` property. Each following code point MUST have the `XID_Continue`
property.

An identifier MUST NOT contain a code point with the
`Default_Ignorable_Code_Point` property. This exclusion includes U+200C ZERO
WIDTH NON-JOINER and U+200D ZERO WIDTH JOINER in Realm version 0. A disallowed
code point terminates the surrounding identifier, produces
`LEX_DISALLOWED_IDENTIFIER_CHARACTER`, and is retained as an invalid lexical
element.

The trivia classification takes precedence for U+200E and U+200F. Each is a
permitted token separator, including when adjacent to identifier characters,
and does not produce `LEX_DISALLOWED_IDENTIFIER_CHARACTER`. It never becomes
part of either adjacent identifier.

Identifiers are case-sensitive. Their semantic identity is the Unicode NFC
normalization of their source spelling. The lossless syntax tree and diagnostics
retain the original spelling and byte range. Symbol tables, duplicate checks,
imports, and mangling MUST use the normalized identity.

Two distinct source spellings with the same normalized identity denote the
same name. A declaration that collides after normalization is a duplicate
declaration and SHOULD display both source spellings.

```realm
let delta = 1;
let Δ = 2;
let café = 3;
```

The following source contains a duplicate name because the two spellings
normalize to the same NFC sequence:

```realm
let café = 1;
let café = 2;
```

The second spelling above uses U+0065 followed by U+0301.

### Confusable Diagnostics

A compiler MUST compute Unicode Technical Standard 39 confusable skeletons
from version-matched data. It MUST warn when two visible declarations in the
same package have distinct normalized identities but equal confusable
skeletons. It MUST also warn when an identifier is confusable with a keyword.

A compiler SHOULD warn for mixed-script identifier chunks that violate the
Unicode highly restrictive profile. These conditions are warnings, not name
aliases and not lexical rejection. Warning suppression MUST NOT change semantic
identity.

### Reserved Identifiers

A single `_` is the discard pattern token, not an identifier expression. Names
beginning with two underscores are reserved for the Realm implementation. User
source MUST NOT declare them.

Realm version 0 has no raw-identifier escape. Keywords cannot be used as names.

## Keywords

The following NFC ASCII spellings are keywords:

```text
as async await break catch const continue else enum false fn for if import in
let loop match move mut pub return struct throw throws true unsafe var while
```

The following spellings are reserved for planned language evolution and cannot
be used as identifiers:

```text
actor dyn interface macro select spawn trait type where yield
```

A non-ASCII spelling that is visually confusable with a keyword remains an
identifier and triggers the confusable warning policy.

## Numeric Literals

Numeric literals use ASCII digits. The unary `+` and `-` operators are separate
tokens and are never part of a numeric literal.

An integer literal has one of these forms:

```ebnf
decimal_integer ::= decimal_digit (decimal_digit | "_")*
binary_integer  ::= "0" ("b" | "B") binary_digit (binary_digit | "_")*
octal_integer   ::= "0" ("o" | "O") octal_digit (octal_digit | "_")*
hex_integer     ::= "0" ("x" | "X") hex_digit (hex_digit | "_")*
```

An underscore MUST occur only between two digits valid for the literal base.
Leading, trailing, doubled, or prefix-adjacent underscores are invalid. A base
prefix MUST be followed by at least one valid digit.

A decimal floating-point literal has a decimal point, an exponent, or both:

```ebnf
float_literal ::= decimal_digits "." decimal_digits? exponent?
                | decimal_digits exponent
exponent      ::= ("e" | "E") ("+" | "-")? decimal_digits
decimal_digits ::= decimal_digit (decimal_digit | "_")*
```

The same underscore placement rule applies independently to the integer,
fraction, and exponent digit sequences. `1.` is a floating-point literal.
`.5` is tokenized as `.` followed by integer `5` and is not a floating-point
literal.

Literal type suffixes and default types are specified by the Phase 0 type-system
task. In this lexical grammar, an immediately following identifier is a
separate token. Numeric value overflow is not a lexical error.

```realm
let decimal = 1_000;
let mask = 0xff_00;
let ratio = 1.25e-2;
```

```realm
let bad_binary = 0b102;
let bad_separator = 1__000;
let missing_exponent = 1e+;
```

The lexer MUST consume the full malformed numeric candidate in each invalid
example so one defect does not become a misleading sequence of valid tokens.

## Boolean Literals

`true` and `false` are Boolean literal keywords.

## Character Literals

A character literal begins and ends with U+0027 APOSTROPHE. Its value MUST be
exactly one Unicode scalar value after escape processing. A line terminator,
unescaped apostrophe, unescaped backslash, surrogate code point, or multiple
scalar values is invalid.

```ebnf
character_literal ::= "'" (character_content | escape_sequence) "'"
```

## String Literals

A string literal begins and ends with U+0022 QUOTATION MARK. Its value is a
sequence of Unicode scalar values encoded as UTF-8. An unescaped quotation mark,
unescaped backslash, line terminator, or U+0000 is invalid within the literal.

```ebnf
string_literal ::= '"' (string_content | escape_sequence)* '"'
```

Realm version 0 has no raw, byte, interpolated, or multiline string literals.

An unclosed character or string literal ends immediately before a line
terminator or at end of file. The lexer MUST emit one unterminated-literal
diagnostic, retain the consumed source as one malformed literal, and leave the
line terminator available for normal lexing.

## Escape Sequences

Character and string literals accept only these escapes:

```text
\0  \\  \"  \'  \n  \r  \t  \u{H...}
```

`\u{H...}` contains one through six ASCII hexadecimal digits and denotes one
Unicode scalar value. Surrogates and values above U+10FFFF are invalid. An
unknown or malformed escape produces one diagnostic but remains inside the
surrounding literal token so lexing can continue after its closing delimiter.

## Operators and Delimiters

The lexer recognizes these fixed tokens:

```text
( ) { } [ ] , ; : :: . -> =>
= += -= *= /= %= &= |= ^= <<= >>=
+ - * / % ! & | ^ << >> && ||
== != < <= > >=
```

When fixed tokens share a prefix, the lexer MUST choose the longest valid token.
For example, `>>=` is one token rather than `>>` followed by `=`. Comments take
priority over `/` and `*` operators after the opening pair is recognized.

## Token Boundaries and Invalid Characters

Trivia separates tokens but is not required when adjacent tokens have an
unambiguous boundary. `letx` is one identifier, not `let` followed by `x`.

A valid Unicode scalar that belongs to no token class produces
`LEX_INVALID_CHARACTER`. The lexer MUST retain it as one invalid element and
continue with the next scalar. Consecutive invalid scalars remain independently
addressable source ranges, although a renderer MAY group adjacent diagnostics.

The lexer MUST always make progress. Each iteration consumes at least one
source byte unless it emits end of file. The ordered concatenation of token,
trivia, and invalid-element byte ranges MUST exactly cover the original byte
sequence without overlap or gaps.

## Lexical Diagnostic Order

Diagnostics are ordered by primary byte-range start, then range end, then
diagnostic code. Confusable warnings that require name resolution follow all
lexical diagnostics and use deterministic declaration order as a final key.

Lexical diagnostics do not stop token production. Parser recovery consumes
malformed literal and invalid-character elements through the invalid-token
interface defined by the grammar specification.

## Conformance Cases

The disposable `RLM-0001` checker provides bounded review evidence for these
classes:

* Empty source, BOM-only source, and each recognized line ending
* Valid one-, two-, three-, and four-byte UTF-8 scalars
* Truncated, overlong, surrogate, out-of-range, and stray-continuation UTF-8
* ASCII, non-ASCII, normalization-equivalent, prohibited default-ignorable,
  and representative confusable identifier spellings
* Line, nested block, and unterminated block comments
* Valid and malformed integers, floating-point numbers, characters, strings,
  and escapes
* Every fixed token and every longest-match ambiguity
* NUL, U+FEFF outside byte zero, and valid but unassigned token characters
* Lossless byte partitioning and deterministic diagnostic order

The checker pins `unicode-ident` and `unicode-normalization` versions whose
embedded constants both report Unicode 17.0.0. Its representative confusable
mappings come from the Unicode 17.0.0 `confusables.txt` data for Cyrillic small
letters A and O and Greek small letter omicron. This subset validates the
documented warning examples but is not a complete UTS 39 implementation.

Complete Unicode property, normalization, confusable, and mixed-script data
validation remains an acceptance requirement for the production lexer in
`RLM-0104`. Complete lossless tree evidence remains assigned to `RLM-0003` and
`RLM-0105`. The bounded cases here provide `RLM-0001` review evidence for
`REQ-001`, `REQ-002`, and the lexical portion of `REQ-005`.
