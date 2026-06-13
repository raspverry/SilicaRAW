---
title: Task Cards
status: active
audience: agents
updated: 2026-06-12
source_of_truth: docs/wiki/roadmaps/post-alpha-product-roadmap.md
---

# Task Cards

## Summary

Task cards are small, LLM-readable pages for the next atomic work items. Use them instead of reading large roadmap or design-spec files when the task is already selected.

For Phase 14 through v1.0 ordering, use the [Post-Alpha Master Execution Plan](../roadmaps/post-alpha-master-execution-plan.md). If a future phase has no task cards yet, create them from that master plan before implementation.

## Phase 12

- [12.0: Phase 12 Design Gate](12.0-phase-12-design-gate.md)
- [12.1: Feature-Gated Core Image RAW Probe](12.1-core-image-raw-probe.md)
- [12.2: RAW Fixture Probe Harness](12.2-raw-fixture-probe-harness.md)
- [12.3: Core Image Support Matrix and LibRaw Gate](12.3-core-image-support-matrix.md)
- [12.4: Product RAW Decode API Contract](12.4-product-raw-decode-api-contract.md)
- [12.5: Legal RAW Fixture Evidence](12.5-legal-raw-fixture-evidence.md)
- [12.6: Product RAW Support Mapping](12.6-product-raw-support-mapping.md)

## Phase 13

- [13.0: Phase 13 Design Gate](13.0-phase-13-design-gate.md)
- [13.1: Color Fixture Source Review](13.1-color-fixture-source-review.md)
- [13.2: Local Color Fixture Corpus and Manifest](13.2-color-fixture-local-manifest.md)
- [13.3: Feature-Gated Color Profile Probe](13.3-feature-gated-color-profile-probe.md)
- [13.4: Color Probe Harness](13.4-color-probe-harness.md)
- [13.5: Color Support Matrix](13.5-color-support-matrix.md)
- [13.6: ICC Export Proof](13.6-icc-export-proof.md)
- [13.7: Color Metadata Contract](13.7-color-metadata-contract.md)
- [13.8: Explicit Export Color Options](13.8-explicit-export-color-options.md)

## Phase 14

- [14.0: Phase 14 Design Gate](14.0-phase-14-design-gate.md)
- [14.1: AppKit/Metal Viewer Bridge Contract](14.1-appkit-metal-viewer-bridge-contract.md)
- [14.2: Native Viewer Feature Gate and Module Shell](14.2-native-viewer-feature-gate.md)
- [14.3: Reserved Viewer Layout Handshake](14.3-reserved-viewer-layout-handshake.md)
- [14.4: Resize, Retina, and Lifecycle Proof](14.4-resize-retina-lifecycle-proof.md)
- [14.5: Viewer Input Ownership Proof](14.5-viewer-input-ownership-proof.md)
- [14.6: Render Request Boundary](14.6-render-request-boundary.md)
- [14.7: Disposable Texture Lifecycle Boundary](14.7-disposable-texture-lifecycle-boundary.md)
- [14.8: Viewer QA Harness and Checklist](14.8-viewer-qa-harness.md)

## Phase 15

- [15.0: Vertical Slice Evidence Gate](15.0-vertical-slice-evidence-gate.md)
- [15.1: Decoded Image Handoff Contract](15.1-decoded-image-handoff-contract.md)
- [15.2: RAW Decode to Preview Artifact](15.2-raw-decode-preview-artifact.md)
- [15.3: Metal Preview Display](15.3-metal-preview-display.md)
- [15.4: Exposure/Contrast Metal Draft Path](15.4-exposure-contrast-metal-draft-path.md)
- [15.5: RAW-Derived JPEG sRGB Export](15.5-raw-derived-jpeg-srgb-export.md)
- [15.6: RAW Export Manual Color QA](15.6-raw-export-manual-color-qa.md)

## Phase 16

- [16.0: Phase 16 Design Gate](16.0-phase-16-design-gate.md)
- [16.1: Undo, History, and Action Semantics Contract](16.1-undo-history-action-semantics-contract.md)
- [16.2: Edit History Persistence](16.2-edit-history-persistence.md)
- [16.3: Undo and Redo Core Commands](16.3-undo-redo-core-commands.md)
- [16.4: Develop History Panel Contract](16.4-develop-history-panel-contract.md)
- [16.5: Append-Only Action Log](16.5-append-only-action-log.md)
- [16.6: Sidecar Sync Status After History Commits](16.6-sidecar-sync-status-after-history.md)

## Phase 17

- [17.1.1: White Balance, Temperature, and Tint Mutators](17.1.1-white-balance-temperature-tint-mutators.md)
- [17.1.2: Tone Recovery Mutators](17.1.2-tone-recovery-mutators.md)
- [17.1.3: Color Presence Mutators](17.1.3-color-presence-mutators.md)
- [17.2.1: White Balance Preview, Commit, and Export Parity](17.2.1-white-balance-preview-commit-export-parity.md)
- [17.2.2: Tone Recovery Preview, Commit, and Export Parity](17.2.2-tone-recovery-preview-commit-export-parity.md)
- [17.2.3: Color Presence Preview, Commit, and Export Parity](17.2.3-color-presence-preview-commit-export-parity.md)
- [17.3: Real Histogram Cache and Display](17.3-real-histogram-cache-display.md)
- [17.4: Reset, Before/After, and Basic Presets](17.4-reset-before-after-basic-presets.md)
- [17.5: Develop P0 Visual QA](17.5-develop-p0-visual-qa.md)

## Notes for LLM Agents

Read one task card at a time. If a task card conflicts with an authoritative schema, dependency policy, ADR, or agent rule, the authoritative source wins.
