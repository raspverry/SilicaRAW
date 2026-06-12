---
title: Phase 12 RAW Proof Plan
status: active
audience: all
updated: 2026-06-12
source_of_truth: docs/wiki/roadmaps/post-alpha-product-roadmap.md
---

# Phase 12 RAW Proof Plan

## Summary

Phase 12 proves Core Image RAW behavior on legal fixtures before SilicaRAW exposes RAW pixels in the product UI.

This is a proof and evidence phase. It is not a broad RAW editor implementation phase.

## Current Status

As of 2026-06-12:

- Task 12.0 design gate is complete.
- Task 12.1 feature-gated Core Image RAW probe is complete.
- Task 12.2 fixture harness exists and has run locally for legal classes A-D.
- Task 12.3 support matrix records fixture-backed Core Image support for classes A-D.
- Task 12.4 product RAW decode API contract is complete in a blocked state.
- Task 12.5.1 source review is complete for raw.pixls.us candidates A-D.
- Task 12.5.2 through Task 12.5.4 are complete locally for raw.pixls.us candidates A-D.
- Task 12.5.5 is complete: the support matrix records fixture-backed Core Image support for classes A-D.
- Task 12.6 is complete: successful A-D probe evidence can map into metadata-only product RAW support plans.

## Goal

Create fixture-backed evidence for which RAW fixture classes Core Image can open on the target macOS environment.

The output of this phase is a support decision, not user-visible RAW editing.

## Non-Goals

- Do not show RAW pixels in Library, Loupe, Develop, Export, or Metal surfaces.
- Do not add LibRaw unless a legal fixture-backed Core Image gap is recorded first.
- Do not add camera profile, lens correction, color correctness, or broad camera support claims.
- Do not commit user photos, generated fake RAW files, or unlicensed samples.
- Do not mutate original photo files.

## Task Sequence

### Task 12.0: Design Gate

- **Card:** [12.0 Phase 12 Design Gate](../tasks/12.0-phase-12-design-gate.md)
- **Status:** complete
- **Output:** Phase 12 scope, stop gates, and implementation plan exist before code work.
- **Validation:** `python3 scripts/harness/check-md-links.py`, `scripts/harness/check.sh`

### Task 12.1: Feature-Gated Core Image RAW Probe

- **Card:** [12.1 Core Image RAW Probe](../tasks/12.1-core-image-raw-probe.md)
- **Status:** complete
- **Output:** `silica-decode` exposes a non-default `core-image-raw-probe` path with structured probe results.
- **Validation:** `cargo test -p silica-decode --features core-image-raw-probe`

### Task 12.2: RAW Fixture Probe Harness

- **Card:** [12.2 RAW Fixture Probe Harness](../tasks/12.2-raw-fixture-probe-harness.md)
- **Status:** harness complete, fixture execution complete locally for classes A-D
- **Output:** A manifest-driven harness can run ignored Core Image probe tests and check original hash preservation.
- **Validation:** `SILICARAW_RAW_FIXTURE_MANIFEST=... scripts/harness/check-raw-probe-fixtures.py`

### Task 12.3: Core Image Support Matrix and LibRaw Gate

- **Card:** [12.3 Core Image Support Matrix](../tasks/12.3-core-image-support-matrix.md)
- **Status:** complete with fixture evidence for classes A-D
- **Output:** [RAW Decoding](../topics/raw-decoding.md) records classes A-D as `core_image_supported` and class E as `blocked_pending_evidence`.
- **Validation:** `python3 scripts/harness/check-md-links.py`, `python3 scripts/harness/check-cargo-deps.py`

### Task 12.4: Product RAW Decode API Contract

- **Card:** [12.4 Product RAW Decode API Contract](../tasks/12.4-product-raw-decode-api-contract.md)
- **Status:** complete as blocked pending evidence
- **Output:** Product RAW candidates return explicit blocked states; no pixels, UI display, export, cache writes, or original mutation.
- **Validation:** `cargo test -p silica-decode -p silica-core`

### Task 12.5: Legal RAW Fixture Evidence Gate

- **Card:** [12.5 Legal RAW Fixture Evidence](../tasks/12.5-legal-raw-fixture-evidence.md)
- **Status:** complete for classes A-D; class E remains pending source review
- **Output:** Legal fixture provenance review, ignored local fixture manifest, Core Image probe run, original-preservation proof, and support-matrix update.
- **Validation:** `SILICARAW_RAW_FIXTURE_MANIFEST=... scripts/harness/check-raw-probe-fixtures.py`, `scripts/harness/check.sh`

### Task 12.6: Product RAW Support Mapping

- **Card:** [12.6 Product RAW Support Mapping](../tasks/12.6-product-raw-support-mapping.md)
- **Status:** complete
- **Output:** Successful fixture-proven classes A-D can produce metadata-only `Supported` product RAW plans from probe results.
- **Validation:** `cargo test -p silica-decode -p silica-core`, `scripts/harness/check.sh`

## Task 12.5 Atomic Breakdown

### Task 12.5.1: Fixture Source Review

- **Location:** external source notes, [RAW Decoding](../topics/raw-decoding.md)
- **Description:** Identify candidate RAW files and prove license/provenance before download or use.
- **Acceptance Criteria:**
  - Each candidate has source URL, license, author/uploader when available, privacy status, and expected fixture class.
  - Unclear licenses are rejected.
  - User photos are not used unless explicitly approved and documented as local-only private fixtures.
- **Validation:** Manual source review recorded in docs.
- **Status:** complete for raw.pixls.us candidates A-D; fixture class E remains pending.

### Task 12.5.2: Ignored Local Fixture Corpus

- **Location:** ignored local directory such as `.tmp/legal-raw-fixtures/`
- **Description:** Store downloaded or approved local fixtures outside git.
- **Acceptance Criteria:**
  - No RAW media file is committed.
  - Fixture file SHA-256 is recorded.
  - Original file hash is stable before the probe.
- **Validation:** `git status --short` shows no fixture media staged.
- **Status:** complete locally for candidates A-D.

### Task 12.5.3: Local Fixture Manifest

- **Location:** ignored local manifest path supplied by `SILICARAW_RAW_FIXTURE_MANIFEST`
- **Description:** Create a local manifest that follows [Fixture Manifest Schema](../../../schemas/fixture_manifest.schema.json).
- **Acceptance Criteria:**
  - Paths are relative and safe.
  - `integrity.sha256` and `expected_source_hashes.sha256` match the fixture file.
  - RAW metadata and blocked decode gate fields are present.
- **Validation:** `scripts/harness/check-raw-probe-fixtures.py` reaches the probe instead of manifest validation failure.
- **Status:** complete locally at `.tmp/legal-raw-fixtures/raw-fixtures.json`.

### Task 12.5.4: Probe Run and Evidence Review

- **Location:** `crates/silica-decode`, `scripts/harness/`
- **Description:** Run the Core Image probe against the legal fixture manifest.
- **Acceptance Criteria:**
  - Probe result records backend, platform, macOS version, source hash, file size, modified time, status, dimensions, orientation, error category, and message.
  - Original SHA-256 remains unchanged after probing.
  - Failures are classified instead of hidden.
- **Validation:** `SILICARAW_RAW_FIXTURE_MANIFEST=... scripts/harness/check-raw-probe-fixtures.py`
- **Status:** complete locally for candidates A-D on macOS 26.4.

### Task 12.5.5: Matrix and Follow-Up Decision

- **Location:** [RAW Decoding](../topics/raw-decoding.md), [Wiki Log](../log.md)
- **Description:** Update the support matrix from actual probe evidence.
- **Acceptance Criteria:**
  - Successful fixture classes can become `core_image_supported`.
  - Failed fixture classes become `blocked_core_image_failed` or `blocked_unsupported_class`.
  - LibRaw remains deferred unless a concrete fixture-backed Core Image gap is recorded.
  - Any code change to product RAW decode support is a separate atomic task after the matrix update.
- **Validation:** `python3 scripts/harness/check-md-links.py`
- **Status:** complete for classes A-D; class E remains pending source review.

## Completion Gate

Phase 12 is not complete until one of these is true:

- Legal fixture evidence has been recorded and the support matrix reflects actual Core Image probe results.
- Maintainers explicitly accept that Phase 12 stops in a blocked-pending-evidence state because no legal fixture source is available.

## Links

- [Phase 12 RAW Proof Brief](../phases/phase-12-raw-proof.md)
- [Task Cards](../tasks/index.md)
- [RAW Decoding](../topics/raw-decoding.md)
- [Phase 12 RAW Proof Design](../../superpowers/specs/2026-06-11-phase-12-raw-proof-design.md)
- [Phase 12 RAW Proof Implementation Plan](../../superpowers/plans/2026-06-11-phase-12-raw-proof.md)
- [Post-Alpha Product Roadmap](post-alpha-product-roadmap.md)
- [Dependencies Policy](../../DEPENDENCIES.md)

## Notes for LLM Agents

Use this page to understand Phase 12 status and the next evidence gate. Use the specific task card before editing files. Do not infer RAW support from file extensions, and do not add RAW UI pixels from this phase.
