<!-- markdownlint-disable-file -->
---
title: Phase 0 RLM-0002 Changes
description: Implemented type-system specification and disposable validation evidence
ms.date: 2026-08-27
ms.topic: reference
---

## Related Plan

`.copilot-tracking/plans/2026-08-25/phase-0-rlm-0002-plan.instructions.md`

## Summary

Defined the Realm version 0 type-system contract for `REQ-008`, `REQ-009`, and
`REQ-016`, reconciled approved syntax details, and added an isolated semantic
table oracle. The final oracle contains 901 ordered rows and emits matching
debug and release digest `fnv1a64:32262fbb72a93d29`. The language owner
accepted the resulting contract on 2026-08-27.

## Added

* `docs/specifications/type-system.md`
* `spikes/type-system/.gitignore`
* `spikes/type-system/Cargo.toml`
* `spikes/type-system/Cargo.lock`
* `spikes/type-system/src/main.rs`
* RLM-0002 research, plan, details, planning log, changes log, and review log
  under `.copilot-tracking/`

## Modified

* `docs/planning/requirements-and-assumptions.md`
* `docs/specifications/grammar.md`
* `docs/specifications/lexical-grammar.md`

## Review Repairs

* Restricted integer-cast failure rows to values inhabiting the labeled source
  type and recorded widening pairs with no representable failure
* Preserved maximum member alignment for nonempty zero-sized tuples
* Defined zero-sized call payload omission and distinct aggregate argument and
  hidden-result-pointer return rules
* Required Boolean zero-extension at call boundaries
* Added negative examples for cross-body inference and omitted public
  function-type returns
* Clarified that host floating operations are bounded observations rather than
  normative semantic evidence

## Validation

* Rust formatting check passed
* Clippy passed for all oracle targets with warnings denied
* 16 debug tests passed
* 16 release tests passed
* Debug and release executable output matched exactly
* Editor diagnostics reported no errors
* `git diff --check` passed
* Production `src/`, root Cargo files, and the syntax oracle remained unchanged

## Release Summary

`RLM-0002` is accepted. Its normative type-system contract and bounded oracle
evidence now unblock the dependent generic-constraint and ownership tasks.
