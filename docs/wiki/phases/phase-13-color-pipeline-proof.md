---
title: Phase 13 Color Pipeline Proof Brief
status: complete
audience: all
updated: 2026-06-12
source_of_truth: docs/wiki/roadmaps/phase-13-color-pipeline-proof-plan.md
---

# Phase 13 Color Pipeline Proof Brief

## Summary

Phase 13 proves the local color pipeline before SilicaRAW expands preview or export color claims.

The phase starts with evidence planning and fixture legality, then adds feature-gated probes, harness checks, export ICC proof, schema-safe metadata propagation, and explicit export color options.

As of 2026-06-12, Phase 13 implementation tasks are complete. Color correctness claims remain blocked until approved tolerance results and executed manual visual review exist.

## Required Read Set

For all Phase 13 tasks, read:

- [Phase 13 Color Pipeline Proof Plan](../roadmaps/phase-13-color-pipeline-proof-plan.md)
- [Color Management](../topics/color-management.md)
- [Spike 003 Color-Managed Preview and Export](../../spikes/003-color-managed-preview-export.md)
- [Golden Image and Tolerance Policy](../../../checklists/GOLDEN_IMAGE_TOLERANCE_POLICY.md)
- The matching task card under [Task Cards](../tasks/index.md)

When the task touches fixture manifests, also read:

- [Schema Reference](../../19_Schema_Reference.md)
- [Fixture Manifest Schema](../../../schemas/fixture_manifest.schema.json)

When the task adds or changes dependencies, also read:

- [Dependencies Policy](../../DEPENDENCIES.md)

## Task Order

0. [Task 13.0: Phase 13 Design Gate](../tasks/13.0-phase-13-design-gate.md)
1. [Task 13.1: Color Fixture Source Review](../tasks/13.1-color-fixture-source-review.md)
2. [Task 13.2: Local Color Fixture Corpus and Manifest](../tasks/13.2-color-fixture-local-manifest.md)
3. [Task 13.3: Feature-Gated Color Profile Probe](../tasks/13.3-feature-gated-color-profile-probe.md)
4. [Task 13.4: Color Probe Harness](../tasks/13.4-color-probe-harness.md)
5. [Task 13.5: Color Support Matrix](../tasks/13.5-color-support-matrix.md)
6. [Task 13.6: ICC Export Proof](../tasks/13.6-icc-export-proof.md)
7. [Task 13.7: Color Metadata Contract](../tasks/13.7-color-metadata-contract.md)
8. [Task 13.8: Explicit Export Color Options](../tasks/13.8-explicit-export-color-options.md)

## Scope

- Review legal Color Class F fixtures.
- Run local ignored fixture manifests through feature-gated color probes.
- Record profile and transform evidence.
- Prove export ICC embedding before enabling broader options.
- Keep color metadata inside existing schemas and crate boundaries.

## Non-Goals

- Do not claim color correctness yet.
- Do not add RAW color behavior.
- Do not add HDR behavior.
- Do not mutate original files.
- Do not add dependencies without updating `docs/DEPENDENCIES.md`.
- Do not add broad fallback behavior.

## Validation Strategy

- Task 13.0: `python3 scripts/harness/check-md-links.py`, `scripts/harness/check.sh`
- Task 13.1: source review recorded in wiki docs
- Task 13.2: `git status --short` confirms fixture media stays ignored
- Task 13.3: `cargo test -p silica-render --features color-probe`
- Task 13.4: `SILICARAW_COLOR_FIXTURE_MANIFEST=... scripts/harness/check-color-probe-fixtures.py`
- Task 13.5: `python3 scripts/harness/check-md-links.py`, `python3 scripts/harness/check-cargo-deps.py`
- Task 13.6: color export tests and manual color QA checklist
- Task 13.7: `cargo test -p silica-edit -p silica-render -p silica-export`
- Task 13.8: export UI smoke and color export tests
- Before completion: `scripts/harness/check.sh`

## Notes for LLM Agents

Read this brief instead of the full post-alpha roadmap when doing Phase 13 work. Then read exactly one task card for the selected task.
