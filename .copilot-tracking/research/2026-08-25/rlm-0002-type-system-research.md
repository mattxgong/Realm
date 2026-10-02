<!-- markdownlint-disable-file -->
---
title: RLM-0002 Type-System Research
description: Approved semantic and target-layout direction for the Realm primitive and type-system specification
ms.date: 2026-08-25
ms.topic: reference
---

## Scope

`RLM-0002` specifies `REQ-008`, `REQ-009`, and `REQ-016` without implementing
the production type checker. It owns primitive value domains, literal typing,
operators, casts, type identity, local inference, coercions, aggregate types,
constant eligibility, public type surfaces, and the initial Windows x64 layout
boundary.

Detailed alternatives, platform evidence, and official sources are recorded in
`../subagents/2026-08-25/rlm-0002-type-system-research.md`.

## Approved Direction

The language owner approved all recommended policy bundles:

* Fixed-width integer primitives plus `f32`, `f64`, `bool`, Unicode-scalar
  `char`, unit, and never; no `f16`, pointer-sized, or 128-bit primitives
* Contextual numeric literal typing with a closed suffix set and `i32`/`f64`
  fallback
* Checked integer arithmetic and checked `as` casts with identical debug and
  optimized behavior
* Strict IEEE-754 binary32 and binary64 behavior without ordinary fast-math
* Nominal declarations, structural built-in aggregates, body-local inference,
  explicit public signatures, and a closed implicit-coercion set
* Unsized `[T]` behind references, bounded typed constants, zero-length arrays,
  zero-byte storage for zero-sized aggregates, and no string-valued constants
* Initial `String` storage as `{ptr, u64 len, u64 cap}` on Windows x64

## Accepted Amendments

The owner authorized these narrow corrections to the accepted syntax contract:

* A numeric token may combine with an immediately contiguous exact suffix token
* Cast expressions use `expression as Type`
* `[T]` is unsized and appears by value only behind `&` or `&mut`
* A public function must spell its return type, including `-> ()`
* Direct slice parameters use `&[T]` or `&mut [T]`

## Selected Validation

Add a disposable standalone Rust table oracle under `spikes/type-system/`. It
models closed semantic tables, integer boundaries, checked casts, type identity,
coercion cases, and target layout examples. It does not parse Realm, infer
types, lower LLVM, or become a production dependency.

Run the oracle in debug and release profiles. Both profiles must report the
same semantic digest. Independent language and Windows x64/LLVM reviews must
find no implicit semantic or representation decision.

## Boundaries

Generic constraints remain in `RLM-0004`. Moves, copy eligibility, loans,
lifetimes, destruction, unsafe pointers, and zero-sized-place identity remain
in `RLM-0005`. Exception ABI, async identity, sendability, C layout, and stable
package identity remain downstream.
