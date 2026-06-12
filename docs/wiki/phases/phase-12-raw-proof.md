---
title: Phase 12 RAW Proof Brief
status: completed
audience: all
updated: 2026-06-12
source_of_truth: docs/wiki/roadmaps/post-alpha-product-roadmap.md
---

# Phase 12 RAW Proof Brief

## Summary

Phase 12 proves Core Image RAW support on legal fixtures before SilicaRAW shows RAW pixels in the product UI.

This phase is a proof phase, not a broad RAW editor implementation phase.

## Current Result

Phase 12 completion gate is satisfied as of 2026-06-12.

Classes A-D have legal local fixture source review, ignored local fixture probe evidence, original-hash preservation evidence, support-matrix entries, and metadata-only product RAW support mapping. Class E remains blocked pending source review. No RAW pixels, UI RAW display, export expansion, cache generation, broad camera support claim, original mutation, or LibRaw dependency was added.

## Required Read Set

For all Phase 12 tasks, read:

- [Phase 12 RAW Proof Plan](../roadmaps/phase-12-raw-proof-plan.md)
- [Phase 12 RAW Proof Design](../../superpowers/specs/2026-06-11-phase-12-raw-proof-design.md)
- [Phase 12 RAW Proof Implementation Plan](../../superpowers/plans/2026-06-11-phase-12-raw-proof.md)
- [LLM Routing Index](../llm/index.md)
- [RAW Decoding](../topics/raw-decoding.md)
- [Architecture Patch](../../20_v1_1_Architecture_Patch.md)
- The matching task card under [Task Cards](../tasks/index.md)

When the task touches fixture manifests, also read:

- [Schema Reference](../../19_Schema_Reference.md)
- [Fixture Manifest Schema](../../../schemas/fixture_manifest.schema.json)

When the task adds or changes dependencies, also read:

- [Dependencies Policy](../../DEPENDENCIES.md)

## Task Order

0. [Task 12.0: Phase 12 Design Gate](../tasks/12.0-phase-12-design-gate.md)
1. [Task 12.1: Feature-Gated Core Image RAW Probe](../tasks/12.1-core-image-raw-probe.md)
2. [Task 12.2: RAW Fixture Probe Harness](../tasks/12.2-raw-fixture-probe-harness.md)
3. [Task 12.3: Core Image Support Matrix and LibRaw Gate](../tasks/12.3-core-image-support-matrix.md)
4. [Task 12.4: Product RAW Decode API Contract](../tasks/12.4-product-raw-decode-api-contract.md)
5. [Task 12.5: Legal RAW Fixture Evidence](../tasks/12.5-legal-raw-fixture-evidence.md)
6. [Task 12.6: Product RAW Support Mapping](../tasks/12.6-product-raw-support-mapping.md)

## Scope

- Add a macOS-only, non-default Core Image RAW probe.
- Run legal fixture manifests through the probe.
- Record structured, fixture-backed results.
- Decide which fixture classes can graduate from blocked to Core Image supported.
- Define the product decode API contract after evidence exists.
- Map fixture-backed probe results to metadata-only product RAW support plans.

## Non-Goals

- Do not show RAW pixels in the UI.
- Do not add LibRaw without a fixture-backed Core Image gap and dependency review.
- Do not add broad camera support claims.
- Do not add camera profile, lens correction, or final color correctness claims.
- Do not mutate original photo files.

## Stop Gates

Stop if:

- A legal RAW fixture source cannot be proven for evidence work.
- Core Image bindings require an undocumented dependency.
- A probe result cannot record source hash and original-preservation evidence.
- A task would move product UI RAW display ahead of the support matrix.

## Validation Strategy

- Task 12.1: `cargo test -p silica-decode --features core-image-raw-probe`
- Task 12.2: `SILICARAW_RAW_FIXTURE_MANIFEST=... cargo test -p silica-decode --features core-image-raw-probe -- --ignored`
- Task 12.3: `python3 scripts/harness/check-md-links.py` and `python3 scripts/harness/check-cargo-deps.py`
- Task 12.4: `cargo test -p silica-decode -p silica-core`
- Task 12.6: `cargo test -p silica-decode -p silica-core`
- Before completion: `scripts/harness/check.sh`

## Notes for LLM Agents

Read this brief instead of the full post-alpha roadmap when doing Phase 12 work. Then read exactly one task card for the selected task.
