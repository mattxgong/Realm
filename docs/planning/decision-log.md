---
title: Realm Architectural Decision Log
description: Lightweight decision records for Realm language, compiler, runtime, and project architecture
ms.date: 2026-08-23
ms.topic: reference
---

## Status Definitions

* `Proposed` identifies a recommendation awaiting plan approval or prototype
  evidence
* `Blocked` identifies a decision whose prerequisites are incomplete
* `Accepted` identifies an approved decision that implementation may rely on
* `Superseded` identifies a decision replaced by a later ADR

No ADR is accepted merely because code happens to implement it. The plan begins
with all recommendations in `Proposed` or `Blocked` state.

## ADR-001: Initial Platform

Status: `Proposed`

Context: Native ABI, object format, exception handling, linker invocation, debug
information, async I/O, and CI all depend on the first target.

Candidate options: Windows x86-64 MSVC, Windows x86-64 GNU, both Windows ABIs,
or a multi-platform MVP.

Recommendation: Support `x86_64-pc-windows-msvc` only in the MVP. Emit COFF and
use the MSVC-compatible linker and Windows SDK.

Rationale: This matches the clarified target while limiting ABI and CI
combinations. Microsoft documents the
[Windows x64 calling convention](https://learn.microsoft.com/en-us/cpp/build/x64-calling-convention),
and LLVM supports target-specific object emission through `TargetMachine`.

Consequences: Windows exception handling, PDB/CodeView, COFF, SDK discovery,
IOCP, and MSVC linker behavior become first-class constraints. Portability must
remain in abstractions but is not a tested promise.

Revisit when: A second target is funded, Windows GNU compatibility is required,
or a selected LLVM facility cannot meet the MSVC ABI contract.

## ADR-002: Compiler Representation Pipeline

Status: `Proposed`

Context: Direct syntax-to-LLVM lowering would mix diagnostics, semantics,
ownership, cleanup, and target details.

Candidate options: AST directly to LLVM, AST plus one semantic IR, or explicit
syntax, HIR, typed HIR, CFG IR, and LLVM stages.

Recommendation: Use a recoverable syntax tree, resolved HIR, typed HIR, and
typed CFG IR before monomorphization and LLVM generation.

Rationale: The [rustc overview](https://rustc-dev-guide.rust-lang.org/overview.html)
shows distinct AST, HIR, THIR, MIR, and LLVM responsibilities. Realm needs a CFG
level for borrow dataflow, definite initialization, cleanup, suspension, and
exception edges.

Consequences: More initial types and lowering tests are required. In return,
each stage has a narrow contract and LLVM remains replaceable.

Revisit when: A stage has no independent invariant or test value after two
vertical slices.

## ADR-003: Syntax Representation

Status: `Proposed`

Context: The future formatter and LSP need malformed and trivia-preserving
source, while the batch compiler needs a typed syntax API.

Candidate options: Lossy AST only, hand-owned lossless tree, or Rowan-style
immutable green tree with typed wrappers.

Recommendation: Prototype a Rowan-style lossless CST, parser event stream, and
typed AST facade. Keep the parser independent of semantic crates.

Rationale: The
[rust-analyzer architecture](https://rust-analyzer.github.io/book/contributing/architecture.html)
uses recoverable parsing, syntax-only value trees, and an independent syntax API
to support both compilers and tools.

Consequences: Trivia and error nodes remain available. Syntax nodes cannot store
resolved definitions or inferred types.

Revisit when: A bounded prototype shows unacceptable complexity or source-size
cost compared with a simpler lossless arena tree.

## ADR-004: Memory Management

Status: `Proposed`

Context: Concurrency safety, exception cleanup, FFI, layouts, and generics all
depend on ownership.

Candidate options: Tracing GC, automatic reference counting, manual memory,
lexical ownership, or Rust-like affine ownership and borrowing.

Recommendation: Use affine values, moves, shared and exclusive borrows,
non-lexical lifetime analysis, deterministic destruction, and a narrow unsafe
boundary.

Rationale: This satisfies the selected memory-safety and runtime-performance
goals and composes with static data-race prevention. Rust demonstrates the
relationship between exclusive mutation and safe transfer through
[`Send` and `Sync`](https://doc.rust-lang.org/nomicon/send-and-sync.html).

Consequences: Borrow analysis becomes a critical-path compiler subsystem.
Suspension points, exception edges, destructors, closures, and FFI require
explicit ownership rules.

Revisit when: The ownership prototype cannot express representative application
and AI workloads without pervasive unsafe code or unmanageable annotations.

## ADR-005: Generic Implementation

Status: `Proposed`

Context: Generic representation affects type checking, code size, runtime
metadata, ABI, and linker behavior.

Candidate options: Full monomorphization, dictionary passing, erased boxing, or
shape-shared hybrid code.

Recommendation: Type-check generic bodies against explicit constraints and
fully monomorphize reachable instances from CFG IR.

Rationale: The
[rustc monomorphization model](https://rustc-dev-guide.rust-lang.org/backend/monomorph.html)
provides direct specialized code and avoids a stable runtime metadata ABI. It
trades compile time and binary size for runtime performance and implementation
clarity.

Consequences: Instance collection, recursion limits, deduplication, symbol
mangling, and code-size measurement are required. Canonical type arguments must
leave room for later shape sharing.

Revisit when: Representative programs exceed agreed compile-time or binary-size
budgets.

## ADR-006: Error Propagation

Status: `Blocked`

Context: Realm requires declared throwing effects and native unwinding, but the
exception ABI and cleanup rules are not yet specified.

Candidate options: Result values, abort-only panic, unchecked exceptions,
declared `throws`, or fully enumerated checked exception sets.

Recommendation: Track a Boolean `throws` effect in signatures, throw nominal
typed values, unwind on Windows x64, run deterministic cleanups, and terminate
after reporting an uncaught exception. Do not enumerate closed exception sets
in MVP signatures.

Rationale: Declared effects make exceptional control flow visible without the
evolution burden of closed checked sets. LLVM provides
[exception-handling mechanisms](https://llvm.org/docs/ExceptionHandling.html),
but language cleanup, personality, and ABI policy remain Realm decisions.

Consequences: The CFG needs unwind edges and cleanup scopes. C boundaries must
catch or prohibit Realm exceptions. A spike must validate Windows unwind and
debug behavior before acceptance.

Revisit when: The ABI spike fails, FFI requires cross-language unwinding, or
effect propagation makes core APIs unusable.

## ADR-007: Primary Concurrency Model

Status: `Proposed`

Context: Native concurrency must define task lifetime, ownership transfer,
failure, cancellation, scheduling, and I/O together.

Candidate options: OS threads and locks, CSP-style lightweight tasks and
channels, actors and mailboxes, or structured async tasks.

Recommendation: Use scoped structured async tasks as the lifetime and failure
model. Include cooperative cancellation, typed channels, timers, and minimal
async I/O. Defer actors, detached tasks, and raw user threads.

Rationale: Swift's
[structured concurrency proposal](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0304-structured-concurrency.md)
shows how bounded child lifetime supports local reasoning, downward
cancellation, and error propagation. Typed channels add communication without
making actor mailboxes the only state model.

Consequences: Async lowering, task frames, executor scheduling, wakeups,
cancellation checks, blocking-call policy, and debugger observability are
language/runtime contracts. Borrow rules must constrain values held across
suspension.

Revisit when: Runtime prototypes cannot provide bounded resources and usable
debugging, or target workloads require actor isolation as the primary model.

## ADR-008: Data-Race Safety

Status: `Blocked`

Context: Structured tasks alone do not prevent shared-memory races.

Candidate options: Data races as undefined behavior, runtime detection, actor
isolation only, or compile-time ownership and sendability enforcement.

Recommendation: Reject data races in safe code through ownership, exclusive
borrowing, captured-value analysis, and compiler-derived sendability. Specify
happens-before edges for task and channel operations.

Rationale: Rust's transfer/sharing marker model and the
[Go memory model](https://go.dev/ref/mem) provide complementary precedents for
static capability constraints and explicit synchronization ordering.

Consequences: Sendability and suspension-borrow rules must be proven in a type
system note and litmus suite before concurrency syntax is stabilized.

Revisit when: The model cannot support required FFI or application patterns
without broad unsafe escape hatches.

## ADR-009: Module and Package Graph

Status: `Proposed`

Context: File ownership, resolution order, initialization, and dependency
fetching need deterministic rules.

Candidate options: Multi-file modules with cycles, file modules in a DAG, or a
global package namespace.

Recommendation: Map one file to one module, require explicit imports, keep
declarations private by default, prohibit wildcard imports and re-exports, and
reject module and package cycles. Allow pure top-level initialization only.

Rationale: A DAG allows deterministic collection and initialization while the
compiler is young. Rust's separation of
[language modules](https://doc.rust-lang.org/reference/items/modules.html) and
[Cargo manifests](https://doc.rust-lang.org/cargo/reference/manifest.html)
supports keeping namespace semantics distinct from dependency acquisition.

Consequences: Some mutually recursive designs require consolidation into one
module. Purity rules and path-to-module mapping must be specified.

Revisit when: Real packages show recurring, legitimate cycles that interfaces or
package refactoring cannot resolve.

## ADR-010: Package Acquisition

Status: `Blocked`

Context: The MVP permits local, HTTPS, and Git dependencies with exact pins but
has no central registry or lockfile.

Candidate options: Local only, exact URL/Git pins with content identities,
Cargo-like resolver and lockfile, or a central registry.

Recommendation: Implement local paths first. Add remote acquisition only after
specifying immutable revision identity, hashes, trust, cache layout, offline
behavior, and update commands. Do not let compilation silently perform network
access.

Rationale: Exact version text does not by itself make mutable URLs reproducible.
Remote fetch also creates supply-chain and build-hermeticity obligations.

Consequences: Remote packages may enter after the compiler MVP even though the
manifest schema reserves them.

Revisit when: A registry or version ranges become release requirements.

## ADR-011: LLVM and Inkwell Version Pair

Status: `Blocked`

Context: Inkwell selects one LLVM major through Cargo features and remains
pre-1.0. LLVM IR, pass APIs, object emission, and debug information change by
major version.

Candidate options: Pin the newest supported pair, select an older long-lived
pair, use `llvm-sys` directly, or abstract multiple LLVM versions.

Recommendation: Run a compatibility spike over candidate released pairs, then
pin one Inkwell release, LLVM major, and CI installation. Do not support multiple
LLVM majors in the MVP.

Rationale: The [Inkwell repository](https://github.com/TheDan64/inkwell)
documents explicit LLVM feature selection and pre-1.0 compatibility. LLVM also
recommends setting target triple and data layout in frontend output.

Consequences: Toolchain updates become deliberate migrations with IR, ABI,
debug, and runtime validation.

Revisit when: Security, platform SDK, or upstream support requires an upgrade.

## ADR-012: Linker Integration

Status: `Proposed`

Context: Object emission alone does not supply startup objects, system
libraries, subsystem flags, debug options, or runtime linkage.

Candidate options: Embed LLD, invoke `link.exe` directly, invoke a configured C
compiler driver, or implement linking logic.

Recommendation: Begin with an explicit MSVC-compatible linker adapter and a
toolchain-discovery boundary. Keep command construction testable and leave an
LLD adapter possible.

Rationale: [LLD documentation](https://lld.llvm.org/) treats linking as a
separate object-level phase. Keeping it outside LLVM IR generation isolates
platform policy.

Consequences: CI needs a known Windows SDK and linker. Diagnostics must preserve
the command and translate common linker failures.

Revisit when: Bundled distribution, cross-compilation, or linker startup time
requires embedded LLD.

## ADR-013: Initial Standard Library

Status: `Proposed`

Context: General-purpose and AI applications eventually need broad libraries,
but a broad library would delay semantic and compiler validation.

Candidate options: Compiler intrinsics only, minimal vertical-slice library,
application-ready library, or AI/tensor stack in the first MVP.

Recommendation: Ship the minimum owned strings, vectors, option/result,
formatting, console I/O, time, tasks, cancellation, channels, and async I/O
needed by validation programs. Expand filesystem, networking, numerical, tensor,
and accelerator support after the core MVP.

Rationale: Small APIs expose ownership and concurrency design flaws earlier and
avoid stabilizing a large surface before the language can express it well.

Consequences: Early Realm is not yet a complete application platform. Library
growth requires API review against ownership, effects, and sendability.

Revisit when: A vertical slice cannot validate an essential target workload.

## ADR-014: Tooling Sequence

Status: `Proposed`

Context: All requested tools are valuable, but making all of them initial
release gates would postpone language validation.

Candidate options: Compiler only, diagnostics/test/debug gates, add formatter
and LSP, or require every tool in the first MVP.

Recommendation: Gate the compiler MVP on structured diagnostics, the language
test runner, and native debug information. Preserve lossless syntax and semantic
APIs for a later formatter and LSP. Stage the REPL, documentation generator,
profiler, sanitizers, race detector, and build integration afterward.

Rationale: The
[rust-analyzer architecture](https://rust-analyzer.github.io/book/contributing/architecture.html)
demonstrates the value of syntax and semantic API boundaries independent of LSP
transport.

Consequences: Editor completion and formatting are deferred, but their data
requirements influence early boundaries.

Revisit when: Contributor usability prevents progress without an earlier
formatter or LSP.

## ADR-015: Self-Hosting

Status: `Proposed`

Context: Self-hosting tests language expressiveness but creates bootstrap,
stability, build-time, and contributor constraints.

Candidate options: No self-hosting, pre-1.0 rewrite, post-1.0 goal, or long-term
experiment.

Recommendation: Keep the production compiler in Rust. Treat a Realm-written
compiler component as a long-term experiment after the language and package
model stabilize.

Rationale: The
[rustc bootstrapping guide](https://rustc-dev-guide.rust-lang.org/building/bootstrapping/intro.html)
shows the operational complexity introduced by self-hosting.

Consequences: MVP design should not depend on Realm compiling itself, although
compiler workloads may inform later language benchmarks.

Revisit when: Realm can build multi-package applications reproducibly and has a
documented compatibility policy.