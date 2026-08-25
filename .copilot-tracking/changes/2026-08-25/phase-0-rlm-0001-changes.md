<!-- markdownlint-disable-file -->
---
title: Phase 0 RLM-0001 Changes
description: Implementation record for the Realm lexical and surface grammar task
ms.date: 2026-08-25
ms.topic: reference
---

## Related Plan

`../../plans/2026-08-25/phase-0-rlm-0001-plan.instructions.md`

## Implementation Date

2026-08-25

## Summary

`RLM-0001` now has normative lexical and surface grammar specifications,
approved dispositions for its owner questions, and bounded executable evidence.
The language owner accepted the implementation after strict validation and two
independent reviews.

## Added

* `.copilot-tracking/research/2026-08-25/phase-0-research.md`
* `.copilot-tracking/research/subagents/2026-08-25/rlm-0001-syntax-research.md`
* `.copilot-tracking/research/subagents/2026-08-25/rlm-0001-grammar-review.md`
* `.copilot-tracking/details/2026-08-25/phase-0-rlm-0001-details.md`
* `.copilot-tracking/plans/2026-08-25/phase-0-rlm-0001-plan.instructions.md`
* `.copilot-tracking/plans/logs/2026-08-25/phase-0-rlm-0001-log.md`
* `.copilot-tracking/changes/2026-08-25/phase-0-rlm-0001-changes.md`
* `.copilot-tracking/reviews/2026-08-25/phase-0-rlm-0001-plan-review.md`
* `docs/specifications/lexical-grammar.md`
* `docs/specifications/grammar.md`
* `spikes/syntax-grammar/.gitignore`
* `spikes/syntax-grammar/Cargo.toml`
* `spikes/syntax-grammar/Cargo.lock`
* `spikes/syntax-grammar/src/main.rs`

## Modified

* `docs/planning/README.md`
* `docs/planning/requirements-and-assumptions.md`

## Removed

No files were removed.

## Deviations

The checker validates representative syntax and Unicode cases without building
the measured lossless CST assigned to `RLM-0003` or the complete Unicode and
UTS 39 implementation assigned to `RLM-0104`. This boundary is explicit in both
specifications and the checker output.

## Release Summary

Realm Phase 0 has a review-ready source and syntax contract for `REQ-001`
through `REQ-007`. Owner acceptance now unlocks the dependent Phase 0 tasks.
