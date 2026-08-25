<!-- markdownlint-disable-file -->
---
description: Execute the first Realm Phase 0 syntax specification task
applyTo: "docs/specifications/**/*.md, spikes/syntax-grammar/**"
---

# Phase 0 RLM-0001 Plan

## User Requests

* Begin implementation of Phase 0 from the implementation roadmap
* Explain options with pros, cons, and complexity when selection is required
* Ask all open questions before committing unresolved language semantics

## Overview and Objectives

Execute `RLM-0001` first. Produce normative lexical and surface grammar
specifications plus disposable validation evidence without modifying production
Rust scaffolding. Preserve the roadmap dependency order.

## Context Summary

The roadmap identifies `RLM-0001` as the first dependency-free language task.
`docs/planning/project-structure.md` prohibits workspace bootstrap before Phase
0 approval. Markdown changes follow the HVE Core markdown and writing-style
instructions. Research is recorded in
`../../research/2026-08-25/phase-0-research.md`.

## Implementation Checklist

### Phase A: Owner Decisions

<!-- parallelizable: false -->

* [x] Select Unicode identifier identity and security policy
* [x] Select negative index and slice normalization
* [x] Select bounds and arithmetic failure category
* [x] Select UTF-8 string indexing and slicing policy

### Phase B: Normative Specifications

<!-- parallelizable: true -->

* [x] Add `docs/specifications/lexical-grammar.md`
* [x] Add `docs/specifications/grammar.md`
* [x] Include valid and invalid examples for every scoped lexical class and
  grammar category
* [x] Define precedence, associativity, semicolon placement, and recovery
  synchronization tables

### Phase C: Disposable Validation

<!-- parallelizable: false -->

* [x] Add a bounded checker under `spikes/syntax-grammar/`
* [x] Map `REQ-001` through `REQ-007` to conformance fixtures
* [x] Verify lossless input partitioning and bounded recovery over at least
  three independent errors
* [x] Record checker command and results in the specification drafts

### Phase D: Acceptance Review

<!-- parallelizable: false -->

* [x] Reconcile every current and stale README contradiction
* [x] Confirm no `QUE-001` through `QUE-003` decision remains implicit
* [x] Review requirement-to-case coverage
* [x] Mark `RLM-0001` complete only after owner approval

Review status: accepted by the language owner after implementation, strict
validation, and two independent reviews completed.

## Dependencies

* `docs/planning/implementation-roadmap.md`
* `docs/planning/requirements-and-assumptions.md`
* `docs/planning/architecture.md`
* `docs/planning/decision-log.md`
* HVE Core markdown and writing-style instructions

## Success Criteria

* Specifications are normative, internally consistent, and reviewable
* Every `REQ-001` through `REQ-007` rule has conformance evidence
* Malformed source examples recover deterministically without a production
  parser commitment
* Production files under `src/` and the root Cargo package remain unchanged
* Downstream tasks can consume the accepted source and syntax contracts without
  inventing missing behavior
