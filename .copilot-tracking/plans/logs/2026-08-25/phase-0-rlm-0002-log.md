<!-- markdownlint-disable-file -->
---
title: Phase 0 RLM-0002 Planning Log
description: Decisions, deviations, and follow-up work for the Realm type-system task
ms.date: 2026-08-27
ms.topic: reference
---

## Decision Log

The language owner approved all six recommended policy bundles. The selections
authorize narrow amendments to numeric suffix parsing, cast syntax, slice type
meaning, direct slice examples, and explicit public unit returns.

The initial target representation is explicit but is not source-level type
identity or a stable external Realm ABI. The eventual LLVM/Inkwell probe may
change lowering mechanics but cannot silently change accepted source semantics.

## Scope Controls

The current `spikes/syntax-grammar/src/main.rs` may contain user or automated
edits that are not represented by a tracked diff because the spike is untracked.
Treat its current contents as authoritative and do not edit it during
`RLM-0002`.

Production compiler scaffolding remains blocked until the Phase 0 baseline.
The type-system oracle is disposable evidence rather than type-checker code.

## Phase C Validation

The isolated oracle contains 901 deterministically ordered evidence rows and
emits `fnv1a64:32262fbb72a93d29`. Debug and release executions produced the
same two-line output. All 16 tests passed in both profiles, formatting passed,
and Clippy passed with warnings denied.

The code-level review replaced vacuous `String` and slice-reference layout
assertions with named target-layout values shared by evidence generation and
tests. Integer and float conversions are computed from mathematical ranges and
IEEE bit fields instead of host casts. The small set of host floating-point
operations is a bounded Windows target observation against fixed expected
bits; it is not the normative source for Realm semantics or a substitute for
the Phase D LLVM review.

## Phase D Review

The first independent language review found that the oracle overstated cast
coverage and lacked concrete negative examples for cross-body inference and
public function-type components. The first backend review found a zero-sized
tuple alignment contradiction, unresolved zero-sized call payloads, conflated
aggregate argument and return rules, and unspecified Boolean extension.

The repaired oracle filters integer-cast failures through the declared source
domain and identifies widening pairs whose entire source domain fits. The
specification now retains member alignment for nonempty zero-sized tuples,
omits zero-sized call payloads, distinguishes indirect arguments from hidden
result-pointer returns, and requires Boolean zero-extension. Added examples
close the inference and public function-type gaps.

Both independent second-pass reviews reported no blocking or nonblocking
findings. Strict validation passed after the repairs with 901 rows and matching
debug and release digest `fnv1a64:32262fbb72a93d29`. Production parser,
type-checker, native ABI, and LLVM conformance checks remain assigned to their
downstream owners.

## Owner Acceptance

The language owner explicitly accepted `RLM-0002` on 2026-08-27 after the
current oracle reproduced all validation evidence. The accepted contract now
unblocks `RLM-0004` and `RLM-0005`; downstream production and LLVM gates remain
unchanged.

## Suggested Follow-On Work

After acceptance, `RLM-0003` remains the next syntax-dependent spike and
`RLM-0004` becomes available from the accepted type rules. `RLM-0005` also
depends on `RLM-0002` and owns the deferred ownership boundaries.
