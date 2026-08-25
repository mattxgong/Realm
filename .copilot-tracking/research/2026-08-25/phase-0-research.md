<!-- markdownlint-disable-file -->
---
title: Phase 0 Research
description: Scope, dependencies, and first implementation task for Realm Phase 0
ms.date: 2026-08-25
ms.topic: reference
---

## Scope and Success Criteria

Phase 0 consists of specifications and disposable spikes. It does not authorize
production compiler scaffolding. The first active task is `RLM-0001` because it
has no dependencies and gates `RLM-0002`, `RLM-0003`, and `RLM-0010`.

Initial success means:

* Owner decisions resolve the semantic choices in `QUE-001` through `QUE-003`
* Normative lexical and grammar drafts cover `REQ-001` through `REQ-007`
* Every rule maps to at least one conformance case
* A disposable prototype checks valid, invalid, and recoverable examples
* README contradictions are retired or explicitly superseded

## Evidence

The roadmap requires Phase 0 work to remain under `docs/specifications/` or a
temporary `spikes/` directory. The proposed project structure says the root
package remains unchanged until `RLM-0101`, after Phase 0 approval. The current
`src/` code is therefore evidence only and must not become production syntax
infrastructure.

Detailed syntax findings and alternatives are recorded in
`../subagents/2026-08-25/rlm-0001-syntax-research.md`.

## Selected Approach

Proceed one roadmap task at a time. Begin `RLM-0001` with owner decisions,
draft the two normative specifications, then add only the smallest disposable
checker needed to validate grammar examples and recovery assumptions.

Do not begin `RLM-0002`, `RLM-0003`, or production workspace changes until
`RLM-0001` acceptance evidence exists.

## Alternatives

Drafting all Phase 0 documents in parallel would expose more downstream issues
but violates task dependencies and risks inconsistent semantics. Beginning with
the LLVM spike is technically independent, but it does not unblock the source
and syntax critical path requested by the current repository state.

## Actionable Next Steps

1. Approve or revise the four open language choices.
2. Draft the lexical grammar with normative examples.
3. Draft the surface grammar and precedence/recovery tables.
4. Build a disposable validation spike.
5. Review requirement and contradiction coverage before accepting `RLM-0001`.