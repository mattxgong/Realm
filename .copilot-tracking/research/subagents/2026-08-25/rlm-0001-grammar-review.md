---
title: RLM-0001 Grammar Review Research
description: Evidence and findings for the RLM-0001 lexical and syntax grammar review
---

## Research Scope

* Compare docs/specifications/lexical-grammar.md and docs/specifications/grammar.md with RLM-0001
* Evaluate coverage of REQ-001 through REQ-007
* Inspect spikes/syntax-grammar/Cargo.toml and spikes/syntax-grammar/src/main.rs for correctness, compile risks, recovery defects, Unicode boundary mistakes, and false validation claims
* Distinguish blockers from acceptable disposable-spike limitations

## Acceptance Criteria Evidence

* `docs/planning/implementation-roadmap.md:42-55` requires a normative lexical
  and surface grammar, valid and invalid examples, resolution or explicit
  blocking follow-up for README contradictions and `QUE-001` through
  `QUE-003`, executable grammar/prototype review, and at least one conformance
  case per syntax requirement.
* `docs/planning/requirements-and-assumptions.md:105-119` defines `REQ-001`
  through `REQ-007`, including malformed UTF-8 diagnostics, a documented
  Unicode policy, explicit semicolons, recoverable lossless syntax, indexing,
  and slicing semantics.
* The specifications state decisions for `QUE-001` through `QUE-003`, and
  `docs/specifications/grammar.md:468-490` reconciles the current README or
  explicitly defers generic spelling to `RLM-0004`.

## Specification Evidence

* `docs/specifications/grammar.md:117-120` has no production for `(T)`, while
  `docs/specifications/grammar.md:130-131` says `(T)` is accepted as a
  parenthesized type.
* `docs/specifications/grammar.md:287` omits `await` from prefix precedence,
  while `docs/specifications/grammar.md:290` assigns it prefix precedence.
* `docs/specifications/grammar.md:401-445` requires a lossless CST, progress,
  missing-token markers, synchronization sets, and a diagnostic cascade limit.
* `docs/specifications/lexical-grammar.md:25-30` requires malformed UTF-8 to
  remain a separate invalid element even next to comments or literals.
* `docs/specifications/lexical-grammar.md:233-269` requires character
  cardinality, NUL, line-boundary, and escape validation.
* `docs/specifications/lexical-grammar.md:312-326` defines a conformance matrix
  broader than the spike tests.

## Spike Evidence

* `spikes/syntax-grammar/src/main.rs:173-206` consumes malformed UTF-8 inside
  comments as trivia without an invalid element or diagnostic.
* `spikes/syntax-grammar/src/main.rs:277-315` consumes malformed UTF-8 and
  unvalidated escapes inside literals and checks only whether a closing quote
  appears.
* `spikes/syntax-grammar/src/main.rs:478-568` returns diagnostics rather than a
  syntax representation, drops trivia before recovery, misdiagnoses tail
  expressions, and indexes past EOF when a delimiter remains open.
* `spikes/syntax-grammar/src/main.rs:601-660` exercises narrow examples and
  then prints that all seven requirements passed.
* `spikes/syntax-grammar/Cargo.toml:7-11` makes the spike a standalone workspace
  with non-exact Unicode dependencies. No lockfile exists beside this manifest.
* Compile/test execution was unavailable: `cargo` was not on the active shell
  path, direct invocation did not yield a usable result, and no `rustc`
  executable was present.

## Findings

### Blockers

1. The executable conformance claim is false for `REQ-004` and `REQ-005`.
   Recovery returns only `Vec<Diagnostic>`, discards trivia, and creates no CST,
   error nodes, skipped-token nodes, or missing-token markers. It also omits the
   specified progress assertion and cascade limit.
2. Recovery can panic on an unclosed `(` or `[` at EOF. EOF is handled only
   when the delimiter stack is empty; otherwise the index advances beyond the
   EOF token and is dereferenced on the next loop iteration.
3. The checker rejects a valid final block expression without a semicolon,
   contradicting the grammar's tail-expression rule.
4. Malformed UTF-8 inside comments and literals is silently absorbed, violating
   `REQ-001` and the lexical element-boundary contract.
5. Character and string literal validation is missing. Empty and multi-scalar
   characters, unknown or malformed escapes, NUL, and escaped line terminators
   can pass without the required diagnostics.
6. Unicode 17.0 and confusable-policy evidence is incomplete and
   non-reproducible. XID and normalization come from independently versioned
   dependencies, default ignorables are hard-coded, confusable skeletons are
   absent, and the standalone spike has no lockfile.
7. The normative type grammar contradicts its prose for parenthesized types.
8. The conformance matrix omits most required lexical classes and broad syntax
   behavior, despite the executable printing that `REQ-001` through `REQ-007`
   passed.

### Lower-Severity Defects

* The precedence table omits `await` even though the unary grammar and prose
  classify it as a prefix operator.
* `char::is_whitespace` accepts the broader Unicode `White_Space` set rather
  than the specified `Pattern_White_Space` set, and the trivia coalescing loop
  combines adjacent line terminators that the specification requires to remain
  separate elements.

### Acceptable Disposable-Spike Limitations

* The lexer repeatedly validates the remaining byte tail and is not suitable
  for production performance.
* Tokens and the recovery algorithm are ad hoc rather than reusable compiler
  architecture.
* Index and slice helpers use `i64` rather than the eventual typed integer
  domain.

These limitations are acceptable only after the conformance blockers are
fixed or the executable-evidence claims are narrowed. Production-quality
architecture and performance remain explicit non-goals for `RLM-0001`.

## Clarifying Questions

None. The roadmap and requirements provide enough information to classify the
findings.
