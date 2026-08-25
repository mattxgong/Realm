<!-- markdownlint-disable-file -->
---
title: Phase 0 RLM-0001 Implementation Details
description: File operations, dependencies, and checks for the first Realm Phase 0 task
ms.date: 2026-08-25
ms.topic: reference
---

## Context

* Plan: `../../plans/2026-08-25/phase-0-rlm-0001-plan.instructions.md`
* Research: `../../research/2026-08-25/phase-0-research.md`
* Detailed findings:
  `../../research/subagents/2026-08-25/rlm-0001-syntax-research.md`
* Roadmap task: `RLM-0001`

## Phase A Details

Present four independent owner decisions. Each question must state the
recommended option, alternatives, consequences, and relative complexity. No
normative specification language is written for an unresolved choice.

Success requires explicit answers for all four choices.

## Phase B Details

Create `docs/specifications/` only when owner decisions are available. The
lexical document owns source decoding, trivia, comments, identifiers, keywords,
literal boundaries, and lexical errors. The grammar document owns declarations,
statements, expressions, types, patterns, precedence, semicolons, indexing,
slicing, and parser recovery anchors.

Examples must distinguish valid input, invalid input, expected diagnostics, and
recovery continuation. Normative keywords such as MUST, MUST NOT, SHOULD, and
MAY should have consistent meanings.

## Phase C Details

Prefer a minimal Rust spike isolated from the root package. It may use a small
parser library only if dependency cost is recorded and the spike remains
disposable. The checker should read specification fixtures, assert expected
acceptance or diagnostics, prove complete byte coverage, and demonstrate
continued parsing after three independent errors.

Run formatting, linting, and tests inside the spike. Do not add it to a future
production workspace.

## Phase D Details

Build a requirement coverage table and a README reconciliation table in the
specifications. Record any semantic rule intentionally delegated to a later
Phase 0 task. A delegated rule must identify the blocking task and cannot leave
syntax ambiguous.