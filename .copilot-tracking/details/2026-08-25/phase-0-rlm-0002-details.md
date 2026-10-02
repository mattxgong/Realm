<!-- markdownlint-disable-file -->
---
title: Phase 0 RLM-0002 Implementation Details
description: File operations and validation for the Realm primitive and type-system specification
ms.date: 2026-08-25
ms.topic: reference
---

## Context

* Plan: `../../plans/2026-08-25/phase-0-rlm-0002-plan.instructions.md`
* Research: `../../research/2026-08-25/rlm-0002-type-system-research.md`
* Roadmap task: `RLM-0002`

## Phase B Details

Create one normative `type-system.md` with explicit value-domain, operator,
cast, coercion, identity, inference, constant, representation, and public-API
tables. Use synthesis/checking judgments and positive and negative examples.

Amend the accepted lexical and grammar specifications only where the owner
approved a correction. Preserve all unrelated syntax rules. Add the selected
policies to the resolved planning source without inventing new `QUE` IDs.

## Phase C Details

Create an isolated Rust crate that uses standard-library arithmetic and explicit
wide mathematical checks. It must not depend on host overflow mode or host casts
as the source of expected semantics. Tests cover all eight integer widths,
panic boundaries, representative cast classes, IEEE bit-pattern classes,
identity/coercion tables, and layout examples.

The executable prints one deterministic semantic digest. Debug and release runs
must print the same digest.

## Phase D Details

One review traces every scoped requirement to a rule, positive case, negative
case, and evidence row. A separate backend review checks Windows x64 layout and
LLVM lowering hazards, including poison, invalid float-to-integer conversion,
fast-math, and aggregate calling classification.

Do not request acceptance until both reviews are clean and strict validation is
current.
