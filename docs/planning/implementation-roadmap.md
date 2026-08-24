---
title: Realm Implementation Roadmap
description: Phases, dependency-aware task backlog, critical path, validation gates, and implementation checkpoints for Realm
ms.date: 2026-08-23
ms.topic: reference
---

## Execution Rules

This roadmap is planning, not authorization to implement. Production tasks begin
only after Phase 0 approves their prerequisites. Future agents execute one task
at a time and stop when a prerequisite or dependency is incomplete.

Every task ID is stable. `XS`, `S`, `M`, and `L` describe relative change and
review size, not elapsed time. An `L` task must still have one primary behavior;
split it before execution if discovery shows multiple independent outcomes.

The expected paths are proposed in
[project-structure.md](project-structure.md). A task may adjust a path through a
reviewed ADR, but it may not move responsibility across architectural layers
silently.

## Phase Summary

| Phase | Objective | Exit evidence |
|---|---|---|
| Phase 0 | Resolve design blockers with specifications and disposable spikes | Required ADRs accepted and production contracts approved |
| Phase 1 | Build deterministic source, diagnostic, lexical, and syntax foundations | Malformed source parses without panic and diagnostics are stable |
| Phase 2 | Resolve and type-check a multi-module language subset | `realm check` validates names, types, effects, and patterns without LLVM |
| Phase 3 | Produce the first native Windows executable | Source compiles through verified LLVM and links/runs in clean CI |
| Phase 4 | Complete core data, ownership, borrowing, and generics | Safe core language and monomorphization pass compile/run suites |
| Phase 5 | Add exceptions and the narrow C ABI | Cleanup and FFI probes pass at supported optimization levels |
| Phase 6 | Add structured concurrency and Windows async runtime | Race-safety, cancellation, channels, timers, and I/O gates pass |
| Phase 7 | Complete MVP library, packages, diagnostics, tests, and debugging | Product-facing MVP gates pass on representative programs |
| Phase 8 | Harden, specify compatibility, and make a release decision | Reproducibility, robustness, performance, and conformance approved |

## Phase 0: Decisions and Spikes

All Phase 0 code is disposable and lives under a temporary `spikes/` directory
or an isolated branch. It must not become production scaffolding by accident.

### `RLM-0001`: Lexical and Surface Syntax Specification

* Purpose and exact scope: Specify UTF-8 errors, Unicode identifiers, trivia,
  comments, literals, semicolons, declarations, expressions, indexing, negative
  indexing, slicing, and recovery synchronization tokens for `REQ-001`-`REQ-007`
* Expected files/modules: Future `docs/specifications/lexical-grammar.md` and
  `docs/specifications/grammar.md`
* Prerequisites: Requirements approved for review; dependencies: none
* Deliverables and acceptance: Normative grammar plus valid/invalid examples;
  every README contradiction and `QUE-001`-`QUE-003` is resolved or explicitly
  split into a blocking follow-up
* Validation: Review examples with a grammar checker or prototype parser and
  map every syntax requirement to at least one conformance case
* Non-goals: Parser implementation and formatter style; risks: `RISK-002`,
  `RISK-026`; size: `M`

### `RLM-0002`: Primitive and Type-System Specification

* Purpose and exact scope: Specify primitive widths, character/string encoding,
  literal defaults, overflow, casts, nominal identity, local inference,
  coercions, never/unit, arrays, tuples, slices, and public-signature rules
* Expected files/modules: Future `docs/specifications/type-system.md`
* Prerequisites: `RLM-0001`; dependencies: `RLM-0001`
* Deliverables and acceptance: Normative rules and judgment examples covering
  `REQ-008`, `REQ-009`, and `REQ-016`; no backend layout remains implicit
* Validation: Hand-check examples at debug and optimized semantics; independent
  design review against Windows x64 and LLVM type constraints
* Non-goals: Generic constraints, ownership, or type-checker code; risks:
  `RISK-002`, `RISK-012`; size: `M`

### `RLM-0003`: Recoverable Syntax Tree Spike

* Purpose and exact scope: Compare Rowan-style event parsing with one minimal
  Realm-owned lossless tree for trivia retention, error nodes, typed access, and
  memory use
* Expected files/modules: Temporary `spikes/syntax-tree/` and ADR-003 update in
  `docs/planning/decision-log.md`
* Prerequisites: Draft grammar from `RLM-0001`; dependencies: `RLM-0001`
* Deliverables and acceptance: Measured prototype over valid and malformed
  fixtures, recommendation, dependency assessment, and accepted or superseded
  ADR-003
* Validation: Round-trip source bytes, recover at least three independent errors,
  and record bytes-per-source-byte on a representative fixture
* Non-goals: Production parser or full grammar; risks: `RISK-003`, `RISK-021`;
  size: `M`

### `RLM-0004`: Generic Constraint Design

* Purpose and exact scope: Define generic parameter syntax, minimum constraints,
  obligation lookup, coherence boundary, substitution, and recursive instance
  limits without designing a broad trait system
* Expected files/modules: Generic section of future
  `docs/specifications/type-system.md` and ADR-005 update
* Prerequisites: `RLM-0002`; dependencies: `RLM-0002`
* Deliverables and acceptance: `QUE-005` resolved for generic identity,
  containers, equality/ordering examples, and constrained body checking
* Validation: Manually derive checking and monomorphization for positive,
  ambiguous, unsatisfied, and infinitely recursive examples
* Non-goals: Dynamic dispatch, specialization, associated types, or macros;
  risks: `RISK-006`, `RISK-014`; size: `M`

### `RLM-0005`: Ownership and Cleanup Model

* Purpose and exact scope: Define places, moves, copy eligibility, shared and
  exclusive loans, reborrows, lifetime end points, partial initialization,
  destruction order, unsafe boundary, and CFG dataflow
* Expected files/modules: Future `docs/specifications/ownership.md` and temporary
  `spikes/borrow-dataflow/`
* Prerequisites: `RLM-0002`; dependencies: `RLM-0002`
* Deliverables and acceptance: `REQ-010`-`REQ-014` rules, `QUE-004` disposition,
  and a branch/loop prototype rejecting known alias and move defects
* Validation: Positive and negative tables plus prototype dataflow over at least
  branch, loop, early return, reborrow, and partial-initialization CFGs
* Non-goals: Production borrow checker, shared ownership, or concurrency;
  risks: `RISK-007`, `RISK-015`; size: `L`

### `RLM-0006`: Windows Exception ABI Spike

* Purpose and exact scope: Prototype nominal exception payload, Windows x64
  unwinding, cleanup execution, catch at a task-like trampoline, C-boundary
  prohibition, double-fault policy, and optimization behavior
* Expected files/modules: Temporary `spikes/windows-unwind/`, future
  `docs/specifications/exceptions.md`, and ADR-006 update
* Prerequisites: `RLM-0005` cleanup order and candidate LLVM pair; dependencies:
  `RLM-0005`, `RLM-0011`
* Deliverables and acceptance: `QUE-006` resolved and native probe runs cleanup
  exactly once at `-O0` and optimized settings without unwinding through C
* Validation: Build/run probe on clean Windows CI and inspect unwind/debug data
* Non-goals: Production exception syntax or runtime; risks: `RISK-005`,
  `RISK-015`, `RISK-016`; size: `L`

### `RLM-0007`: Sendability and Memory-Model Specification

* Purpose and exact scope: Define structural `Send`/`Sync`, negative cases,
  capture modes, borrows across suspension, data race, synchronizes-with, and
  happens-before for tasks and channels
* Expected files/modules: Future `docs/specifications/memory-model.md` and
  concurrency type-system sections; ADR-008 update
* Prerequisites: `RLM-0005` and the selected concurrency direction;
  dependencies: `RLM-0005`
* Deliverables and acceptance: `REQ-015`, `REQ-046`, and `QUE-007` resolved with
  positive, compile-fail, and runtime litmus examples
* Validation: Independent soundness review and mapping of each synchronization
  operation to LLVM-compatible ordering
* Non-goals: User atomics, lock APIs, or implementation; risks: `RISK-008`,
  `RISK-017`; size: `L`

### `RLM-0008`: Task Runtime and Async I/O Spike

* Purpose and exact scope: Compare Realm-owned state machines with LLVM
  coroutines; prototype deterministic execution, task wakeup, one timer, one
  Windows overlapped I/O operation, cancellation, and task metadata
* Expected files/modules: Temporary `spikes/async-runtime/`, ADR-007 update, and
  `QUE-008` disposition
* Prerequisites: `RLM-0005`, draft `RLM-0007`, and `RLM-0011`; dependencies:
  `RLM-0005`, `RLM-0007`, `RLM-0011`
* Deliverables and acceptance: Measured lowering/runtime recommendation; no lost
  wakeup, use-after-free, or double completion in scripted race tests
* Validation: Deterministic schedules, parallel stress, clean shutdown, and
  source suspension metadata inspection
* Non-goals: Production scheduler, channels, networking API, or worker tuning;
  risks: `RISK-009`, `RISK-017`, `RISK-018`; size: `L`

### `RLM-0009`: Channel Semantics Specification

* Purpose and exact scope: Decide bounded capacity, optional rendezvous,
  sender/receiver capabilities, close/drop, ordering, fairness, cancellation
  races, buffered destruction, and selection deferral
* Expected files/modules: Channel section of future
  `docs/specifications/concurrency.md` and ADR-007 note
* Prerequisites: `RLM-0007` and runtime findings from `RLM-0008`; dependencies:
  `RLM-0007`, `RLM-0008`
* Deliverables and acceptance: `REQ-045` and `QUE-009` resolved with state tables
  for every send/receive/close/cancel combination
* Validation: Review state machine against ownership transfer and happens-before
  examples
* Non-goals: Channel implementation or multi-channel selection; risks:
  `RISK-019`; size: `M`

### `RLM-0010`: Module and Package Specification

* Purpose and exact scope: Specify file-to-module mapping, namespaces,
  visibility, imports, cycle errors, pure initialization, manifest schema, local
  paths, immutable remote identity, cache, trust, offline behavior, and updates
* Expected files/modules: Future
  `docs/specifications/modules-and-packages.md` and ADR-009/ADR-010 updates
* Prerequisites: `RLM-0001`; dependencies: `RLM-0001`
* Deliverables and acceptance: `REQ-027`-`REQ-033`, `QUE-010`, and `QUE-011`
  resolved; local-only implementation subset clearly marked
* Validation: Resolve sample DAGs, full-cycle diagnostics, pure/impure initializers,
  and content-identity scenarios on paper
* Non-goals: Registry, version ranges, lockfile, or network implementation;
  risks: `RISK-020`, `RISK-026`; size: `M`

### `RLM-0011`: LLVM and Inkwell Compatibility Spike

* Purpose and exact scope: Test candidate released pairs for Windows target/data
  layout, verification, optimization, COFF, CodeView/PDB input, exception APIs,
  coroutine APIs, installation, and clean CI linking
* Expected files/modules: Temporary `spikes/llvm-compat/`, CI experiment, and
  ADR-011 update
* Prerequisites: Windows MSVC toolchain available; dependencies: none
* Deliverables and acceptance: One exact pair, installation recipe, feature
  flags, known gaps, and accepted ADR-011 resolving `QUE-013`
* Validation: Clean-machine probe emits, links, runs, verifies, and exposes a
  debugger line mapping
* Non-goals: Production backend or multi-version support; risks: `RISK-004`,
  `RISK-013`, `RISK-023`; size: `M`

### `RLM-0012`: Phase 0 Approval and Architecture Baseline

* Purpose and exact scope: Reconcile all spike evidence, accept/supersede ADRs,
  approve specifications, confirm deferred scope, and establish change control
* Expected files/modules: Planning documents, future specifications, and ADR
  status updates only
* Prerequisites: `RLM-0001`-`RLM-0011`; dependencies: all earlier Phase 0 tasks
* Deliverables and acceptance: ADR-001 through ADR-015 have actionable status;
  no production task has an unresolved semantic or toolchain prerequisite
* Validation: Requirement-to-task and risk-to-mitigation review by language,
  compiler, runtime, and Windows ABI owners
* Non-goals: Production code or project scaffolding; risks: `RISK-001`,
  `RISK-026`, `RISK-028`; size: `M`

### Phase 0 Checkpoint

Stop if ADR-003, ADR-004, ADR-005, ADR-006, ADR-007, ADR-008, ADR-011, or
ADR-012 is not accepted with evidence. Remote package work may remain blocked;
local package semantics may proceed under an accepted subset of ADR-010.

## Phase 1: Source and Syntax Foundations

### `RLM-0101`: Bootstrap the Rust Workspace

* Purpose and exact scope: Convert the root package to the approved workspace,
  pin Rust, add baseline lints/format/test configuration, and preserve or retire
  the lexer experiment explicitly
* Expected files/modules: Root `Cargo.toml`, `Cargo.lock`, future
  `rust-toolchain.toml`, initial crate manifests, and CI build definition
* Prerequisites: `RLM-0012`; dependencies: `RLM-0012`
* Deliverables and acceptance: Empty minimal crates required by this phase build
  without cycles; current behavior is captured or deliberately removed
* Validation: Formatting, lint, build, unit test, dependency-tree, and clean CI
  commands pass on Windows
* Non-goals: Creating all future crates or adding LLVM; risks: `RISK-021`,
  `RISK-028`; size: `M`

### `RLM-0102`: Implement Base IDs and Source Management

* Purpose and exact scope: Add typed IDs, immutable source snapshots, UTF-8
  validation, byte ranges, logical paths, line indices, and injected loading
* Expected files/modules: `compiler/realm-base`, `compiler/realm-source`
* Prerequisites: Accepted source coordinate policy; dependencies: `RLM-0101`
* Deliverables and acceptance: Multiple files retain stable session IDs and all
  valid byte offsets map correctly to line/display positions
* Validation: Unit/property tests for empty, CRLF, Unicode, invalid UTF-8, and
  boundary offsets
* Non-goals: Lexer or filesystem walking; risks: `RISK-010`, `RISK-011`; size:
  `S`

### `RLM-0103`: Implement Structured Diagnostics

* Purpose and exact scope: Add diagnostic values, labels, notes, edits,
  applicability, stable ordering, human rendering, and source snippets
* Expected files/modules: `compiler/realm-diagnostics`
* Prerequisites: Accepted diagnostic schema; dependencies: `RLM-0102`
* Deliverables and acceptance: Multi-file diagnostics render deterministically
  with primary/secondary labels and no direct process output from the crate
* Validation: Structured assertions and snapshots under changed path roots,
  Unicode, CRLF, overlapping labels, and unavailable source
* Non-goals: JSON schema and feature-specific messages; risks: `RISK-022`,
  `RISK-024`; size: `M`

### `RLM-0104`: Implement the Production Lexer

* Purpose and exact scope: Tokenize the accepted lexical grammar, retain trivia,
  separate literal cooking, and recover from invalid characters and escapes
* Expected files/modules: `compiler/realm-lexer`, `tests/parse/lexer`
* Prerequisites: ADR-003 and lexical specification accepted; dependencies:
  `RLM-0102`, `RLM-0103`
* Deliverables and acceptance: Every lexical form and malformed class produces
  specified token ranges and diagnostics without panic
* Validation: Fixture tests, source-range properties, arbitrary-byte fuzzing,
  and deterministic output across runs
* Non-goals: Parsing or semantic literal typing; risks: `RISK-010`, `RISK-024`;
  size: `M`

### `RLM-0105`: Implement Syntax Infrastructure and Expressions

* Purpose and exact scope: Add syntax kinds, tree/event infrastructure, typed
  AST wrappers, Pratt expression parsing, blocks, calls, fields, indexing, and
  slicing syntax
* Expected files/modules: `compiler/realm-syntax`, `tests/parse/expressions`
* Prerequisites: ADR-003 and grammar accepted; dependencies: `RLM-0104`
* Deliverables and acceptance: Expression fixtures round-trip losslessly and
  precedence/associativity match the specification
* Validation: CST snapshots plus structural AST assertions and malformed
  expression recovery cases
* Non-goals: Name resolution or indexing semantics; risks: `RISK-003`,
  `RISK-010`; size: `M`

### `RLM-0106`: Implement Items, Statements, Types, and Patterns

* Purpose and exact scope: Parse modules, imports, visibility, functions,
  nominal types, generic syntax, bindings, control flow, type references, and
  patterns
* Expected files/modules: `compiler/realm-syntax`, `tests/parse/items`,
  `tests/parse/statements`, `tests/parse/patterns`
* Prerequisites: Accepted full grammar; dependencies: `RLM-0105`
* Deliverables and acceptance: Every MVP grammar production has typed AST access
  and at least one valid and invalid fixture
* Validation: Grammar coverage inventory, CST snapshots, and source round trips
* Non-goals: Semantic validation or macro syntax; risks: `RISK-002`,
  `RISK-010`; size: `M`

### `RLM-0107`: Harden Parser Recovery and Fuzzing

* Purpose and exact scope: Add synchronization sets, missing-token strategy,
  diagnostic cascade limits, parser progress assertions, corpus seeds, and fuzz
  harnesses
* Expected files/modules: `compiler/realm-syntax`, `tests/parse/recovery`, fuzz
  workspace or CI target
* Prerequisites: Complete MVP parser; dependencies: `RLM-0106`
* Deliverables and acceptance: Arbitrary token streams terminate, preserve all
  source bytes, and never emit unbounded duplicate diagnostics
* Validation: Time-bounded fuzz campaign, regression corpus, and malformed-file
  memory/time limits
* Non-goals: Semantic recovery or performance optimization; risks: `RISK-010`,
  `RISK-024`; size: `M`

### `RLM-0108`: Add Check CLI and Compiler Test Harness

* Purpose and exact scope: Add thin CLI argument parsing, source loading,
  parse-only `check`, stage dumps, exit codes, fixture discovery, and normalized
  structured expectations
* Expected files/modules: `tools/realm-cli`, `compiler/realm-driver`,
  `compiler/realm-test-support`, top-level `tests/`
* Prerequisites: CLI contract approved; dependencies: `RLM-0103`, `RLM-0107`
* Deliverables and acceptance: `realm check` reports parse diagnostics for one
  file and the harness runs pass/fail fixtures deterministically
* Validation: End-to-end CLI tests for success, source error, missing file,
  stage dump, and internal-error exit paths
* Non-goals: Semantic checking, packages, or native output; risks: `RISK-011`,
  `RISK-022`, `RISK-024`; size: `M`

### Phase 1 Checkpoint

Review syntax API independence, diagnostic quality, fuzz results, deterministic
ordering, and crate dependency rules. No semantic crate proceeds while malformed
source can panic or tree recovery can fail to make progress.

## Phase 2: Names and Types

### `RLM-0201`: Implement Local Manifest and Module Graphs

* Purpose and exact scope: Parse manifests, derive file modules, validate local
  exact dependencies, order DAGs, reject cycles, and expose an abstract package
  graph
* Expected files/modules: `compiler/realm-package`, `tests/packages/local`
* Prerequisites: Accepted local subset of ADR-010; dependencies: `RLM-0108`
* Deliverables and acceptance: Multi-package local DAG loads deterministically;
  cycle diagnostic prints the complete cycle
* Validation: Manifest schema, path normalization, duplicate, missing package,
  visibility-root, and cycle fixtures
* Non-goals: HTTPS, Git, cache, or version solving; risks: `RISK-011`,
  `RISK-020`; size: `M`

### `RLM-0202`: Define HIR and Lower Syntax

* Purpose and exact scope: Add semantic IDs, HIR arenas, source maps, desugared
  block/item/body structures, unresolved path slots, and syntax-to-HIR lowering
* Expected files/modules: `compiler/realm-hir`, lowering entry in
  `compiler/realm-resolve`
* Prerequisites: HIR contract approved; dependencies: `RLM-0106`, `RLM-0201`
* Deliverables and acceptance: Valid and erroneous syntax lower without storing
  CST nodes or third-party tree values in HIR
* Validation: Structural HIR snapshots and source-origin assertions for every
  expression and pattern category
* Non-goals: Name binding or typing; risks: `RISK-010`, `RISK-029`; size: `M`

### `RLM-0203`: Implement Item Collection and Name Resolution

* Purpose and exact scope: Allocate definitions, build scopes/namespaces,
  process imports/visibility, bind type/value paths, and preserve error sentinels
* Expected files/modules: `compiler/realm-resolve`, `tests/ui/resolve`
* Prerequisites: Namespace and visibility rules accepted; dependencies:
  `RLM-0202`
* Deliverables and acceptance: Multi-module definitions resolve by stable ID;
  duplicate, missing, private, ambiguous, and cyclic uses diagnose correctly
* Validation: Compile-pass/fail fixtures and deterministic definition-table/HIR
  assertions
* Non-goals: Overload resolution, wildcard imports, or re-exports; risks:
  `RISK-010`, `RISK-011`; size: `M`

### `RLM-0204`: Implement Canonical Types and Substitutions

* Purpose and exact scope: Add canonical primitive, tuple, array, slice,
  function, nominal, generic, inference-placeholder, and error types plus effects
  and substitutions
* Expected files/modules: `compiler/realm-types`
* Prerequisites: `RLM-0002`, `RLM-0004`; dependencies: `RLM-0101`, `RLM-0202`
* Deliverables and acceptance: Equivalent types intern identically and formatting
  is deterministic without LLVM target types
* Validation: Unit/property tests for interning, substitution, cycles, error
  values, and stable formatting
* Non-goals: Inference algorithm or physical layout; risks: `RISK-006`,
  `RISK-011`; size: `S`

### `RLM-0205`: Implement Local Type Inference and Checking

* Purpose and exact scope: Add inference variables, unification, literal
  defaults, explicit coercions, binding/assignment/call/return checks, and typed
  expression results
* Expected files/modules: `compiler/realm-typeck`, `tests/ui/typeck`
* Prerequisites: Accepted primitive/type rules; dependencies: `RLM-0203`,
  `RLM-0204`
* Deliverables and acceptance: Function bodies infer locals but reject omitted
  public signature types, invalid coercions, assignments, and calls
* Validation: Positive/negative fixtures plus unification rollback and error
  recovery unit tests
* Non-goals: Generic obligations, borrowing, or LLVM layout; risks: `RISK-006`,
  `RISK-010`; size: `L`

### `RLM-0206`: Implement Functions and Declared Effects

* Purpose and exact scope: Validate signatures, calls, async marker, Boolean
  `throws` propagation, catch context, return completeness input, and effect
  diagnostics
* Expected files/modules: `compiler/realm-typeck`, `tests/ui/effects`
* Prerequisites: Accepted exception surface specification; dependencies:
  `RLM-0205`
* Deliverables and acceptance: Calls to throwing functions require propagation
  or catch; non-throwing and async signature mismatches are rejected
* Validation: Effect matrix fixtures including nested calls, generics placeholders,
  catches, and error sentinels
* Non-goals: Unwinding, task execution, or exception payload lowering; risks:
  `RISK-005`, `RISK-010`; size: `M`

### `RLM-0207`: Implement Nominal ADTs and Pattern Typing

* Purpose and exact scope: Collect structures and sum variants, validate fields
  and constructors, type literal/tuple/structure/variant patterns, and type match
  arms
* Expected files/modules: `compiler/realm-resolve`, `compiler/realm-typeck`,
  `tests/ui/adts`
* Prerequisites: ADT and pattern rules accepted; dependencies: `RLM-0205`
* Deliverables and acceptance: Nominally distinct equal-shape types remain
  incompatible and payload patterns bind correct types
* Validation: Constructor, privacy, duplicate-field, wrong-variant, and nested
  pattern fixtures
* Non-goals: Runtime layout or exhaustiveness algorithm; risks: `RISK-002`,
  `RISK-012`; size: `M`

### `RLM-0208`: Implement Exhaustiveness and Reachability

* Purpose and exact scope: Add pattern-space analysis for Boolean, finite
  integer literals, tuples, structures, and ADTs with guards handled conservatively
* Expected files/modules: `compiler/realm-typeck`, `tests/ui/patterns`
* Prerequisites: Pattern semantics accepted; dependencies: `RLM-0207`
* Deliverables and acceptance: Missing witnesses and unreachable arms receive
  stable diagnostics without relying on code generation
* Validation: Matrix of nested sums/products, wildcard, duplicate, guard, and
  pathological-size fixtures
* Non-goals: Regex patterns or optimization; risks: `RISK-010`, `RISK-025`;
  size: `M`

### Phase 2 Checkpoint

`realm check` must analyze a local multi-module program through typed HIR. Review
all error sentinels, diagnostic cascades, HIR/type crate boundaries, and timing
baselines before introducing CFG or LLVM.

## Phase 3: First Native Vertical Slice

### `RLM-0301`: Define and Lower Basic CFG IR

* Purpose and exact scope: Add locals, places, constants, assignment, arithmetic,
  calls, branches, loops, return, unreachable, validation, and typed-HIR lowering
* Expected files/modules: `compiler/realm-cfg`, basic lowering in
  `compiler/realm-analysis`, `tests/codegen/cfg`
* Prerequisites: CFG contract accepted; dependencies: `RLM-0205`, `RLM-0206`
* Deliverables and acceptance: Basic functions lower to valid explicit CFG with
  no syntax traversal and all blocks terminated
* Validation: Structural CFG assertions, malformed-internal CFG validator tests,
  and return/unreachable dataflow cases
* Non-goals: Borrows, drops, exceptions, async, ADT layout, or generics; risks:
  `RISK-012`, `RISK-029`; size: `M`

### `RLM-0302`: Define Runtime ABI and Minimal Native Runtime

* Purpose and exact scope: Add ABI version/symbol declarations, process startup,
  console output primitive, controlled fatal error, and shutdown
* Expected files/modules: `compiler/realm-runtime-abi`,
  `runtime/realm-runtime/{src/lib.rs,src/memory,src/platform/windows}`
* Prerequisites: ADR-001 and ABI baseline accepted; dependencies: `RLM-0101`
* Deliverables and acceptance: Rust static library exports only declared
  C-compatible symbols and ABI layout tests pass
* Validation: Symbol inspection, C header/static assertions, Rust unit tests, and
  native smoke executable
* Non-goals: Exceptions, owned collections, tasks, or async I/O; risks:
  `RISK-012`, `RISK-016`, `RISK-029`; size: `M`

### `RLM-0303`: Implement Minimal LLVM Code Generation

* Purpose and exact scope: Initialize selected target, map primitive/function
  types, lower basic CFG, declare runtime calls, verify modules, and emit COFF
* Expected files/modules: `compiler/realm-codegen-llvm`
* Prerequisites: ADR-011 accepted and toolchain installed; dependencies:
  `RLM-0301`, `RLM-0302`
* Deliverables and acceptance: Deterministic verified LLVM modules and COFF
  objects for constants, arithmetic, branches, calls, and return
* Validation: Pre/post-optimization verification, focused IR assertions, object
  inspection, and native semantic comparisons at `-O0` and optimized mode
* Non-goals: Aggregates, generics, debug locals, exceptions, or coroutines;
  risks: `RISK-004`, `RISK-012`; size: `L`

### `RLM-0304`: Implement the MSVC Linker Adapter

* Purpose and exact scope: Add explicit toolchain configuration, discovery,
  pure command construction, response files, runtime/system inputs, invocation,
  and translated linker diagnostics
* Expected files/modules: `compiler/realm-link`, `tests/abi/linker`
* Prerequisites: ADR-012 accepted; dependencies: `RLM-0302`, `RLM-0303`
* Deliverables and acceptance: COFF object links on clean CI and explicit
  override works with paths containing spaces
* Validation: Command unit tests, missing/incompatible toolchain failures, and
  end-to-end link/run smoke test
* Non-goals: Embedded LLD, cross-linking, or package build graph; risks:
  `RISK-013`; size: `M`

### `RLM-0305`: Orchestrate Native Builds

* Purpose and exact scope: Extend build requests through semantic check, CFG,
  LLVM, object, link, atomic artifact publication, stage stops, and temporary
  cleanup
* Expected files/modules: `compiler/realm-driver`, `tools/realm-cli`
* Prerequisites: Stable stage APIs; dependencies: `RLM-0303`, `RLM-0304`
* Deliverables and acceptance: `realm build` emits an executable only after all
  stages succeed and reports reproducible commands/artifacts on failure
* Validation: End-to-end success, each stage-stop, source error, verifier error,
  linker error, stale-output, and interrupted-build tests
* Non-goals: Incremental builds, packages beyond one root, or parallel passes;
  risks: `RISK-011`, `RISK-012`; size: `M`

### `RLM-0306`: Validate First Executable and Debug Line Table

* Purpose and exact scope: Add representative arithmetic/control/call program,
  console result, source file/line debug metadata, and clean-machine evidence
* Expected files/modules: `tests/run-pass/first-executable`,
  `tests/debug-info/line-table`, CI workflow
* Prerequisites: Native build path complete; dependencies: `RLM-0305`
* Deliverables and acceptance: Program builds, runs with expected output/exit,
  and symbol tooling maps generated instructions to Realm source lines
* Validation: Run at supported optimization levels on clean Windows CI and
  inspect object/executable debug records
* Non-goals: Local variable debugging or async stacks; risks: `RISK-012`,
  `RISK-023`; size: `S`

### Phase 3 Checkpoint

The first vertical slice must be reproducible from documented prerequisites.
Review generated IR mechanically against CFG, linker diagnostics, artifact
publication, platform leakage, and debug line truth before adding language
breadth.

## Phase 4: Ownership, Data, and Generics

### `RLM-0401`: Implement Aggregate Layout and ADT Code Generation

* Purpose and exact scope: Specify target layout queries and lower tuples,
  arrays, structures, ADT discriminants/payloads, construction, projection, and
  pattern branches
* Expected files/modules: `compiler/realm-types`, `compiler/realm-cfg`,
  `compiler/realm-codegen-llvm`, `tests/codegen/aggregates`
* Prerequisites: Accepted ABI/layout specification; dependencies: `RLM-0208`,
  `RLM-0306`
* Deliverables and acceptance: Native aggregate and ADT programs match specified
  layout-independent behavior and internal layout validation
* Validation: Run-pass programs, LLVM assertions, size/alignment probes, and
  invalid-discriminant unsafe tests where applicable
* Non-goals: Stable Realm-native ABI or C aggregate export; risks: `RISK-012`,
  `RISK-016`; size: `L`

### `RLM-0402`: Implement Definite Initialization and Move Analysis

* Purpose and exact scope: Track initialized places, partial moves, assignments,
  branch joins, loops, call argument moves, and use-after-move diagnostics
* Expected files/modules: `compiler/realm-analysis`, `tests/ui/moves`
* Prerequisites: Ownership specification accepted; dependencies: `RLM-0301`,
  `RLM-0401`
* Deliverables and acceptance: `REQ-010` move and initialization examples pass
  or fail at CFG level with precise source labels
* Validation: Compile-fail matrix for branch/loop/field/partial moves and
  property tests for dataflow convergence
* Non-goals: Borrow conflicts or cleanup insertion; risks: `RISK-007`,
  `RISK-015`; size: `M`

### `RLM-0403`: Implement Borrow Conflict Analysis

* Purpose and exact scope: Create loans for shared/exclusive borrows, model
  place overlap, mutation, moves, reborrows, call use, and conservative lexical
  loan extents
* Expected files/modules: `compiler/realm-analysis`, `tests/ui/borrows`
* Prerequisites: Accepted loan/conflict rules; dependencies: `RLM-0402`
* Deliverables and acceptance: Shared reads coexist, exclusive access excludes
  competitors, and all known aliasing violations are rejected
* Validation: Positive/negative place-projection and call fixtures plus model
  examples from the ownership specification
* Non-goals: Non-lexical end points, suspension, or unsafe pointers; risks:
  `RISK-007`; size: `L`

### `RLM-0404`: Add Non-Lexical Lifetimes and Cleanup Elaboration

* Purpose and exact scope: Derive loan end points from liveness/control flow,
  register initialized-value cleanup, and insert reverse-order drops on normal
  CFG exits
* Expected files/modules: `compiler/realm-analysis`, `compiler/realm-cfg`,
  `tests/ui/nll`, `tests/run-pass/drop`
* Prerequisites: Cleanup model accepted; dependencies: `RLM-0403`
* Deliverables and acceptance: Sequential reuse after last borrow works and each
  initialized owned value drops exactly once on normal/early/loop exit
* Validation: Compile-pass/fail NLL fixtures and instrumented destructor counters
  over every normal edge
* Non-goals: Exception or cancellation cleanup; risks: `RISK-007`,
  `RISK-015`; size: `L`

### `RLM-0405`: Implement Strings, Vectors, Indexing, and Slicing Semantics

* Purpose and exact scope: Add owned UTF-8 string/vector/slice representations,
  borrow-aware indexing, negative index translation, slicing, bounds behavior,
  and minimum iteration
* Expected files/modules: `library/core`, `library/std`, relevant type/CFG/codegen
  modules, `tests/run-pass/collections`, `tests/run-fail/bounds`
* Prerequisites: `QUE-002` and `QUE-003` resolved; dependencies: `RLM-0404`
* Deliverables and acceptance: Owned and borrowed collection programs match all
  specified bounds, mutability, and lifetime cases
* Validation: Boundary/property tests including empty, Unicode string boundaries,
  negative limits, mutable slice conflicts, and optimized builds
* Non-goals: General collection suite or regex; risks: `RISK-002`, `RISK-015`,
  `RISK-025`; size: `L`

### `RLM-0406`: Implement Generic Obligation Checking

* Purpose and exact scope: Represent accepted constraints, check generic bodies
  once, resolve operations through obligations, and report unsatisfied or
  ambiguous constraints
* Expected files/modules: `compiler/realm-types`, `compiler/realm-typeck`,
  `tests/ui/generics`
* Prerequisites: ADR-005 and generic specification accepted; dependencies:
  `RLM-0205`, `RLM-0401`
* Deliverables and acceptance: Generic functions and nominal types check without
  dependence on currently reachable concrete instances
* Validation: Positive/negative generic bodies, nested substitutions,
  ambiguity, and constraint-visibility fixtures
* Non-goals: Dynamic dispatch, specialization, or associated types; risks:
  `RISK-006`; size: `L`

### `RLM-0407`: Implement Monomorphization and Symbol Mangling

* Purpose and exact scope: Collect roots, canonicalize instances, substitute CFG,
  deduplicate, detect expansion, assign deterministic symbols, and feed codegen
* Expected files/modules: `compiler/realm-mono`, `compiler/realm-codegen-llvm`,
  `tests/codegen/generics`
* Prerequisites: Generic checking and CFG substitution defined; dependencies:
  `RLM-0406`, `RLM-0303`
* Deliverables and acceptance: Each reachable concrete instance emits once,
  unreachable instances do not emit, and recursion diagnostics show instance chain
* Validation: Instance graph assertions, symbol/object inspection, native generic
  programs, and deterministic rebuild comparison
* Non-goals: Shape sharing or stable symbol ABI; risks: `RISK-014`,
  `RISK-025`; size: `L`

### `RLM-0408`: Establish Compiler and Runtime Baselines

* Purpose and exact scope: Measure per-stage time/memory, diagnostics throughput,
  LLVM time, object/binary size, generic instance counts, startup, and collection
  operations
* Expected files/modules: `benchmarks/compiler`, `benchmarks/runtime`, benchmark
  runner configuration
* Prerequisites: Representative core programs exist; dependencies: `RLM-0405`,
  `RLM-0407`
* Deliverables and acceptance: Reproducible baseline report with machine/toolchain
  metadata and provisional regression thresholds
* Validation: Repeated runs with variance reported and benchmark smoke execution
  in CI
* Non-goals: Optimization or release budgets; risks: `RISK-014`, `RISK-025`;
  size: `S`

### Phase 4 Checkpoint

Review ownership soundness, cleanup exactly once, generic diagnostics, instance
growth, collection semantics, and measured baseline. Exceptions cannot proceed
until normal cleanup is demonstrably correct.

## Phase 5: Exceptions and C Interoperability

### `RLM-0501`: Add Exceptional CFG and Cleanup Paths

* Purpose and exact scope: Add throw, catch dispatch, normal/unwind call edges,
  cleanup scopes, handler selection, and reverse-order cleanup on exceptional exit
* Expected files/modules: `compiler/realm-cfg`, `compiler/realm-analysis`,
  `tests/codegen/exceptions`
* Prerequisites: Accepted exception specification and normal cleanup; dependencies:
  `RLM-0404`, `RLM-0206`
* Deliverables and acceptance: CFG validation proves every throwing call has an
  unwind successor and initialized values have one cleanup path
* Validation: Structural CFG tests for nested catch, rethrow, early return, and
  exception during partial construction
* Non-goals: Native unwind emission or task boundaries; risks: `RISK-005`,
  `RISK-015`; size: `L`

### `RLM-0502`: Implement Exception Runtime Support

* Purpose and exact scope: Add exception allocation/type identity, raise/catch
  bridge, task-compatible boundary trampoline, uncaught reporting, and
  double-fault policy
* Expected files/modules: `runtime/realm-runtime/src/exception`,
  `compiler/realm-runtime-abi`
* Prerequisites: `RLM-0006` evidence accepted; dependencies: `RLM-0302`
* Deliverables and acceptance: Runtime exports match ABI and native probes retain
  payload/type identity through catch and uncaught termination
* Validation: ABI symbol/layout tests, fault injection, leak checks, and clean
  optimized Windows runs
* Non-goals: Compiler lowering or foreign exception support; risks: `RISK-005`,
  `RISK-016`; size: `L`

### `RLM-0503`: Lower Throw, Catch, and Unwinding to LLVM

* Purpose and exact scope: Emit personality and Windows exception constructs,
  landing/cleanup paths, catch dispatch, payload transfer, and task boundary stop
* Expected files/modules: `compiler/realm-codegen-llvm`,
  `tests/run-pass/exceptions`, `tests/run-fail/exceptions`
* Prerequisites: Backend exception mechanism fixed; dependencies: `RLM-0501`,
  `RLM-0502`
* Deliverables and acceptance: Typed throw/catch/rethrow programs run cleanup
  exactly once at supported optimization levels
* Validation: LLVM verification, IR assertions, destructor counters, debugger
  stack inspection, and uncaught exit tests
* Non-goals: C++ exceptions or cancellation lowering; risks: `RISK-005`,
  `RISK-012`, `RISK-015`; size: `L`

### `RLM-0504`: Implement Narrow C ABI Types and Calls

* Purpose and exact scope: Add `extern C` declarations/exports, primitive and
  pointer mapping, explicit C-layout aggregates, unsafe calls, callbacks, and
  ownership/lifetime diagnostics
* Expected files/modules: Type/CFG/codegen ABI modules, `tests/abi/c`, generated
  probe headers where needed
* Prerequisites: Accepted ABI specification; dependencies: `RLM-0401`,
  `RLM-0503`
* Deliverables and acceptance: Realm calls C and C calls Realm for every allowed
  type; unsupported types and escaping exceptions are rejected
* Validation: Bidirectional C probes, static size/alignment assertions, callback
  lifetime tests, optimized runs, and symbol inspection
* Non-goals: C++, Rust ABI, automatic binding generation, or broad pointer API;
  risks: `RISK-016`; size: `L`

### `RLM-0505`: Validate Error, Cleanup, and FFI Boundaries

* Purpose and exact scope: Integrate nested ownership, partial initialization,
  throws, callbacks, foreign failures, uncaught reporting, and C boundary guards
* Expected files/modules: `tests/abi/exceptions`, `tests/run-pass/cleanup`,
  `tests/run-fail/cleanup`
* Prerequisites: Exception and C ABI implementation complete; dependencies:
  `RLM-0503`, `RLM-0504`
* Deliverables and acceptance: Conformance matrix passes without leak, double
  drop, foreign unwind, invalid exit code, or misleading diagnostic
* Validation: Native tests at all supported optimization levels plus available
  sanitizer/runtime instrumentation
* Non-goals: Async exceptions or blocking-pool behavior; risks: `RISK-005`,
  `RISK-015`, `RISK-016`; size: `M`

### Phase 5 Checkpoint

Accept native exception and C ABI behavior only after clean CI evidence and ABI
review. Concurrency implementation cannot use exceptions until the task boundary
capture behavior is proven.

## Phase 6: Structured Concurrency

### `RLM-0601`: Lower Async Functions to Task Frames

* Purpose and exact scope: Add suspension/resume CFG, liveness-based frame fields,
  state discriminant, wait registration calls, result state, and cleanup metadata
* Expected files/modules: `compiler/realm-cfg`, `compiler/realm-analysis`, async
  lowering in `compiler/realm-codegen-llvm`, `tests/codegen/async`
* Prerequisites: ADR-007 and lowering choice accepted; dependencies: `RLM-0503`
* Deliverables and acceptance: An async function suspends/resumes multiple times
  without losing values and runs frame cleanup once
* Validation: State-machine structural tests, native scripted wakeups, throw and
  early-drop cases, and debug suspension metadata check
* Non-goals: Parallel executor, scopes, channels, or I/O; risks: `RISK-009`,
  `RISK-015`; size: `L`

### `RLM-0602`: Enforce Sendability and Suspension Borrow Rules

* Purpose and exact scope: Derive `Send`/`Sync`, classify task captures, reject
  escaping/non-sendable values, and validate shared/mutable borrows across suspend
* Expected files/modules: `compiler/realm-types`, `compiler/realm-typeck`,
  `compiler/realm-analysis`, `tests/ui/concurrency`
* Prerequisites: ADR-008 and memory model accepted; dependencies: `RLM-0403`,
  `RLM-0601`
* Deliverables and acceptance: All specification positive cases compile and all
  race/escape/suspension counterexamples fail before codegen
* Validation: Compile-fail matrix and type-property derivation tests for nested
  aggregates, raw/foreign handles, and task frames
* Non-goals: User atomics, mutexes, or actors; risks: `RISK-008`; size: `L`

### `RLM-0603`: Implement the Deterministic Executor

* Purpose and exact scope: Add task state, ready queue, wake protocol, root task,
  single-thread polling, scripted scheduling, virtual clock hook, and shutdown
* Expected files/modules: `runtime/realm-runtime/src/task`, runtime ABI, runtime
  unit/integration tests
* Prerequisites: Runtime spike state machine accepted; dependencies: `RLM-0601`
* Deliverables and acceptance: No duplicate concurrent poll or lost wakeup;
  scripted schedules and clean root shutdown are reproducible
* Validation: State-transition unit tests, model checking where practical,
  seeded schedule tests, and fault injection
* Non-goals: Worker threads, real timers, or channels; risks: `RISK-017`; size:
  `L`

### `RLM-0604`: Implement Structured Task Scopes

* Purpose and exact scope: Add scope entry/exit, child handles, result storage,
  implicit join, child accounting, first-failure selection, sibling cancellation
  request, and no-escape enforcement
* Expected files/modules: Compiler task operations, runtime task scope, standard
  task API seed, `tests/concurrency/scopes`
* Prerequisites: Accepted scope failure precedence; dependencies: `RLM-0602`,
  `RLM-0603`
* Deliverables and acceptance: Children cannot outlive a scope; success joins all
  results and failure deterministically propagates after child cleanup
* Validation: Scripted child completion/failure orders, nested scopes, handle
  escape compile failures, and destructor counters
* Non-goals: Detached tasks, supervisors, or actors; risks: `RISK-008`,
  `RISK-017`; size: `L`

### `RLM-0605`: Implement Cooperative Cancellation

* Purpose and exact scope: Add cancellation state/tree propagation, explicit
  checks, cancellation points, wake blocked tasks, terminal outcome, effect
  distinction, and cleanup
* Expected files/modules: Runtime task modules, compiler effect/lowering modules,
  `tests/concurrency/cancellation`
* Prerequisites: Cancellation semantic representation accepted; dependencies:
  `RLM-0604`
* Deliverables and acceptance: Requests are idempotent/downward, blocked children
  wake, and cancellation never skips or duplicates cleanup
* Validation: Scripted cancellation before/during/after wait, nested scope races,
  exception precedence, and CPU loop explicit-check tests
* Non-goals: Forced preemption, deadlines, or implicit checks everywhere; risks:
  `RISK-015`, `RISK-018`; size: `L`

### `RLM-0606`: Implement Bounded Typed Channels

* Purpose and exact scope: Add sender/receiver capabilities, fixed capacity,
  move send, receive, blocking registration, close/drop, cancellation races,
  ordering, and buffered cleanup
* Expected files/modules: `runtime/realm-runtime/src/channel`, standard channel
  API, runtime ABI, `tests/concurrency/channels`
* Prerequisites: `RLM-0009`; dependencies: `RLM-0605`
* Deliverables and acceptance: Channel state table is fully implemented and each
  successful receive obtains exactly one owned message
* Validation: Deterministic race matrix, happens-before litmus tests, destructor
  counters, parallel stress, and compile-fail non-sendable messages
* Non-goals: Unbounded channels, broadcast, or select; risks: `RISK-019`,
  `RISK-017`; size: `L`

### `RLM-0607`: Add the Bounded Parallel Worker Pool

* Purpose and exact scope: Add configurable workers, local/global queues, work
  stealing if selected, thread initialization, wake parking, fairness metrics,
  and ordered shutdown
* Expected files/modules: `runtime/realm-runtime/src/task`, Windows platform
  synchronization, `tests/concurrency/scheduler`
* Prerequisites: Deterministic semantics stable; dependencies: `RLM-0604`,
  `RLM-0606`
* Deliverables and acceptance: CPU child tasks execute in parallel within bounds
  with no concurrent poll, starvation trigger, or shutdown hang
* Validation: Parallel stress, oversubscription bounds, repeated shutdown, queue
  race model tests, and task trace review
* Non-goals: Priorities, affinity, custom executors, or observable worker IDs;
  risks: `RISK-017`, `RISK-025`; size: `L`

### `RLM-0608`: Implement Timers and Virtual Time

* Purpose and exact scope: Add timer registration, cancellation, ordered wakeups,
  real Windows clock driver, deterministic virtual clock, and scope cleanup
* Expected files/modules: `runtime/realm-runtime/src/time`, platform Windows
  timer module, standard time API, `tests/concurrency/timers`
* Prerequisites: Executor wait interface stable; dependencies: `RLM-0605`,
  `RLM-0607`
* Deliverables and acceptance: Timers wake no earlier than specified policy,
  cancel exactly once, and virtual-time tests require no wall-clock sleeps
* Validation: Scripted same-deadline order, cancellation/completion races,
  overflow boundaries, and worker shutdown with timers pending
* Non-goals: Calendar/time-zone APIs or deadlines; risks: `RISK-017`,
  `RISK-018`; size: `M`

### `RLM-0609`: Implement Minimal Windows Async I/O and Blocking Isolation

* Purpose and exact scope: Add selected IOCP operation, pinned operation/buffer
  lifetime, completion/cancel race, bounded blocking pool, and unsafe FFI contract
* Expected files/modules: `runtime/realm-runtime/src/io`,
  `runtime/realm-runtime/src/platform/windows`, standard I/O seed,
  `tests/concurrency/io`
* Prerequisites: ADR-007 event-driver result accepted; dependencies: `RLM-0605`,
  `RLM-0607`, `RLM-0608`, `RLM-0504`
* Deliverables and acceptance: One real async operation completes/cancels once
  without dead frame/buffer; blocked C calls cannot exhaust normal workers
* Validation: Deterministic injected completions, real Windows integration,
  cancel/completion stress, pool saturation, and leak instrumentation
* Non-goals: Broad filesystem/network APIs or forced C-call cancellation; risks:
  `RISK-009`, `RISK-018`; size: `L`

### `RLM-0610`: Run the Concurrency Semantics Gate

* Purpose and exact scope: Integrate task frames, sendability, scopes, exceptions,
  cancellation, channels, workers, timers, I/O, traces, and logical stack output
* Expected files/modules: Top-level `tests/concurrency`, runtime trace fixtures,
  debug metadata checks
* Prerequisites: All Phase 6 features complete; dependencies: `RLM-0601`-
  `RLM-0609`
* Deliverables and acceptance: Every gate in `concurrency-model.md` passes and
  ADR-007/ADR-008 evidence is updated with no open high-impact trigger
* Validation: Deterministic schedule suite, parallel stress, model/sanitizer runs
  available to Rust runtime, native optimized tests, and task-stack inspection
* Non-goals: New features or performance tuning; risks: `RISK-008`, `RISK-017`,
  `RISK-018`, `RISK-019`; size: `M`

### Phase 6 Checkpoint

Concurrency remains experimental if any race, cleanup, cancellation, or logical
stack gate fails. Do not compensate for unsound sendability with documentation
or runtime checks.

## Phase 7: MVP Product Surface

### `RLM-0701`: Complete Core and Standard Library Foundations

* Purpose and exact scope: Stabilize option/result, formatting, console, owned
  strings, vectors, slices, iteration, time, task, cancellation, channel, and
  minimum async I/O APIs over existing semantics
* Expected files/modules: `library/core`, `library/std`, library conformance tests
* Prerequisites: Core ownership and concurrency APIs implemented; dependencies:
  `RLM-0405`, `RLM-0610`
* Deliverables and acceptance: `REQ-050`/`REQ-051` API subset builds from Realm
  source and signatures expose ownership/effects/sendability truthfully
* Validation: Library unit/run tests, leak/drop checks, documentation examples,
  and API review against specifications
* Non-goals: Broad networking, serialization, tensors, or accelerators; risks:
  `RISK-001`, `RISK-015`; size: `L`

### `RLM-0702`: Build Local Multi-Package Programs

* Purpose and exact scope: Connect manifest/module DAGs to driver builds,
  compile package order, enforce visibility, link one executable, and diagnose
  package-stage failures
* Expected files/modules: `compiler/realm-package`, `compiler/realm-driver`,
  `tools/realm-cli`, `tests/packages/build`
* Prerequisites: Package graph and native driver stable; dependencies:
  `RLM-0201`, `RLM-0305`, `RLM-0701`
* Deliverables and acceptance: A local dependency DAG builds/runs deterministically
  and private declarations/cycles fail before LLVM
* Validation: Multi-package run-pass/fail fixtures and clean rebuild comparison
* Non-goals: Remote fetch, incremental caching, or version ranges; risks:
  `RISK-011`, `RISK-020`; size: `M`

### `RLM-0703`: Implement Explicit Remote Package Acquisition

* Purpose and exact scope: Add HTTPS/Git exact immutable identity, verified cache,
  explicit fetch/update commands, trust errors, offline mode, and compiler
  separation
* Expected files/modules: `tools/realm-package-fetch`, CLI package commands,
  `tests/packages/remote`
* Prerequisites: ADR-010 accepted after security review; dependencies:
  `RLM-0702`
* Deliverables and acceptance: Same manifest/content identity resolves identical
  bytes; build never performs hidden network access; tampering is rejected
* Validation: Local test servers/repositories, cache corruption, offline,
  redirect, mutable-ref rejection, and credential-redaction tests
* Non-goals: Registry, solver, lockfile, or arbitrary install scripts; risks:
  `RISK-020`, `RISK-021`; size: `L`

### `RLM-0704`: Implement the Realm Test Runner

* Purpose and exact scope: Discover Realm tests, compile isolated programs,
  filter, execute with timeout, capture output, and report human/structured results
* Expected files/modules: `tools/realm-test-runner`, `library/test`, CLI `test`
  command, runner fixtures
* Prerequisites: Local package builds stable; dependencies: `RLM-0702`
* Deliverables and acceptance: `realm test` handles pass, assertion failure,
  compile failure, throw, timeout, filter, and deterministic result ordering
* Validation: End-to-end self-tests including paths/spaces, parallel isolation,
  cleanup, and JSON result assertions
* Non-goals: Benchmark runner, coverage, or distributed tests; risks:
  `RISK-024`; size: `M`

### `RLM-0705`: Complete Native Debug Information

* Purpose and exact scope: Emit file/line, function, parameter, supported local,
  nominal type, exception stack, task creation/suspension, and logical async stack
  metadata
* Expected files/modules: Backend debug module, runtime task metadata,
  `tests/debug-info`
* Prerequisites: Debug representation approved; dependencies: `RLM-0610`,
  `RLM-0701`
* Deliverables and acceptance: Supported debugger/symbol tooling reports truthful
  source, values where promised, native stack, and logical task stack
* Validation: Debugger or symbol-script fixtures at each supported optimization
  level and exception/cancellation/task cases
* Non-goals: Custom debugger UI or expression evaluator; risks: `RISK-023`;
  size: `L`

### `RLM-0706`: Finalize Diagnostic JSON and Internal Errors

* Purpose and exact scope: Version JSON schema, expose all labels/edits/codes,
  stabilize CLI format selection, redact sensitive paths where configured, and
  provide controlled internal-error reproduction metadata
* Expected files/modules: `compiler/realm-diagnostics`, `compiler/realm-driver`,
  `tools/realm-cli`, diagnostic schema fixtures
* Prerequisites: Feature diagnostics representative; dependencies: `RLM-0704`
* Deliverables and acceptance: Human and JSON outputs represent the same ordered
  diagnostics; internal errors never publish artifacts or panic through CLI
* Validation: Schema validation, golden/structural tests, redaction, broken-pipe,
  and injected-invariant-failure cases
* Non-goals: LSP transport or localization; risks: `RISK-022`, `RISK-026`;
  size: `M`

### `RLM-0707`: Validate Representative General and AI Workloads

* Purpose and exact scope: Select small evidence-based programs covering CLI
  processing, structured I/O pipeline, C numerical-library FFI, parallel CPU
  work, and bounded data movement
* Expected files/modules: `examples/`, workload fixtures, benchmark/report data
* Prerequisites: MVP library/tooling available; dependencies: `RLM-0701`,
  `RLM-0702`, `RLM-0704`
* Deliverables and acceptance: Programs build/test/run without broad unsafe code;
  missing numerical/tensor/accelerator needs are measured and deferred explicitly
* Validation: Correctness, memory, throughput, binary size, and developer-friction
  report with reproducible inputs
* Non-goals: Tensor framework or accelerator backend; risks: `RISK-027`,
  `RISK-025`; size: `M`

### Phase 7 Checkpoint

The MVP product gate requires local packages, the minimal standard library,
language test runner, structured diagnostics, native debug information, and
representative programs. Remote acquisition may be deferred from the compiler
MVP if ADR-010 remains blocked, but the manifest must not claim unsupported
remote reproducibility.

## Phase 8: Hardening and Release Decision

### `RLM-0801`: Run Robustness and Fuzz Campaigns

* Purpose and exact scope: Fuzz lexer/parser, structured stage inputs where safe,
  manifest parsing, diagnostic rendering, and selected runtime state machines;
  triage and preserve regressions
* Expected files/modules: Fuzz targets, corpora, CI scheduled jobs, regression
  fixtures
* Prerequisites: MVP surface complete; dependencies: `RLM-0706`
* Deliverables and acceptance: Agreed campaign budget completes with no crash,
  uncontrolled panic, hang, memory-safety report, or unbounded resource use
* Validation: Reproducible seeds, minimized corpus, sanitizer-compatible Rust
  targets where available, and issue disposition report
* Non-goals: Formal verification of the compiler; risks: `RISK-010`,
  `RISK-024`; size: `M`

### `RLM-0802`: Run ABI and Concurrency Stress Matrix

* Purpose and exact scope: Execute optimization, exception, C ABI, CPU-count,
  cancellation, channel, timer, I/O, shutdown, and blocking-pool matrices
* Expected files/modules: `tests/abi`, `tests/concurrency`, CI matrix definitions,
  evidence report
* Prerequisites: Debug and runtime gates complete; dependencies: `RLM-0610`,
  `RLM-0705`
* Deliverables and acceptance: No flaky outcome, leak, double completion, invalid
  unwind, lost wakeup, or debugger corruption in the approved stress budget
* Validation: Repeated clean Windows runs with retained traces for failures
* Non-goals: Unsupported platforms or actors; risks: `RISK-005`, `RISK-016`,
  `RISK-017`, `RISK-018`, `RISK-019`; size: `M`

### `RLM-0803`: Verify Reproducibility and Supply Chain

* Purpose and exact scope: Audit dependencies/licenses/unsafe code, pin toolchain
  inputs, verify offline builds, compare clean artifacts, validate remote cache if
  shipped, and document SDK prerequisites
* Expected files/modules: Workspace manifests/lockfile, CI, audit configuration,
  reproducibility report
* Prerequisites: Dependency set frozen for candidate; dependencies: `RLM-0703`
  if remote packages ship, otherwise `RLM-0702`
* Deliverables and acceptance: Identical declared inputs produce equivalent
  artifacts under documented normalization; no unresolved prohibited dependency
* Validation: Two clean builds, dependency/license/advisory scans, offline test,
  architecture dependency check, and cache-tamper test when applicable
* Non-goals: Bit identity where toolchain metadata prevents it without an
  approved requirement; risks: `RISK-004`, `RISK-020`, `RISK-021`; size: `M`

### `RLM-0804`: Set and Enforce Performance Budgets

* Purpose and exact scope: Compare Phase 4 baselines, profile regressions, set
  compile time/memory, binary size, startup, allocation, task, channel, and I/O
  budgets, and fix only release-blocking bottlenecks through follow-up tasks
* Expected files/modules: `benchmarks/`, CI threshold configuration, performance
  report
* Prerequisites: Representative workloads stable; dependencies: `RLM-0408`,
  `RLM-0707`, `RLM-0802`
* Deliverables and acceptance: Approved budgets with variance policy and no
  unresolved release-blocking regression
* Validation: Repeated controlled benchmark runs and profiler evidence for every
  requested optimization task
* Non-goals: Winning broad language benchmarks or weakening semantics; risks:
  `RISK-014`, `RISK-025`; size: `M`

### `RLM-0805`: Complete Conformance and Compatibility Review

* Purpose and exact scope: Map every `REQ-*` to specification and tests, label
  experimental surfaces, version CLI JSON/runtime ABI/manifest boundaries, and
  record compatibility promises
* Expected files/modules: Specifications, conformance index, decision log,
  version policy, release notes draft
* Prerequisites: All intended MVP features complete; dependencies: `RLM-0801`-
  `RLM-0804`
* Deliverables and acceptance: No requirement lacks evidence or explicit deferral;
  no undocumented behavior is called stable
* Validation: Independent requirement, ADR, risk, and test traceability audit
* Non-goals: Self-hosting or ecosystem stability guarantee; risks: `RISK-002`,
  `RISK-026`; size: `M`

### `RLM-0806`: Release and Self-Hosting Feasibility Decision

* Purpose and exact scope: Review all gates, accept residual risks explicitly,
  decide release readiness, and evaluate a bounded post-release self-hosting
  experiment without committing production rewrite
* Expected files/modules: Decision log, risk status, release checklist, future
  experiment proposal only
* Prerequisites: `RLM-0805`; dependencies: `RLM-0805`
* Deliverables and acceptance: Go/no-go decision with evidence, named owners for
  accepted risks, and ADR-015 retained/updated from measured compiler workloads
* Validation: Cross-discipline sign-off covering language, compiler, ABI,
  runtime, security, tooling, and support prerequisites
* Non-goals: Implementing self-hosting or declaring 1.0 compatibility; risks:
  `RISK-001`, `RISK-026`, `RISK-030`; size: `S`

## Critical Path

The shortest path to a trustworthy native language is:

```text
RLM-0001/0002/0003/0005/0011 -> RLM-0012 -> RLM-0101 -> RLM-0102
-> RLM-0103 -> RLM-0104 -> RLM-0105 -> RLM-0106 -> RLM-0108
-> RLM-0202 -> RLM-0203 -> RLM-0204 -> RLM-0205 -> RLM-0301
-> RLM-0302/0303 -> RLM-0304 -> RLM-0305 -> RLM-0306
-> RLM-0402 -> RLM-0403 -> RLM-0404 -> RLM-0501 -> RLM-0503
-> RLM-0601 -> RLM-0602 -> RLM-0603 -> RLM-0604 -> RLM-0605
-> RLM-0606/0607/0608/0609 -> RLM-0610 -> RLM-0701
-> RLM-0704/0705/0706 -> RLM-0801..0806
```

The first executable at `RLM-0306` is the earliest major integration milestone.
It intentionally excludes ownership completeness, generics, exceptions, and
concurrency while preserving architecture that can add them.

## Parallel Work

* `RLM-0003`, `RLM-0010`, and `RLM-0011` can run in parallel after their stated
  inputs; ownership, exception, and concurrency spikes remain dependency ordered
* After `RLM-0101`, source/diagnostic foundations and package manifest parsing
  can be developed independently, but package-driver integration waits
* `RLM-0204` type representation can proceed alongside HIR/resolution after
  shared IDs are stable
* `RLM-0302` runtime ABI/runtime startup can proceed alongside basic CFG work
* `RLM-0401` aggregate codegen and the ownership analysis sequence can overlap
  after CFG representation is stable
* In Phase 6, compiler sendability work and runtime deterministic executor work
  can proceed in parallel after the async frame ABI is fixed
* `RLM-0704`, `RLM-0705`, and `RLM-0707` can overlap after their dependencies;
  remote package acquisition is isolated from the release critical path unless
  explicitly made an MVP gate

Parallel tasks must not edit the same representation contract concurrently. A
representation owner lands interface changes before consumer tasks begin.

## Integration Points

| Integration point | Producer | Consumer | Required proof |
|---|---|---|---|
| Tokens to syntax | `RLM-0104` | `RLM-0105` | Stable kind/range/trivia and recovery contract |
| Syntax to HIR | `RLM-0106` | `RLM-0202` | No syntax library types escape HIR boundary |
| HIR to typed HIR | `RLM-0203` | `RLM-0205` | Every path has definition/error ID and source origin |
| Typed HIR to CFG | `RLM-0206` | `RLM-0301` | Every expression has type, call target, coercion, and effect |
| CFG to LLVM | `RLM-0301` | `RLM-0303` | CFG validator passes; backend performs no semantic repair |
| Generated code to runtime | `RLM-0302` | `RLM-0303` | Versioned symbol/layout/calling contract and ABI tests |
| Objects to linker | `RLM-0303` | `RLM-0304` | Target/data layout and runtime/system inputs recorded |
| Ownership to exceptions | `RLM-0404` | `RLM-0501` | Cleanup exactly once on every normal edge before unwind edges |
| Exceptions to tasks | `RLM-0503` | `RLM-0601` | Unwind stops at task trampoline and becomes terminal state |
| Sendability to scheduler | `RLM-0602` | `RLM-0604` | Capture proof attached before runtime task creation |
| Runtime APIs to library | `RLM-0610` | `RLM-0701` | Safe signatures preserve ownership, effects, and cancellation |
| Package graph to driver | `RLM-0201` | `RLM-0702` | Deterministic local DAG with no acquisition side effects |

## Requirement Traceability

| Requirements | Primary tasks |
|---|---|
| `REQ-001`-`REQ-007` | `RLM-0001`, `RLM-0102`, `RLM-0104`-`RLM-0107`, `RLM-0405` |
| `REQ-008`-`REQ-016` | `RLM-0002`, `RLM-0005`, `RLM-0007`, `RLM-0204`, `RLM-0205`, `RLM-0402`-`RLM-0405`, `RLM-0602` |
| `REQ-017`-`REQ-021` | `RLM-0004`, `RLM-0207`, `RLM-0208`, `RLM-0406`, `RLM-0407` |
| `REQ-022`-`REQ-026` | `RLM-0006`, `RLM-0206`, `RLM-0501`-`RLM-0503`, `RLM-0605` |
| `REQ-027`-`REQ-033` | `RLM-0010`, `RLM-0201`, `RLM-0702`, `RLM-0703` |
| `REQ-034`-`REQ-040` | `RLM-0011`, `RLM-0103`, `RLM-0108`, `RLM-0301`-`RLM-0306`, `RLM-0706` |
| `REQ-041`-`REQ-049` | `RLM-0007`-`RLM-0009`, `RLM-0302`, `RLM-0504`, `RLM-0601`-`RLM-0610` |
| `REQ-050`-`REQ-054` | `RLM-0701`, `RLM-0704`-`RLM-0706` |
| `REQ-055`-`REQ-062` | Every phase gate; especially `RLM-0107`, `RLM-0408`, `RLM-0610`, and `RLM-0801`-`RLM-0805` |

## Roadmap Change Policy

Discovery that changes language behavior returns to a specification and ADR
task before implementation continues. Discovery that only changes an internal
algorithm may update a task after dependency and acceptance review.

Do not mark a task complete because its files exist. Completion requires its
deliverables, acceptance criteria, and validation evidence. If a task triggers a
listed high-impact risk, pause dependent tasks and update the risk register and
decision log before resuming.