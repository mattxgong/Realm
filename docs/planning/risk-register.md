---
title: Realm Risk Register
description: Technical, product, security, quality, and delivery risks with mitigations, triggers, and phase ownership
ms.date: 2026-08-23
ms.topic: reference
---

## Rating Model

Likelihood and impact use `Low`, `Medium`, or `High` relative ratings. A risk is
open until its mitigation acceptance criteria pass; completing its owning phase
does not close it automatically.

* `High` impact can invalidate language semantics, corrupt generated programs,
  cause unsafe behavior, or block the supported platform
* `Medium` impact can force significant rework, miss a release gate, or make a
  major feature unusable
* `Low` impact is localized and has a known workaround

Phase ownership identifies where mitigation must begin. Later phases continue
monitoring risks whose triggers remain possible.

## Summary

| ID | Category | Risk | Likelihood | Impact | Owning phase |
|---|---|---|---|---|---|
| `RISK-001` | Scope | MVP breadth prevents end-to-end progress | High | High | Phase 0 |
| `RISK-002` | Specification | Ambiguous surface semantics cause incompatible implementations | High | High | Phase 0 |
| `RISK-003` | Parsing | Lossless recovery architecture is too complex or memory-heavy | Medium | Medium | Phase 0 |
| `RISK-004` | Toolchain | LLVM/Inkwell pair is unavailable or unstable on Windows | Medium | High | Phase 0 |
| `RISK-005` | ABI | Windows x64 exception unwinding cannot meet cleanup contracts | Medium | High | Phase 0 |
| `RISK-006` | Type system | Generic constraint model expands into an unplanned trait system | High | High | Phase 0 |
| `RISK-007` | Ownership | Borrow checking rejects ordinary code or accepts unsound aliases | High | High | Phase 0 |
| `RISK-008` | Concurrency safety | Sendability or suspension rules leave races or unusable APIs | High | High | Phase 0 |
| `RISK-009` | Runtime | Async executor and Windows event driver exceed project capacity | High | High | Phase 0 |
| `RISK-010` | Compiler correctness | Stage error sentinels cascade or panic on malformed programs | Medium | High | Phase 1 |
| `RISK-011` | Determinism | Hash or traversal order changes diagnostics and artifacts | Medium | Medium | Phase 1 |
| `RISK-012` | Native code | LLVM output verifies but violates Realm semantics or ABI | Medium | High | Phase 3 |
| `RISK-013` | Linking | MSVC/SDK discovery is brittle across developer and CI machines | High | Medium | Phase 3 |
| `RISK-014` | Generics | Monomorphization causes runaway expansion or binary bloat | Medium | High | Phase 4 |
| `RISK-015` | Cleanup | Moves, partial initialization, exceptions, and cancellation double-drop or leak | High | High | Phase 4 |
| `RISK-016` | FFI | Foreign layouts, callbacks, or unwinding create undefined behavior | Medium | High | Phase 5 |
| `RISK-017` | Scheduler | Lost wakeups, starvation, or shutdown races corrupt task behavior | High | High | Phase 6 |
| `RISK-018` | Cancellation | Cancellation races leak kernel operations or skip cleanup | High | High | Phase 6 |
| `RISK-019` | Channels | Close, fairness, or cancellation semantics become nondeterministic | Medium | High | Phase 6 |
| `RISK-020` | Packaging security | Remote dependencies are mutable or unauthenticated | High | High | Phase 7 |
| `RISK-021` | Supply chain | Rust or native dependencies introduce unsafe or abandoned code | Medium | High | Phase 0 |
| `RISK-022` | Diagnostics | Internal architecture leaks into unstable or poor user messages | Medium | Medium | Phase 1 |
| `RISK-023` | Debugging | Optimized and async code lacks truthful source/debugger mapping | High | Medium | Phase 7 |
| `RISK-024` | Testing | Snapshot-heavy tests miss semantic defects and resist refactoring | Medium | High | Phase 1 |
| `RISK-025` | Performance | Compile time, runtime overhead, or binary size misses goals | Medium | High | Phase 4 |
| `RISK-026` | Compatibility | Premature syntax, ABI, or package stability blocks correction | High | High | Phase 0 |
| `RISK-027` | AI product fit | Core design is optimized around hypothetical AI workloads | Medium | Medium | Phase 0 |
| `RISK-028` | Delivery | Cross-cutting tasks are too large for independent agent execution | High | Medium | Phase 0 |
| `RISK-029` | Platform lock-in | Windows details leak into language and target-neutral IR | Medium | Medium | Phase 3 |
| `RISK-030` | Bootstrap | Self-hosting pressure destabilizes the Rust compiler too early | Medium | High | Phase 8 |

## Risk Details

### `RISK-001`: MVP Breadth

Mitigation: Preserve a thin vertical critical path, use explicit deferred lists,
and require each phase to produce independently testable behavior before adding
language breadth.

Trigger: A phase adds more than two unrelated language features before its exit
checkpoint, or no native vertical-slice milestone moves for two planning cycles.

Owner: Phase 0 planning lead; enforced at every phase review.

### `RISK-002`: Ambiguous Surface Semantics

Mitigation: Write normative lexical, grammar, numeric, indexing, slicing,
ownership, exception, module, and concurrency specifications with conformance
examples before dependent implementation.

Trigger: An implementation task must choose behavior not covered by an accepted
specification or ADR.

Owner: Phase 0 language design.

### `RISK-003`: Lossless Parser Cost

Mitigation: Benchmark a Rowan-style event parser against a simple Realm-owned
lossless arena using malformed and representative files. Accept ADR-003 only
after recovery and memory budgets are measured.

Trigger: The prototype cannot round-trip trivia and recover multiple errors, or
uses more than the agreed bytes per source byte.

Owner: Phase 0 syntax spike.

### `RISK-004`: LLVM/Inkwell Compatibility

Mitigation: Probe released version pairs on the actual Windows target, pin one
pair and installation method, compile object/debug fixtures in CI, and isolate
Inkwell behind `realm-codegen-llvm`.

Trigger: Required LLVM APIs are absent, linking LLVM fails in clean CI, module
verification differs by machine, or the selected pair is unsupported upstream.

Owner: Phase 0 backend spike.

### `RISK-005`: Exception ABI

Mitigation: Build a disposable Windows x64 unwind probe that runs Realm-like
cleanups, catches at a task boundary, rejects C-boundary escape, and preserves
debug stack information. Keep ADR-006 blocked until it passes.

Trigger: Cleanup pads cannot express the required ownership order, exceptions
cross incompatible frames, or optimized builds change behavior.

Owner: Phase 0 ABI spike; implemented in Phase 5.

### `RISK-006`: Generic Constraint Scope

Mitigation: Specify the smallest constraint operations needed by initial generic
containers and algorithms. Exclude associated types, specialization, dynamic
dispatch, coherence across remote packages, and broad operator overloading.

Trigger: Type checking a planned generic requires an unplanned solver feature or
constraint behavior depends on reachable concrete instances.

Owner: Phase 0 type-system design.

### `RISK-007`: Borrow Checker Soundness and Usability

Mitigation: Define places, moves, loans, conflicts, liveness, reborrowing, and
drop rules over CFG; validate with compile-fail suites and representative code;
prefer conservative rejection over unsound acceptance.

Trigger: A safe fixture demonstrates use after free, aliasing mutation, double
drop, or routine patterns require broad unsafe blocks.

Owner: Phase 0 ownership model; implemented in Phase 4.

### `RISK-008`: Concurrency Type Safety

Mitigation: Specify structural `Send` and `Sync`, capture classification,
suspension borrows, task-handle escape, and happens-before. Validate with
compile-fail cases and memory-model litmus tests.

Trigger: A non-sendable value reaches another worker, a mutable alias crosses a
task boundary, or common scoped pipelines cannot be expressed.

Owner: Phase 0 concurrency type-system spike; implemented in Phase 6.

### `RISK-009`: Runtime Ambition

Mitigation: Start with a deterministic single-thread executor, add one mechanism
at a time, use bounded queues and pools, keep the async I/O surface minimal, and
defer actors, priorities, selection, and custom executors.

Trigger: Runtime tasks require more than one phase without a passing child-task
vertical slice, or platform work dominates compiler progress.

Owner: Phase 0 runtime spike; implemented in Phase 6.

### `RISK-010`: Error Recovery Cascades

Mitigation: Define stage preconditions and typed error sentinels, cap derivative
diagnostics, fuzz malformed input, and convert only violated internal invariants
to controlled internal errors.

Trigger: One source defect emits an unbounded diagnostic cascade, crashes a
later stage, or creates an artifact.

Owner: Phase 1 front-end implementation.

### `RISK-011`: Nondeterministic Builds

Mitigation: Use sorted stable IDs and outputs, avoid semantic hash iteration,
seed test runs differently, and compare artifacts and diagnostics across clean
rebuilds.

Trigger: Identical requests produce different diagnostics, symbols, LLVM IR,
objects, or test order.

Owner: Phase 1 foundations; monitored through Phase 8.

### `RISK-012`: Semantically Incorrect LLVM

Mitigation: Keep code generation mechanical, verify modules before and after
optimization, compare `-O0` and optimized results, assert focused IR properties,
and run native conformance programs.

Trigger: LLVM verification passes while native behavior differs from CFG
interpretation or language semantics.

Owner: Phase 3 backend.

### `RISK-013`: Windows Toolchain Discovery

Mitigation: Separate pure linker command construction from discovery, support
explicit overrides, pin CI images, report discovered paths and versions, and
test paths containing spaces.

Trigger: A clean supported machine cannot link without undocumented environment
setup, or discovery selects incompatible SDK components.

Owner: Phase 3 linker integration.

### `RISK-014`: Monomorphization Expansion

Mitigation: Canonicalize instance keys, deduplicate, cap recursive expansion,
explain instance chains in diagnostics, measure instance counts and object size,
and retain a future shape-sharing boundary.

Trigger: Instance count grows superlinearly on representative programs or one
generic produces unbounded specialization.

Owner: Phase 4 generics.

### `RISK-015`: Cleanup Correctness

Mitigation: Model initialization and cleanup explicitly in CFG, insert drops
after ownership validation, test every exit edge, and add runtime counters for
exactly-once destruction in debug fixtures.

Trigger: A value leaks or is destroyed twice on branch, early return, partial
construction, throw, catch, cancellation, or task completion.

Owner: Phase 4 ownership; expanded in Phases 5 and 6.

### `RISK-016`: FFI Undefined Behavior

Mitigation: Limit C ABI types, require explicit C layout and unsafe calls,
generate static layout assertions, probe calls in both directions, and prohibit
unwinding across the boundary.

Trigger: Realm and C disagree on size, alignment, calling convention, ownership,
callback lifetime, or exception behavior.

Owner: Phase 5 ABI/FFI.

### `RISK-017`: Scheduler Correctness

Mitigation: Build deterministic task-state tests first, model-check small Rust
synchronization components where practical, instrument state transitions, and
stress parallel shutdown and wakeup paths.

Trigger: Lost wakeup, task polled concurrently, persistent starvation, orphaned
scope, or nondeterministic shutdown hang.

Owner: Phase 6 runtime.

### `RISK-018`: Cancellation and Kernel Lifetime

Mitigation: Specify operation ownership, pin buffers through terminal kernel
completion, race cancellation against completion in deterministic tests, and
require one terminal state and cleanup winner.

Trigger: Cancelled I/O accesses freed memory, completes twice, leaks a task
frame, or skips a destructor.

Owner: Phase 6 Windows event driver.

### `RISK-019`: Channel Semantics

Mitigation: Begin with fixed bounded channels, specify close and drop behavior,
defer selection, distinguish guarantees from fairness goals, and test each race
with a deterministic executor.

Trigger: Outcome depends on undocumented waiter ordering, buffered values leak,
or cancellation consumes or duplicates a message.

Owner: Phase 6 channels.

### `RISK-020`: Remote Dependency Integrity

Mitigation: Implement local paths first. Require immutable revision and content
hashes, explicit fetch/update commands, trust policy, cache verification, and
offline behavior before remote dependencies.

Trigger: Compilation performs hidden network access or the same manifest can
resolve different bytes without an explicit update.

Owner: Phase 7 package acquisition.

### `RISK-021`: Implementation Supply Chain

Mitigation: Review every dependency for maintenance, license, unsafe code,
transitive size, platform support, and exit strategy; pin versions; automate
advisory and license checks.

Trigger: An abandoned or vulnerable dependency owns a safety-critical path or a
new dependency materially widens native build requirements.

Owner: Phase 0 workspace policy; monitored continuously.

### `RISK-022`: Diagnostic Quality

Mitigation: Design diagnostics with user concepts, stable codes, primary and
secondary labels, and fixes; maintain bad-program fixtures; review JSON and
human output independently.

Trigger: Messages expose internal IR names, lack a relevant source label, change
code without policy, or suggest an invalid edit.

Owner: Phase 1 diagnostics; monitored for every feature.

### `RISK-023`: Debug Information

Mitigation: Start line-table probes in the native slice, test locals and stacks
before optimization, retain async frame metadata, and gate release on truthful
source locations at supported optimization levels.

Trigger: Debugger stepping skips or misattributes Realm code, stack traces omit
throw/task boundaries, or async suspension locations are unavailable.

Owner: Phase 7 tooling, with a Phase 3 probe.

### `RISK-024`: Test Strategy Blind Spots

Mitigation: Pair snapshots with structural assertions, property/fuzz tests,
native execution, ABI probes, differential stage checks, and concurrency litmus
tests. Normalize only truly unstable output.

Trigger: A semantic regression changes no snapshot, or harmless formatting
changes require widespread fixture rewrites.

Owner: Phase 1 test harness; expanded every phase.

### `RISK-025`: Performance Failure

Mitigation: Establish baselines before optimization, track stage time and memory,
generic instance counts, object size, task throughput, latency, and allocation;
optimize only measured bottlenecks.

Trigger: A benchmark exceeds an approved budget in two consecutive changes or
an optimization weakens diagnostics or semantics.

Owner: Phase 4 baseline; release budgets in Phase 8.

### `RISK-026`: Premature Stability

Mitigation: Mark language, ABI, diagnostics, manifests, and library APIs
experimental; require accepted specification and conformance coverage before
stability; version each compatibility boundary independently.

Trigger: External code depends on undocumented behavior or an internal ABI must
be preserved without an approved compatibility promise.

Owner: Phase 0 governance.

### `RISK-027`: Unverified AI Fit

Mitigation: Select concrete AI-oriented workloads after the core native slice,
measure FFI, numerical layout, pipeline, and accelerator needs, and defer tensor
syntax or runtime until evidence exists.

Trigger: A proposed core feature is justified only by “AI applications” without
a representative program and measured need.

Owner: Phase 0 requirements; workload evaluation in Phase 7.

### `RISK-028`: Tasks Too Large for Agents

Mitigation: Require one primary behavior and one focused validation path per
task, explicit expected files, no hidden decision-making, and phase checkpoints
that integrate completed tasks.

Trigger: A task spans more than two architectural layers, cannot be reviewed
without another unfinished task, or has acceptance criteria joined by unrelated
behaviors.

Owner: Phase 0 roadmap maintenance.

### `RISK-029`: Platform Leakage

Mitigation: Keep target facts in target/ABI descriptions, platform code in
backend/link/runtime adapters, and test target-neutral IR without Windows types.

Trigger: Syntax, HIR, type checking, CFG, or standard APIs expose LLVM or Windows
handle/layout details unnecessarily.

Owner: Phase 3 backend review.

### `RISK-030`: Early Self-Hosting

Mitigation: Keep Rust as the supported compiler implementation, use Realm
compiler workloads only as benchmarks, and require stable packages, diagnostics,
and bootstrap policy before any production rewrite.

Trigger: A release task depends on compiling the compiler in Realm or Rust work
is duplicated solely to claim self-hosting.

Owner: Phase 8 governance.

## Review Cadence

At each phase entry, review open risks owned by that phase and confirm their
mitigation tasks are scheduled. At phase exit, record evidence, downgrade or
close mitigated risks, and add newly discovered risks without renumbering.

Any triggered `High`-impact risk blocks dependent tasks until the decision log
records disposition. A risk accepted without mitigation requires an explicit
owner and review date; silence is not acceptance.