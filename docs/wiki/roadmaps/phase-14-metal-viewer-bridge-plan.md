---
title: Phase 14 Product Metal Viewer Bridge Plan
status: active
audience: all
updated: 2026-06-12
source_of_truth: docs/wiki/roadmaps/post-alpha-product-roadmap.md
---

# Phase 14 Product Metal Viewer Bridge Plan

## Summary

Phase 14 turns Spike 001 Path B into a product-grade native viewer bridge boundary.

This phase is not the full RAW/color/Metal vertical slice. It proves that SilicaRAW can reserve a native AppKit/Metal viewer region, host a feature-gated product module separate from the spike, own lifecycle and input deliberately, and define the render-request boundary needed by Phase 15.

For Phase 14 through v1.0 sequencing, use the [Post-Alpha Master Execution Plan](post-alpha-master-execution-plan.md). This Phase 14 plan remains the detailed Wave A execution plan.

## Current Status

As of 2026-06-12:

- Spike 001 recorded Path B: Tauri remains viable, but the product viewer needs a stronger AppKit/Metal bridge.
- The existing `metal_host_spike.rs` is proof code only and must not become the product viewer by rename.
- Phase 12 RAW proof and Phase 13 color proof are complete enough for Metal bridge planning.
- Task 14.0 design gate is complete.
- Task 14.1 bridge contract is complete.
- Task 14.2 feature-gated product module shell is complete.
- Task 14.3 reserved viewer layout handshake is complete.
- Task 14.4 resize, Retina, and lifecycle proof is complete.
- No viewer input proof, texture lifecycle, or viewer QA checklist exists yet.

## Goal

Create a product viewer bridge foundation that proves:

- a reserved native viewer region that does not cover arbitrary web UI
- feature-gated macOS-only product viewer code separate from Spike 001
- explicit AppKit/Metal lifecycle ownership
- resize, Retina scale, render timing, and input evidence
- render-request and disposable texture-cache boundaries
- automated and manual QA routes for viewer layout and input

## Non-Goals

- Do not implement RAW pixel display in this phase.
- Do not move exposure/contrast rendering to Metal in this phase.
- Do not add shader pass breadth beyond the minimum bridge proof.
- Do not claim color correctness from a native surface.
- Do not remove the SwiftUI/AppKit fallback rule.
- Do not add dependencies without updating `docs/DEPENDENCIES.md`.
- Do not mutate original photo files.

## External Reference Gate

Before implementation tasks that touch Tauri/AppKit/Metal APIs, verify current primary docs:

- Tauri Rust window APIs: <https://docs.rs/tauri/latest/tauri/window/struct.Window.html>
- raw-window-handle AppKit handle constraints: <https://docs.rs/raw-window-handle/latest/raw_window_handle/struct.AppKitWindowHandle.html>
- Apple `MTKView`: <https://developer.apple.com/documentation/MetalKit/MTKView>
- Apple `MTKViewDelegate`: <https://developer.apple.com/documentation/MetalKit/MTKViewDelegate>
- Apple `NSView.addSubview(_:)`: <https://developer.apple.com/documentation/appkit/nsview/addsubview%28_%3A%29>
- Apple `CAMetalLayer`: <https://developer.apple.com/documentation/QuartzCore/CAMetalLayer>

If these docs conflict with Spike 001 assumptions, stop and update the bridge contract before code work.

## Task Sequence

### Task 14.0: Phase 14 Design Gate

- **Card:** [14.0 Phase 14 Design Gate](../tasks/14.0-phase-14-design-gate.md)
- **Status:** complete
- **Output:** Phase 14 scope, official-doc gate, fallback rule, and atomic task order are recorded before product viewer code.
- **Validation:** `python3 scripts/harness/check-md-links.py`, `scripts/harness/check.sh`

### Task 14.1: AppKit/Metal Viewer Bridge Contract

- **Card:** [14.1 AppKit/Metal Viewer Bridge Contract](../tasks/14.1-appkit-metal-viewer-bridge-contract.md)
- **Status:** complete
- **Output:** [Metal Rendering](../topics/metal-rendering.md) records reserved layout, AppKit lifecycle ownership, event ownership, render request boundaries, and Path B stop gates.
- **Validation:** `python3 scripts/harness/check-md-links.py`

### Task 14.2: Native Viewer Feature Gate and Module Shell

- **Card:** [14.2 Native Viewer Feature Gate and Module Shell](../tasks/14.2-native-viewer-feature-gate.md)
- **Status:** complete
- **Output:** A macOS-only `native-metal-viewer` feature and product module skeleton exist separately from `metal_host_spike.rs`.
- **Validation:** `cargo check -p silica-desktop`, `cargo check -p silica-desktop --features native-metal-viewer`

### Task 14.3: Reserved Viewer Layout Handshake

- **Card:** [14.3 Reserved Viewer Layout Handshake](../tasks/14.3-reserved-viewer-layout-handshake.md)
- **Status:** complete
- **Output:** The web shell exposes a stable viewer host rectangle and the native bridge consumes that geometry without overlapping panels or controls.
- **Validation:** static UI contract, viewer layout smoke, `cargo test -p silica-desktop --features native-metal-viewer`

### Task 14.4: Resize, Retina, and Lifecycle Proof

- **Card:** [14.4 Resize Retina Lifecycle Proof](../tasks/14.4-resize-retina-lifecycle-proof.md)
- **Status:** complete
- **Output:** Product viewer module records resize, drawable size, backing scale, install/uninstall lifecycle, and render timing evidence through neutral feature-gated proof state.
- **Validation:** `cargo test -p silica-desktop --features native-metal-viewer`, manual macOS runbook evidence

### Task 14.5: Viewer Input Ownership Proof

- **Card:** [14.5 Viewer Input Ownership Proof](../tasks/14.5-viewer-input-ownership-proof.md)
- **Status:** next
- **Output:** Native viewer owns only viewer-surface mouse/drag/scroll/magnify events while web UI controls retain normal interaction.
- **Validation:** input smoke logs and manual QA checklist entries

### Task 14.6: Render Request Boundary

- **Card:** [14.6 Render Request Boundary](../tasks/14.6-render-request-boundary.md)
- **Status:** planned
- **Output:** `silica-render` and desktop bridge agree on a typed preview render request where latest request wins and no catalog state is written.
- **Validation:** `cargo test -p silica-render -p silica-desktop --features native-metal-viewer`

### Task 14.7: Disposable Texture Lifecycle Boundary

- **Card:** [14.7 Disposable Texture Lifecycle Boundary](../tasks/14.7-disposable-texture-lifecycle-boundary.md)
- **Status:** planned
- **Output:** The native viewer has a minimal disposable texture/cache lifecycle contract ready for Phase 15 image pixels.
- **Validation:** lifecycle tests and cache cleanup checks

### Task 14.8: Viewer QA Harness and Checklist

- **Card:** [14.8 Viewer QA Harness and Checklist](../tasks/14.8-viewer-qa-harness.md)
- **Status:** planned
- **Output:** Manual and automated QA cover viewer layout, input, resize, Retina, external display movement, and UI responsiveness.
- **Validation:** `scripts/harness/check.sh`, manual checklist output

## Completion Gate

Phase 14 is complete only when all of these are true:

- Product bridge contract exists and explicitly continues Spike 001 Path B.
- `metal_host_spike.rs` remains proof code, not the product module.
- `native-metal-viewer` builds only when explicitly enabled.
- Web UI reserves a viewer region and does not hide controls under a native overlay.
- Resize, Retina scale, lifecycle, render timing, and input evidence are recorded.
- Render request and texture lifecycle boundaries are typed and documented.
- Viewer QA checklist exists and has run instructions.
- SwiftUI/AppKit fallback remains available if Path B product bridge evidence fails.

## Links

- [Phase 14 Product Metal Viewer Bridge Brief](../phases/phase-14-product-metal-viewer-bridge.md)
- [Metal Rendering](../topics/metal-rendering.md)
- [Spike 001 Tauri + Native Metal Viewer](../../spikes/001-tauri-metal-viewer.md)
- [Architecture Patch](../../20_v1_1_Architecture_Patch.md)
- [Metal Render Pipeline Specification](../../08_Metal_Render_Pipeline_Specification.md)
- [Post-Alpha Master Execution Plan](post-alpha-master-execution-plan.md)
- [Post-Alpha Product Roadmap](post-alpha-product-roadmap.md)

## Notes for LLM Agents

Treat Phase 14 as bridge proof, not full viewer rendering. Do not skip directly to Phase 15 RAW/color/Metal pixels. Use one task card at a time and keep every change atomic and committable.
