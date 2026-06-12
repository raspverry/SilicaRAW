---
title: Phase 14 Product Metal Viewer Bridge Brief
status: active
audience: all
updated: 2026-06-12
source_of_truth: docs/wiki/roadmaps/phase-14-metal-viewer-bridge-plan.md
---

# Phase 14 Product Metal Viewer Bridge Brief

## Summary

Phase 14 replaces the Spike 001 proof surface with a product AppKit/Metal viewer bridge foundation.

The phase starts with a design gate and bridge contract, then adds a feature-gated product module, reserved layout handshake, lifecycle and input proof, render-request boundaries, texture lifecycle boundaries, and viewer QA.

## Required Read Set

For all Phase 14 tasks, read:

- [Phase 14 Product Metal Viewer Bridge Plan](../roadmaps/phase-14-metal-viewer-bridge-plan.md)
- [Metal Rendering](../topics/metal-rendering.md)
- [Spike 001 Tauri + Native Metal Viewer](../../spikes/001-tauri-metal-viewer.md)
- [Architecture Patch](../../20_v1_1_Architecture_Patch.md)
- [Metal Render Pipeline Specification](../../08_Metal_Render_Pipeline_Specification.md)
- The matching task card under [Task Cards](../tasks/index.md)

When the task touches Tauri, AppKit, Metal, MetalKit, QuartzCore, or raw window handles, also verify the external reference gate in the Phase 14 plan.

When the task adds or changes dependencies, also read:

- [Dependencies Policy](../../DEPENDENCIES.md)

## Task Order

0. [Task 14.0: Phase 14 Design Gate](../tasks/14.0-phase-14-design-gate.md)
1. [Task 14.1: AppKit/Metal Viewer Bridge Contract](../tasks/14.1-appkit-metal-viewer-bridge-contract.md)
2. [Task 14.2: Native Viewer Feature Gate and Module Shell](../tasks/14.2-native-viewer-feature-gate.md)
3. [Task 14.3: Reserved Viewer Layout Handshake](../tasks/14.3-reserved-viewer-layout-handshake.md)
4. [Task 14.4: Resize, Retina, and Lifecycle Proof](../tasks/14.4-resize-retina-lifecycle-proof.md)
5. [Task 14.5: Viewer Input Ownership Proof](../tasks/14.5-viewer-input-ownership-proof.md)
6. [Task 14.6: Render Request Boundary](../tasks/14.6-render-request-boundary.md)
7. [Task 14.7: Disposable Texture Lifecycle Boundary](../tasks/14.7-disposable-texture-lifecycle-boundary.md)
8. [Task 14.8: Viewer QA Harness and Checklist](../tasks/14.8-viewer-qa-harness.md)

## Scope

- Continue Spike 001 Path B deliberately.
- Reserve native viewer layout explicitly.
- Add product viewer bridge code separately from spike proof code.
- Prove lifecycle, resize, Retina, timing, and input ownership.
- Define render request and disposable texture boundaries.
- Keep fallback rules and manual QA visible.

## Non-Goals

- No RAW pixel display.
- No exposure/contrast Metal preview path.
- No shader pass library.
- No color correctness claim.
- No product-side mutation of original photo files.
- No dependency changes without dependency documentation.

## Validation Strategy

- Task 14.0: `python3 scripts/harness/check-md-links.py`, `scripts/harness/check.sh`
- Task 14.1: Markdown link checks and scope guardrails
- Task 14.2: `cargo check -p silica-desktop --features native-metal-viewer`
- Task 14.3: static UI contract and feature-gated desktop tests
- Task 14.4: feature-gated desktop tests plus manual macOS runbook evidence
- Task 14.5: input smoke evidence plus manual checklist
- Task 14.6: render and desktop feature-gated tests
- Task 14.7: lifecycle/cache cleanup tests
- Task 14.8: full harness plus manual viewer QA checklist
- Before completion: `scripts/harness/check.sh`

## Notes for LLM Agents

Read this brief instead of the full post-alpha roadmap when doing Phase 14 work. Then read exactly one task card for the selected task. Stop if a task would treat the spike module as production viewer code by rename.
