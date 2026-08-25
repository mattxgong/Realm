<!-- markdownlint-disable-file -->
---
title: Phase 0 RLM-0001 Planning Log
description: Decisions, deviations, and follow-up work for the first Realm Phase 0 task
ms.date: 2026-08-25
ms.topic: reference
---

## Discrepancy Log

* `QUE-003` combines bounds and arithmetic failure semantics. `RLM-0001` needs
  the failure category for indexing and slicing, while detailed arithmetic
  operation rules belong in `RLM-0002`.
* The planning index lists stale README defects. The current README has already
  corrected the missing semicolon, dotted indexing, result value, and newline
  spelling, but its grammar remains non-normative and ambiguous.
* String indexing semantics are not assigned a `QUE` identifier, yet
  `RLM-0405` requires Unicode string-boundary validation. The policy must be
  chosen now or recorded as an explicit blocking follow-up.
* The first independent review found an ambiguity between U+200E/U+200F trivia
  and default-ignorable identifier exclusion. The lexical specification now
  gives exact trivia classification precedence, and the checker verifies these
  characters separate adjacent identifiers without diagnostics.
* The first independent review found that the conformance inventory exceeded
  its fixtures. Existing tests now cover empty and BOM-only sources, U+0378 as
  a valid unassigned scalar, and fixed-token prefix ambiguities. The inventory
  now limits confusable claims to representative mappings.

## Implementation Paths Considered

The selected path starts with `RLM-0001`, obtains owner decisions, writes two
specifications, and validates them with a disposable checker. This follows the
roadmap critical path and prevents the current lexer experiment from becoming
accidental architecture.

Parallel execution of all Phase 0 tasks was rejected because most tasks depend
on `RLM-0001`, `RLM-0002`, or `RLM-0005`. Starting `RLM-0011` in parallel can be
considered after the language owner confirms whether Phase 0 should use parallel
work streams.

The checker remains an isolated disposable crate. Exact dependency versions
provide Unicode 17.0.0 XID and NFC tables. Representative confusable mappings
are bounded evidence, while complete UTS 39 and mixed-script analysis remain
assigned to the production lexer task.

## Validation Record

The final implementation state passed these checks:

* `cargo fmt --manifest-path spikes/syntax-grammar/Cargo.toml -- --check`
* `cargo clippy --manifest-path spikes/syntax-grammar/Cargo.toml --all-targets -- -D warnings`
* `cargo test --manifest-path spikes/syntax-grammar/Cargo.toml` with 21 tests
* `cargo run --manifest-path spikes/syntax-grammar/Cargo.toml`
* Markdown frontmatter, trailing-whitespace, and final-newline checks
* `git diff --check`
* VS Code diagnostics for all edited source and specification files

Two independent read-only reviews were performed. The first produced two local
findings that were repaired. The second found no remaining issues and rated the
task ready for owner approval.

The language owner explicitly approved `RLM-0001` after reviewing the final
status and the consequences of acceptance. The task is complete.

## Suggested Follow-On Work

After `RLM-0001` acceptance, begin `RLM-0002` and `RLM-0003`. `RLM-0010` can
also consume the accepted grammar. `RLM-0011` remains independent and is a
candidate for a separate Windows toolchain work stream.
