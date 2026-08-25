---
title: Realm Implementation Plan
description: Planning index for the Realm language, compiler, runtime, standard library, and developer tooling
ms.date: 2026-08-25
ms.topic: overview
---

## Status

This plan is **Proposed**. It converts the early Realm concept into a staged,
reviewable program of work. It does not make the root README authoritative and
does not accept unresolved decisions by implication.

Production implementation must not begin until the Phase 0 decision gates in
[implementation-roadmap.md](implementation-roadmap.md) are approved. Small,
discardable architectural spikes may begin earlier when their task explicitly
permits it.

## Repository Baseline

The repository is a single Rust 2024 package named `Realm`, version `0.1.0`.
Its only direct dependency is `logos` 0.16.1. The lockfile also contains the
procedural-macro and regular-expression crates pulled in by `logos`.

The current library contains seven token categories plus skipped ASCII spaces.
It prints token results and an LLVM placeholder to standard output. The binary
compiles one hard-coded expression. There is no parser, source manager,
diagnostic model, semantic analysis, LLVM or Inkwell integration, object
emission, linker driver, runtime, standard library, test suite, or package
support. Existing code is therefore a lexer experiment, not an architecture to
preserve.

The concept README is intentionally non-normative. Its examples contain these
contradictions and ambiguities:

* The primitive list omits a comma between `f64` and `bool`, includes `f16`
  without defining target support, and does not list the used `string` type
* Fixed arrays use `[bool, 5]`, dynamic lists use `[i32]`, and repetition uses
  `['d', 3]`, leaving type and value grammar ambiguous
* Mutability, literal inference, integer overflow, tuple indexing, bounds
  behavior, negative indices, slicing, ownership, errors, and concurrency have
  no defined semantics

An older README revision also omitted a semicolon, used dotted indexing,
claimed output `5`, and used `/n` for a newline. The current README has already
corrected those four defects. The `RLM-0001` specifications formalize explicit
semicolons, zero-based square-bracket indexing, negative indexing, and strict
half-open slicing; they remain under review until the task receives owner
approval.

## Planning Documents

* [requirements-and-assumptions.md](requirements-and-assumptions.md) defines
  stable requirements, scope, assumptions, unresolved questions, and terms
* [architecture.md](architecture.md) defines the compiler pipeline, stage
  boundaries, data flow, ABI, runtime, tests, and tooling boundaries
* [concurrency-model.md](concurrency-model.md) compares viable models and
  proposes structured tasks, typed channels, cancellation, and async I/O
* [project-structure.md](project-structure.md) proposes the Rust workspace and
  dependency rules for each crate and major directory
* [implementation-roadmap.md](implementation-roadmap.md) defines phases, the
  executable task backlog, critical path, parallel work, and review checkpoints
* [risk-register.md](risk-register.md) assigns mitigations, triggers, and phase
  ownership to technical and delivery risks
* [decision-log.md](decision-log.md) records lightweight architectural decision
  records (ADRs), including proposed and blocked decisions

## Recommended Direction

Realm should begin as a deterministic batch compiler with explicit boundaries:

```text
UTF-8 source -> tokens -> recoverable syntax -> resolved HIR -> typed HIR
  -> CFG IR -> monomorphized instances -> LLVM IR -> COFF objects
  -> MSVC-compatible linker -> Windows x86-64 executable
```

The compiler should use immutable IDs and data-oriented stage outputs. Syntax
must remain independent of semantic state. LLVM lowering must not perform name
resolution, type inference, borrow checking, or cleanup planning.

The language direction is Rust-like affine ownership with shared and exclusive
borrows, deterministic destruction, local type inference, nominal types,
monomorphized generics, algebraic data types, and exhaustive pattern matching.
Functions that can raise exceptions carry a declared `throws` effect. The
runtime unwinds Windows x64 stacks, runs language cleanups, and terminates on an
uncaught exception.

Concurrency is foundational. The proposed MVP uses scoped structured tasks,
cooperative cancellation, typed channels, timers, and async I/O. Safe code must
be data-race-free through ownership, borrowing, and sendability checks. Raw OS
threads, actors, detached tasks, and general shared-memory atomics are deferred
until the core safety model is validated.

## Evidence Standard

Recommendations distinguish external evidence from Realm-specific judgment:

* **Evidence** describes a contract, precedent, or measured behavior in a cited
  primary source
* **Realm judgment** selects or adapts an approach for this project and remains
  subject to its ADR status
* **Assumption** fills a missing requirement temporarily and must carry a review
  condition

Primary references include the
[LLVM Language Reference](https://llvm.org/docs/LangRef.html),
[LLVM frontend performance guidance](https://llvm.org/docs/Frontend/PerformanceTips.html),
[Inkwell repository](https://github.com/TheDan64/inkwell),
[rustc development guide](https://rustc-dev-guide.rust-lang.org/overview.html),
[rust-analyzer architecture](https://rust-analyzer.github.io/book/contributing/architecture.html),
[Swift structured concurrency proposal](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0304-structured-concurrency.md),
[Go memory model](https://go.dev/ref/mem), and
[Erlang concurrency documentation](https://www.erlang.org/doc/system/conc_prog.html).
Each detailed recommendation cites its closest source in the relevant document.

## Approval Gates

Planning approval requires all of the following:

* Stable requirements have an owner, status, and validation path
* Blocked ADRs have a bounded spike or specification task
* The exact LLVM major and Inkwell release are pinned after a compatibility
  probe
* The ownership, exception-cleanup, and concurrency semantics agree on value
  movement, cancellation, unwinding, and destruction
* The first vertical slice can compile, link, and run without depending on the
  future package manager or full runtime
* Every implementation task has bounded scope, dependencies, acceptance
  criteria, tests, non-goals, risk, and size

## Change Control

Requirement IDs, task IDs, risk IDs, and ADR numbers are stable. Do not reuse a
retired identifier. A changed requirement keeps its ID and records the change in
the decision log. A task whose acceptance criteria change materially should be
superseded by a new task rather than silently widened.

Future implementation agents should execute one task at a time, read its linked
requirements and ADRs, and stop at any unmet decision gate. Estimates use `XS`,
`S`, `M`, and `L` as relative review size, not calendar commitments.
