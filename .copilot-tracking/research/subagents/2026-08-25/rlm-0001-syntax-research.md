<!-- markdownlint-disable-file -->
---
title: RLM-0001 Syntax Research
description: Evidence and recommendations for the Realm lexical and surface syntax specification
ms.date: 2026-08-25
ms.topic: reference
---

## Scope

Research covers `RLM-0001`, `REQ-001` through `REQ-007`, `QUE-001` through
`QUE-003`, the current root README example, and the syntax boundaries required
by later parser and collection tasks. Production Rust code is outside scope.

## Findings

The planning baseline already requires UTF-8 source, Unicode identifiers,
semicolon-terminated statements, recoverable lossless syntax, zero-based
square-bracket indexing, negative indexing, and slicing. Four owner decisions
remain before the specifications can be normative:

* Unicode identifier normalization and confusable handling
* Negative index and slice-bound normalization
* Bounds and arithmetic failure behavior
* Whether owned UTF-8 strings permit direct indexing or slicing

The grammar must also settle UTF-8 recovery, line endings, whitespace, comment
nesting, literal and escape forms, keyword inventory, precedence, semicolon
placement, and parser synchronization tokens. These details have low product
ambiguity and can follow the recommended engineering defaults in the plan.

## Recommended Decisions

### Unicode Identifiers

Pin a Unicode version, accept `_` or `XID_Start` followed by `XID_Continue`,
compare identifiers by case-sensitive NFC, and preserve original source bytes
in the syntax tree. Exclude default-ignorable code points from identifiers.
Warn for mixed-script confusables and skeleton collisions rather than treating
confusable spellings as aliases.

This policy balances source fidelity and canonical equality. Raw code-point
equality creates visually identical but distinct names. NFKC can collapse
meaningfully distinct mathematical and styled characters. Rejecting every
confusable produces excessive false positives. Implementation complexity is
medium because normalization data and identifier-security data must be pinned
and tested with the compiler.

### Indexing and Slicing

Normalize a negative index from the end. For a sequence of length `n`, valid
element indices satisfy `-n <= i < n`; a negative `i` denotes `n - abs(i)`.
Use `value[start:end]` with an inclusive start, exclusive end, no step in the
MVP, omitted bounds of `0` and `n`, and negative bounds normalized from the end.
Require `0 <= start <= end <= n`; equal bounds produce an empty slice.

Strict bounds expose mistakes and preserve simple ownership rules. Clamping or
reversed-to-empty behavior is more permissive but can hide defects. Slice steps
substantially expand grammar, runtime representation, borrowing, and iterator
semantics. The strict MVP has low-to-medium implementation complexity.

### Bounds and Arithmetic Failures

Use a dedicated non-catchable language panic in every build. It runs initialized
cleanups and terminates deterministically. Always check integer overflow,
division and remainder by zero, signed minimum divided by negative one,
negation overflow, and invalid shifts. Provide explicit checked, wrapping, and
saturating operations later.

Catchable failures would add `throws` to ordinary arithmetic and indexing.
Build-dependent wrapping or abort semantics make correctness depend on
optimization settings. Immediate abort is simpler but conflicts with
deterministic cleanup. The recommended policy has medium implementation
complexity and requires explicit CFG failure edges plus a panic runtime path.

### UTF-8 String Access

Prohibit direct string indexing. Expose explicit byte and Unicode-scalar
iteration. Permit byte-range slicing only at UTF-8 code-point boundaries;
grapheme operations belong in a Unicode library.

Byte indexing is fast but can expose invalid character fragments. Scalar or
grapheme indexing suggests constant-time access that UTF-8 cannot provide.
Prohibiting direct indexing keeps complexity low and leaves APIs explicit.

## Grammar Boundary

The two specifications should define source decoding, trivia, comments,
identifiers, keywords, literals, declarations, statements, expressions, types,
patterns, postfix indexing and slicing, precedence, associativity, semicolon
rules, and recovery. Array syntax should remain unambiguous:

```ebnf
array_type     ::= "[" type ";" const_expression "]"
slice_type     ::= "[" type "]"
array_literal  ::= "[" expression ("," expression)* ","? "]"
array_repeat   ::= "[" expression ";" const_expression "]"
index          ::= "[" expression "]"
slice          ::= "[" expression? ":" expression? "]"
```

The grammar should permit a final unsemicolonized expression only as a block
value. Newlines remain trivia and never infer statement termination. Recovery
anchors should include semicolons, matching closing delimiters, commas in
lists, item starters, statement starters, and expression followers.

## README Reconciliation

The root README is non-normative and source compatibility is an explicit
non-goal. `RLM-0001` must resolve these current issues:

* The primitive list advertises deferred `f16` and omits the used string type
* `[bool, 5]`, `[i32]`, and `['d', 3]` mix type and value grammar
* Literal defaults, tuple access, bounds, negative indices, and slicing are not
  demonstrated

The planning index also describes stale contradictions. The current README now
contains semicolons, `list[3]`, output `4`, and `Hello\n`; the older claims about
a missing semicolon, dotted indexing, output `5`, and `Hello/n` should be
retired when the normative example is approved.

## Validation

Use a disposable lexer/parser or grammar checker under `spikes/`. Map each of
`REQ-001` through `REQ-007` to positive, negative, range, diagnostic, and
recovery cases. Include malformed UTF-8, all selected line endings,
nested/unterminated comments, literal boundaries, normalization-equivalent
identifiers, forbidden invisibles, confusable collisions, semicolon omissions,
precedence trees, index boundaries, empty and reversed slices, and identical
debug/optimized failure behavior.

Run version-matched Unicode normalization, derived-core-property, identifier,
and confusable data through the prototype. Property tests must prove byte-range
partitioning, lossless reconstruction, parser progress, deterministic ordered
diagnostics, and bounded recovery.

## Sources

* [Unicode Standard Annex 31](https://www.unicode.org/reports/tr31/)
* [Unicode Technical Standard 39](https://www.unicode.org/reports/tr39/)
* [Unicode Technical Standard 55](https://www.unicode.org/reports/tr55/)
* [Rust identifiers](https://doc.rust-lang.org/reference/identifiers.html)
* [Python sequence operations](https://docs.python.org/3/library/stdtypes.html#common-sequence-operations)
* [LLVM arithmetic intrinsics](https://llvm.org/docs/LangRef.html#arithmetic-with-overflow-intrinsics)
* `docs/planning/implementation-roadmap.md`
* `docs/planning/requirements-and-assumptions.md`
* `docs/planning/architecture.md`
* `docs/planning/decision-log.md`
* `docs/planning/README.md`
* `README.md`

## Next Step

Obtain owner decisions for the four open semantic choices, then draft
`docs/specifications/lexical-grammar.md` and
`docs/specifications/grammar.md`. Keep arithmetic details scoped for
`RLM-0002` while recording the selected failure category in `RLM-0001`.