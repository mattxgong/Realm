---
title: Realm Requirements and Assumptions
description: Stable requirements, MVP scope, assumptions, open questions, and terminology for Realm
ms.date: 2026-08-25
ms.topic: reference
---

## Product Intent

Realm is a general-purpose, ahead-of-time compiled language for native
applications and AI-oriented programs. It prioritizes runtime performance,
memory safety, readable code for humans and AI-assisted development, integrated
tooling, and first-class concurrency.

The first supported environment is Windows x86-64 using the MSVC ABI and
linker ecosystem. Realm is implemented in Rust, emits LLVM IR through Inkwell,
and produces native COFF objects and executables.

## Goals

* Compile small Realm programs through every stage into native executables early
* Reject memory-unsafe aliasing and data races in safe code
* Make structured concurrency part of the type, effect, cleanup, and runtime
  designs from their first specifications
* Provide stable, independently testable compiler-stage boundaries
* Favor precise diagnostics and correctness over feature count
* Support nominal types, local inference, generics, algebraic data types, and
  exhaustive pattern matching
* Preserve room for a long-term self-hosting experiment without making it a
  release dependency

## Explicit Non-Goals

The following items are outside the compiler MVP:

* Source compatibility with the root README examples
* C++ or Rust ABI interoperability
* A stable Realm-native binary ABI
* Garbage collection or implicit shared ownership
* Actors, distributed concurrency, detached tasks, green-thread stack growth,
  or direct user control of the scheduler
* General metaprogramming or macros
* Regular expressions as a language pattern form
* A central package registry, version-range solver, or lockfile
* Cross-compilation or non-Windows targets
* Self-hosting
* Incremental compilation, a parallel front end, or distributed builds
* A production-complete AI/tensor ecosystem

## Minimum Viable Language

The MVP language includes:

* UTF-8 source and Unicode identifiers with a specified normalization policy
* Semicolon-terminated statements and Rust-influenced expressions
* Immutable `let` and mutable `var` local bindings
* Functions, lexical blocks, conditionals, loops, and explicit return values
* Signed and unsigned fixed-width integers, `bool`, a defined character type,
  `f32`, `f64`, unit, never, tuples, fixed arrays, slices, and owned strings
* Zero-based square-bracket indexing, negative indexing, and slicing
* Nominal structures and algebraic data types
* Exhaustive structural pattern matching over literals, tuples, structures, and
  algebraic data types
* Parametric generics with explicit constraints and full monomorphization
* Rust-like affine ownership, moves, shared borrows, exclusive borrows,
  deterministic destruction, and a narrow unsafe boundary
* Declared `throws` effects, typed exception values, stack unwinding, `catch`,
  and deterministic cleanup
* One source file per module, private-by-default declarations, explicit imports,
  acyclic module and package graphs, and no wildcard imports or re-exports
* Scoped async functions and child tasks, cooperative cancellation, typed
  channels, timers, and a minimal async I/O surface
* A narrow C ABI for primitives, pointers, callbacks, and explicitly C-layout
  aggregates

The first vertical slices intentionally implement subsets of this MVP. A
feature belongs to the MVP even when its dependency-ordered task appears after
the initial executable milestone.

## Deferred Language Features

* Traits or interfaces beyond the minimum generic-constraint mechanism
* Operator overloading and user-defined implicit conversions
* Closures that escape or cross tasks
* Shared ownership types, weak references, and cycle management
* Raw pointers and atomics outside a restricted unsafe module
* Async streams, channel selection, task priorities, deadlines, and detached
  work
* Actors and actor isolation
* Reflection, dynamic loading, plugins, and runtime generic metadata
* Hygienic declarative or procedural macros
* Regular-expression library integration
* Additional numeric widths, including `f16`, until target and ABI semantics
  are specified
* Formatter, language server, REPL, documentation generator, profiler,
  sanitizer integration, and race detector

## Functional Requirements

Requirement status is `Proposed` until the plan is approved. Future changes
must preserve IDs.

### Source and Syntax

* `REQ-001`: The compiler shall accept UTF-8 source and diagnose invalid byte
  sequences without panicking
* `REQ-002`: Identifiers shall support Unicode under a documented normalization
  and confusable-character policy
* `REQ-003`: Statements shall use semicolon termination; the grammar shall not
  infer terminators from line breaks
* `REQ-004`: The parser shall return a recoverable syntax representation plus
  ordered diagnostics for malformed input
* `REQ-005`: The syntax representation shall retain source ranges and enough
  trivia for future formatting and IDE use
* `REQ-006`: Indexing shall use `value[index]`, start at zero for non-negative
  indices, and define negative-index and bounds behavior before implementation
* `REQ-007`: Slicing shall define inclusive or exclusive bounds, omitted bounds,
  negative bounds, mutability, ownership, and out-of-range behavior before
  implementation

### Types and Memory Safety

* `REQ-008`: User-defined types shall have nominal identity independent of
  structural equality
* `REQ-009`: Type inference shall be local to a function body and shall not infer
  public signatures
* `REQ-010`: Safe Realm shall use affine ownership with move semantics and shall
  reject use after move
* `REQ-011`: Shared borrows shall prohibit mutation for their live extent, and
  exclusive borrows shall prohibit all competing access
* `REQ-012`: Borrow checking shall support non-lexical end points derived from
  control flow before the full MVP exits
* `REQ-013`: Values shall receive deterministic cleanup exactly once on normal
  return, early return, loop exit, and exception unwinding
* `REQ-014`: Unsafe operations shall be syntactically explicit and isolated from
  safe APIs by reviewable contracts
* `REQ-015`: Safe code shall not permit a data race; values crossing task or
  thread boundaries shall satisfy compiler-checked sendability constraints
* `REQ-016`: Integer overflow, division by zero, casts, floating-point behavior,
  character representation, and string encoding shall have specified debug and
  optimized semantics before code generation is accepted

### Generics and Data Types

* `REQ-017`: Realm shall support parametric functions and nominal data types
  with explicit generic parameters and constraints
* `REQ-018`: The MVP shall monomorphize every reachable concrete generic
  instance and diagnose unbounded or recursive instance expansion
* `REQ-019`: Generic checking shall occur against declared constraints rather
  than re-type-checking only after substitution
* `REQ-020`: Algebraic data types shall support payload variants and exhaustive
  pattern matching
* `REQ-021`: Pattern analysis shall detect unreachable cases and non-exhaustive
  matches without relying on LLVM

### Errors and Effects

* `REQ-022`: A function that may propagate an exception shall declare a `throws`
  effect in its signature
* `REQ-023`: Exception values shall be nominal typed values with a specified
  root protocol or constraint
* `REQ-024`: Unwinding shall run initialized-value cleanups in reverse ownership
  order and shall not unwind across an incompatible C frame
* `REQ-025`: An uncaught exception shall produce a diagnostic termination path;
  the policy for a second exception during cleanup shall be specified
* `REQ-026`: Exception and cancellation effects shall remain distinguishable,
  even if cancellation uses an internal exception-like lowering

### Modules and Packages

* `REQ-027`: One source file shall define one module, and its logical module path
  shall derive deterministically from its package-relative file path
* `REQ-028`: Declarations shall be private by default and may be exposed only by
  an explicit public marker
* `REQ-029`: Imports shall be explicit; wildcard imports and re-exports shall be
  rejected in the MVP
* `REQ-030`: Module and package dependency graphs shall be directed acyclic
  graphs, with cycle diagnostics that show the complete cycle
* `REQ-031`: Top-level initialization shall allow constants and expressions
  proven pure under specified rules; arbitrary top-level side effects shall be
  rejected
* `REQ-032`: A package manifest shall define name, version, authors, and pinned
  local-path, HTTPS, or Git dependencies
* `REQ-033`: Remote dependencies shall use a content identity and local cache;
  integrity, trust, and offline behavior shall be specified before remote fetch
  is implemented

### Compiler and Native Output

* `REQ-034`: The compiler shall expose deterministic stages for source loading,
  lexing, parsing, resolution, typing, ownership analysis, CFG construction,
  monomorphization, LLVM generation, object emission, and linking
* `REQ-035`: Each diagnostic shall carry a stable code, severity, message,
  primary source label, optional secondary labels, notes, and machine-applicable
  edits where valid
* `REQ-036`: LLVM generation shall consume validated monomorphic CFG IR and
  shall not resolve names or infer source-language types
* `REQ-037`: Every LLVM module shall set the Windows x86-64 target triple and
  data layout and pass LLVM verification before and after optimization
* `REQ-038`: The compiler shall emit COFF objects and invoke an
  MSVC-compatible linker with explicit runtime and system-library inputs
* `REQ-039`: The initial CLI shall support check-only compilation, executable
  production, stage dumps, diagnostic format selection, optimization level, and
  output path selection
* `REQ-040`: Compiler defects shall produce a nonzero exit status and a stable
  internal-error diagnostic without presenting partial output as valid

### Runtime, Concurrency, and FFI

* `REQ-041`: The runtime shall initialize allocation, exception, task,
  scheduler, timer, I/O, and shutdown services through a versioned internal ABI
* `REQ-042`: Child tasks shall be scoped: a scope shall not complete until all
  children have completed or cooperatively cancelled
* `REQ-043`: Failure escaping a child task shall cancel unfinished siblings and
  propagate through a deterministic parent observation point
* `REQ-044`: Cancellation shall propagate downward, remain idempotent and
  cooperative, and run ownership cleanups
* `REQ-045`: Typed channels shall define capacity, close, send, receive,
  cancellation, fairness, and synchronization semantics
* `REQ-046`: Task creation, completion, channel operations, and async I/O shall
  establish documented happens-before relationships
* `REQ-047`: The Windows scheduler shall multiplex Realm tasks over a bounded OS
  thread pool and define how blocking foreign calls are isolated
* `REQ-048`: Timers and MVP async I/O shall integrate with cancellation without
  retaining dead task frames or violating borrow lifetimes
* `REQ-049`: The C FFI shall require explicit ABI-safe types and unsafe call
  sites; Realm exceptions shall not cross the C ABI boundary

### Standard Library and Tooling

* `REQ-050`: The MVP library shall provide owned strings, basic vectors, result
  and option types, formatting, console I/O, time, tasks, cancellation, typed
  channels, and the minimum async I/O needed by validation programs
* `REQ-051`: Standard-library APIs shall expose ownership, borrowing,
  sendability, throwing, and cancellation behavior in their signatures
* `REQ-052`: The compiler shall emit Windows-compatible source debug information
  sufficient for native debugger line mapping and stack traces
* `REQ-053`: A language test runner shall compile and execute isolated Realm
  test programs and report structured results
* `REQ-054`: Compiler diagnostics, test runner behavior, and debug information
  are MVP release gates; formatter, LSP, REPL, documentation, profiler,
  sanitizer, race detector, and broader build integration are staged follow-ups

## Quality Requirements

* `REQ-055`: No malformed source input shall cause undefined behavior, memory
  unsafety, or an uncontrolled compiler panic
* `REQ-056`: Compiler-stage outputs and diagnostics shall be deterministic for
  identical source, options, target, and dependency inputs
* `REQ-057`: Every implemented feature shall have positive, negative,
  diagnostic, and relevant stage-level tests
* `REQ-058`: The test suite shall include parser fuzzing, ownership and type
  compile-fail cases, LLVM verification, ABI probes against C, and concurrency
  litmus tests
* `REQ-059`: CI shall pin Rust, LLVM, Inkwell, linker tooling, and target SDK
  versions needed for reproducible support
* `REQ-060`: Performance work shall use measured compile-time, runtime, binary
  size, scheduler, and allocation benchmarks; no unmeasured optimization shall
  weaken semantics
* `REQ-061`: Public language behavior shall not be stabilized until its
  specification, conformance tests, diagnostics, and compatibility policy are
  approved
* `REQ-062`: Crates that model source, diagnostics, syntax, semantics, and CFG IR
  shall not depend on Inkwell or the runtime implementation

## Assumptions

* `ASM-001`: Source files use UTF-8, line and column displays derive from UTF-8
  byte offsets, and identifiers use the Unicode policy resolved by `QUE-001`
* `ASM-002`: The initial target is `x86_64-pc-windows-msvc`; cross-compilation
  and other triples are deferred
* `ASM-003`: Full monomorphization is acceptable for the MVP despite compile-time
  and binary-size costs
* `ASM-004`: The runtime may use a small amount of audited unsafe Rust and
  platform FFI behind safe Realm-facing contracts
* `ASM-005`: Cooperative cancellation is acceptable; Realm will not forcibly
  stop user code at arbitrary instructions
* `ASM-006`: A minimal standard library is sufficient for the compiler MVP;
  filesystem, networking, numerical, tensor, and accelerator APIs expand later
* `ASM-007`: Strict dependency pins make a lockfile unnecessary in the MVP, but
  remote dependencies still need immutable content identity
* `ASM-008`: The compiler remains implemented in Rust through initial releases;
  self-hosting is an experiment after language stabilization

## Resolved Questions

The language owner approved these `RLM-0001` dispositions on 2026-08-25:

* `QUE-001`: Realm version 0 pins Unicode 17.0, uses NFC identifier identity,
  excludes default-ignorable code points, and warns for version-matched UTS 39
  confusable skeleton collisions and highly restrictive mixed-script defects
* `QUE-002`: Negative indices and bounds count from the end. Slices use an
  inclusive start, exclusive end, omitted bounds of zero and the sequence
  length, strict range checks, and no clamping or reversal
* `QUE-003`: Bounds and checked arithmetic failures use a non-catchable Realm
  panic in every build profile. The panic runs initialized cleanups before
  deterministic termination. `RLM-0002` owns the complete arithmetic matrix

## Unresolved Questions

These questions are not permission to improvise during implementation. Their
roadmap tasks must resolve them before dependent work starts.

* `QUE-004`: What exact unsafe operations, pointer types, and validity invariants
  are exposed to language users?
* `QUE-005`: Which generic constraint model provides the MVP behavior without
  prematurely committing to a broad trait system?
* `QUE-006`: What is the exception object layout, personality routine strategy,
  cleanup representation, and double-fault policy on Windows x64?
* `QUE-007`: Which types are implicitly sendable, how are negative sendability
  declarations represented, and can borrows cross suspension points?
* `QUE-008`: Which executor and Windows async I/O backend prototype best satisfy
  cancellation, debugger visibility, and bounded-resource requirements?
* `QUE-009`: Are channels bounded by default, and are rendezvous channels,
  closure, fairness, and multi-channel selection in the first release?
* `QUE-010`: Which operations qualify as pure during module initialization?
* `QUE-011`: How are HTTPS and Git dependency revisions authenticated, cached,
  updated, and made reproducible without a lockfile?
* `QUE-012`: Which AI workloads are representative enough to guide later
  numerical, accelerator, and FFI design?
* `QUE-013`: Which exact LLVM major and released Inkwell version form the first
  supported toolchain pair?

## Glossary

* **ABI**: Binary-level rules for calls, layouts, names, exceptions, and linking
* **ADT**: Algebraic data type composed from product and sum types
* **Affine type**: A value that may be used at most once unless borrowed
* **AOT**: Ahead-of-time compilation before program execution
* **Borrow**: Temporary access to a value without taking ownership
* **CFG**: Control-flow graph of basic blocks and explicit terminators
* **CST**: Concrete syntax tree that retains source structure and trivia
* **Cancellation**: Cooperative request for a task and descendants to stop work
* **Data race**: Unsynchronized concurrent conflicting access to one memory
  location, with at least one write
* **Definition identity**: Stable nominal compiler ID for a declared item
* **Diagnostic**: Structured compiler message with source labels and guidance
* **Effect**: Statically visible behavior such as throwing or suspension
* **HIR**: High-level intermediate representation after syntax desugaring and
  name resolution
* **Happens-before**: Ordering relation that makes memory effects observable
  across concurrent execution
* **LLVM IR**: Typed low-level intermediate representation consumed by LLVM
* **Monomorphization**: Creation of specialized code for concrete generic
  arguments
* **Non-lexical lifetime**: Borrow lifetime derived from use and control flow
  rather than block end alone
* **Ownership**: Responsibility for a value's lifetime and destruction
* **Sendability**: Type property permitting ownership transfer across tasks or
  threads
* **Structured concurrency**: Task hierarchy whose children cannot outlive
  their lexical or dynamic parent scope
* **Typed HIR**: Semantic expression representation with resolved types,
  coercions, calls, and effects
* **Unwinding**: Stack traversal that runs cleanups while propagating an
  exception

## Source Precedents

The staged IR and monomorphization requirements follow the separation described
by the [rustc compiler overview](https://rustc-dev-guide.rust-lang.org/overview.html).
Recoverable, syntax-only trees follow the invariants in the
[rust-analyzer syntax architecture](https://rust-analyzer.github.io/book/contributing/syntax.html).
Data-race and happens-before terminology follows the
[Go memory model](https://go.dev/ref/mem) while Realm adds static rejection.
Sendability follows the safety role of
[Rust `Send` and `Sync`](https://doc.rust-lang.org/nomicon/send-and-sync.html).
Scoped task lifetime, cancellation, and child error propagation follow the
[Swift structured concurrency precedent](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0304-structured-concurrency.md).
