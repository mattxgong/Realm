<!-- markdownlint-disable-file -->
---
description: Execute the Realm Phase 0 primitive and type-system specification task
applyTo: "docs/specifications/**/*.md, spikes/type-system/**"
---

# Phase 0 RLM-0002 Plan

## User Requests

* Continue with suggested work item 1, `RLM-0002`
* Preserve the current externally edited syntax checker unless a verified
  requirement demands a change
* Explain owner choices with benefits, costs, and complexity
* Complete the RPI research, plan, implementation, review, and discovery cycle

## Overview

Produce a normative type-system specification and bounded table-oracle evidence
for `REQ-008`, `REQ-009`, and `REQ-016`. Reconcile the five owner-approved
syntax clarifications while keeping production compiler files unchanged.

## Context

* Research: `../../research/2026-08-25/rlm-0002-type-system-research.md`
* Detailed research:
  `../../research/subagents/2026-08-25/rlm-0002-type-system-research.md`
* Accepted syntax contract: `docs/specifications/grammar.md` and
  `docs/specifications/lexical-grammar.md`
* Roadmap task: `docs/planning/implementation-roadmap.md`, `RLM-0002`
* Applicable guidance: HVE Core Markdown, writing-style, and prompt-builder
  instructions

## Implementation Checklist

### Phase A: Owner Decisions

<!-- parallelizable: false -->

* [x] Approve primitive and initial layout profile
* [x] Approve numeric literal suffixes and defaults
* [x] Approve arithmetic, cast, and floating-point semantics
* [x] Approve identity, inference, and coercion boundaries
* [x] Approve slice and constant rules

### Phase B: Normative Specification

<!-- parallelizable: false -->

* [x] Add `docs/specifications/type-system.md`
* [x] Amend numeric-suffix, cast, slice, and public-return syntax text
* [x] Record approved decisions in planning sources
* [x] Map every `REQ-008`, `REQ-009`, and `REQ-016` clause to evidence

### Phase C: Disposable Validation

<!-- parallelizable: false -->

* [x] Add an isolated table oracle under `spikes/type-system/`
* [x] Cover integer boundaries, casts, float classes, identity, coercions, and
  target layout
* [x] Prove debug and release profiles produce the same semantic digest
* [x] Pass formatting, lint, tests, and executable checks

### Phase D: Review and Acceptance

<!-- parallelizable: false -->

* [x] Complete independent language-requirement review
* [x] Complete independent Windows x64 and LLVM review
* [x] Repair all task-scoped findings and rerun strict validation
* [x] Mark `RLM-0002` complete only after explicit owner approval

## Dependencies

* Accepted `RLM-0001` source and syntax contract
* `REQ-008`, `REQ-009`, and `REQ-016`
* Initial `x86_64-pc-windows-msvc` target decision
* Rust stable MSVC toolchain for the isolated oracle

## Success Criteria

* No source-level type, operator, cast, inference, coercion, constant, or
  public-signature behavior remains implicit
* Every scoped type has an explicit initial Windows x64 representation boundary
* Debug and optimized semantics are identical where required
* Production `src/`, root `Cargo.toml`, root `Cargo.lock`, and the existing
  syntax checker remain unchanged
* Generic, ownership, exception, async, sendability, and FFI decisions remain
  assigned to their owning downstream tasks
