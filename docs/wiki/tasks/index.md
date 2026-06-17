---
title: Task Cards
status: active
audience: agents
updated: 2026-06-17
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

## Phase 18

Phase 18 is complete. Prefer [Phase 18 Summary](../phases/phase-18-summary.md) unless a task directly changes or audits a specific completed Phase 18 slice.

- [18.1.1: Tone Curve Mutators](18.1.1-tone-curve-mutators.md)
- [18.1.2: Tone Curve Preview, Commit, and Export Parity](18.1.2-tone-curve-preview-commit-export-parity.md)
- [18.1.3: Tone Curve Panel UI and QA](18.1.3-tone-curve-panel-ui-qa.md)
- [18.2.1: HSL Color Mixer Mutators](18.2.1-hsl-color-mixer-mutators.md)
- [18.2.2: HSL Preview, Commit, and Export Parity](18.2.2-hsl-preview-commit-export-parity.md)
- [18.2.3: HSL Panel UI and QA](18.2.3-hsl-panel-ui-qa.md)
- [18.3.1: Detail Mutators](18.3.1-detail-mutators.md)
- [18.3.2: Detail Preview and Export Boundary](18.3.2-detail-preview-export-boundary.md)
- [18.3.3: Detail Panel UI and QA](18.3.3-detail-panel-ui-qa.md)
- [18.4.1: Lens and Geometry Mutators](18.4.1-lens-geometry-mutators.md)
- [18.4.2: Geometry Preview and Export Parity](18.4.2-geometry-preview-export-parity.md)
- [18.4.3: Lens Geometry Panel UI and QA](18.4.3-lens-geometry-panel-ui-qa.md)
- [18.5.1: Edit Clipboard Contract](18.5.1-edit-clipboard-contract.md)
- [18.5.2: Batch Sync History](18.5.2-batch-sync-history.md)
- [18.5.3: Copy Paste Batch UI and QA](18.5.3-copy-paste-batch-ui-qa.md)

## Phase 19

Phase 19 is complete. Prefer [Phase 19 Manual Masks](../phases/phase-19-manual-masks.md) unless changing completed mask behavior.

- [19.1: Mask Schema and Edit Graph Audit](19.1-mask-schema-edit-graph-audit.md)
- [19.2: Linear and Radial Manual Masks](19.2-linear-radial-manual-masks.md)
- [19.3: Brush Mask Storage and Rasterization](19.3-brush-mask-storage-rasterization.md)
- [19.4: Mask Compositing in Preview and Export](19.4-mask-compositing-preview-export.md)
- [19.5: Mask Editor Visual QA](19.5-mask-editor-visual-qa.md)

## Phase 20

- [20.1: Export Settings Model and Presets](20.1-export-settings-model-presets.md)
- [20.2: PNG and TIFF Export](20.2-png-tiff-export.md)
- [20.3: Export Metadata Policy](20.3-export-metadata-policy.md)
- [20.4: Batch Export Progress and Recent Exports](20.4-batch-export-progress-recent-exports.md)
- [20.5: Display P3 Export Enablement](20.5-display-p3-export-enablement.md)

## Phase 21

- [21.1: Preferences Information Architecture](21.1-preferences-information-architecture.md)
- [21.2: Appearance Preferences](21.2-appearance-preferences.md)
- [21.3: Library and Cache Preferences](21.3-library-cache-preferences.md)
- [21.4: Color and Export Defaults](21.4-color-export-defaults.md)
- [21.5: Advanced Agent Access Preferences](21.5-advanced-agent-access-preferences.md)

## Phase 22

- [22.1: Expanded Visual QA Surface Set](22.1-expanded-visual-qa-surface-set.md)
- [22.2: Library Scale Benchmarks](22.2-library-scale-benchmarks.md)
- [22.3: Migration and Backup Failure Tests](22.3-migration-backup-failure-tests.md)
- [22.4: RAW and Metal Performance Profiling](22.4-raw-metal-performance-profiling.md)
- [22.5: Manual Photographer QA Checklist](22.5-manual-photographer-qa-checklist.md)

## Phase 24

- [24.1: MLX Runtime Spike](24.1-mlx-runtime-spike.md)
- [24.2: Model Manifest Validation](24.2-model-manifest-validation.md)

## Notes for LLM Agents

Read one task card at a time. If a task card conflicts with an authoritative schema, dependency policy, ADR, or agent rule, the authoritative source wins.
