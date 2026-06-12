---
title: Metal Rendering
status: active
audience: all
updated: 2026-06-12
source_of_truth: docs/08_Metal_Render_Pipeline_Specification.md
---

# Metal Rendering

## Summary

Metal rendering is central to SilicaRAW's product identity. The app should be Metal-first, not merely Metal-accelerated.

## Current Stance

- Interactive preview and adjustment rendering must use a Metal render path.
- Spike 001 recorded Path B: Tauri can host a native Metal view, but the viewer must be isolated behind a stronger AppKit/Metal bridge.
- Tauri remains viable as the shell for now, but product viewer work must not proceed through a naive overlay.
- Phase 5.1 adds preview readiness status only; it does not add the product Metal viewer.
- Phase 14 now has a dedicated product viewer bridge plan and task cards.

## Blocked Work

- Product native Metal viewer implementation beyond the feature-gated module shell.
- Shader passes.
- Texture manager implementation.
- Final viewer event ownership.
- Full render loop coordination beyond the proof delegate.
- Wiring M004/M005 preview surfaces to a real native viewer.

These items remain blocked until the matching Phase 14 task is active. Phase 14 starts with a bridge contract and feature gate before any product native module behavior.

## Spike 001 Result

- Result: Path B - partial success.
- Native `MTKView` output, Retina backing scale, resize behavior, render timing, and Rust-controlled clear/present loop were proven.
- Mouse down, drag, trackpad scroll, and magnify events were manually verified on the native surface.
- The first proof surface showed why native viewer layout must be reserved explicitly rather than overlaid on arbitrary web content.

## Phase 14 Route

[Phase 14 Product Metal Viewer Bridge Plan](../roadmaps/phase-14-metal-viewer-bridge-plan.md) is the active route for turning Path B into product bridge evidence.

Current planned order:

```txt
14.0 design gate
14.1 AppKit/Metal bridge contract
14.2 native viewer feature gate and module shell
14.3 reserved viewer layout handshake
14.4 resize, Retina, and lifecycle proof
14.5 viewer input ownership proof
14.6 render request boundary
14.7 disposable texture lifecycle boundary
14.8 viewer QA harness and checklist
```

The phase does not include RAW pixel display, exposure/contrast Metal rendering, shader pass breadth, or color correctness proof.

Task 14.2 added the non-default `native-metal-viewer` feature and `native_metal_viewer` product module shell. The shell records the product boundary only; it does not install an AppKit view, render pixels, allocate textures, or change UI behavior.

Task 14.3 added reserved viewer host markers for the Loupe and Develop viewer surfaces. The static UI exposes an inert feature-off `window.SilicaRAWViewerHost` geometry API that reports logical web coordinates, backing scale, drawable size, surface identity, and web-fallback state for the reserved host only.

Task 14.4 added feature-gated product module lifecycle proof state for reserved host geometry, drawable size changes, Retina backing scale, neutral render timing, and cleanup. The recorded proof output is explicitly `neutral_clear_only=true`; it does not install the final product image viewer, allocate product textures, render RAW pixels, or change the default feature-off app path. Manual evidence is recorded in [Native Viewer Lifecycle QA Checklist](../../../checklists/NATIVE_VIEWER_LIFECYCLE_QA.md).

## Product Bridge Contract

Task 14.1 defines the product AppKit/Metal viewer bridge contract. This contract continues Spike 001 Path B: Tauri remains the shell and control layer, but the viewer is isolated behind a product native module and a reserved layout handshake.

Reference status checked on 2026-06-12:

- Tauri `Window` 2.11.2 exposes desktop window size, scale, event, and main-thread execution APIs used by the bridge.
- `raw-window-handle` 0.6.2 exposes an AppKit `NSView` handle and records that AppKit view access is main-thread-only.
- Apple `MTKView`, `MTKViewDelegate`, `NSView.addSubview(_:)`, and `CAMetalLayer` remain the primary native surface references.

### Ownership Boundaries

| Owner | Owns | Must Not Own |
| --- | --- | --- |
| Web/Tauri shell | application chrome, Library/Develop panels, toolbars, dialogs, command invocation, current selection, and the logical viewer host rectangle | native Metal lifetime, texture lifetime, direct Metal device handles |
| Product native viewer module | macOS-only AppKit view installation, `MTKView`/MetalKit lifetime, drawable sizing, native viewer input within the reserved rectangle, and frame timing evidence | arbitrary web UI layout, catalog writes, sidecar writes, export output files |
| `silica-render` | typed render request contracts, render status, request identity, and future disposable texture inputs | Tauri window handles, AppKit objects, catalog persistence |
| `silica-core` | photo/edit state reads, validated edit graph commits, and command-level coordination | per-frame native viewer lifecycle or direct texture ownership |

The product native viewer module must be separate from `metal_host_spike.rs`. The spike module remains evidence code and cannot be renamed into production code.

### Reserved Layout Contract

- The web shell must expose one stable viewer host rectangle for the native viewer.
- The native viewer may be installed only inside that rectangle.
- The native viewer must not cover sidebars, top bars, bottom bars, inspector panels, export dialogs, popovers, buttons, sliders, text, or any other web control.
- Phase 14 does not allow web controls inside the native viewer rectangle unless a later explicit overlay task defines native/web layering behavior.
- A missing, zero-sized, stale, offscreen, or ambiguous rectangle disables the native viewer and leaves the web fallback visible.
- Geometry conversion must be explicit: logical web coordinates, AppKit points, backing scale, and drawable pixels are separate values and must be logged in the relevant proof tasks.

Task 14.3 implements the first layout handshake. Task 14.1 only fixes the contract.

### AppKit and Metal Lifecycle Contract

- AppKit handle access, native view creation, `addSubview`, frame updates, and view removal must run on the macOS main thread.
- The module must fail closed if the Tauri window, AppKit content view, Metal device, command queue, or drawable setup is unavailable.
- Install, resize/update, and uninstall paths must be explicit. Window close, app shutdown, feature disablement, and invalid geometry must remove or hide the native view.
- The native view, delegate, device, command queue, and texture state must have a documented retain/release owner before product code ships.
- Raw AppKit or Metal handles must not cross IPC into the web layer.
- The default desktop build must not compile or run the product native viewer.

Task 14.2 creates the feature-gated module shell. Task 14.4 proves resize, Retina scale, render timing, and lifecycle evidence through neutral product-module proof state.

### Event Ownership Contract

- Web UI owns all input outside the reserved viewer rectangle.
- Native viewer owns pointer, drag, scroll, and magnify events only while the event target is inside the reserved viewer rectangle.
- Native viewer input must not break sidebar selection, toolbar commands, inspector controls, export dialogs, text inputs, or keyboard shortcuts.
- Native focus is viewer-local. It must not capture global shortcuts unless a future explicit viewer shortcut task defines that behavior.
- Drag-and-drop import remains a web/shell workflow until a separate task changes it.
- Phase 14.5 must record event evidence for mouse down, drag, scroll, magnify, focus changes, and web-control interaction after native viewer installation.

### Render Request Boundary

Phase 14 does not render RAW pixels or exposure/contrast output in Metal. The bridge only prepares the boundary.

Future render requests must be typed, request-scoped, and disposable. They must include enough identity to discard stale work when selection, edit state, viewport, scale, or decoded image input changes. Latest accepted request wins; older in-flight requests must not replace the visible current request.

Render requests must not write catalog state, sidecars, original files, or export outputs. Edit commits remain Core/catalog work. Exports remain export-pipeline work.

Task 14.6 defines the typed render request shape.

### Disposable Texture Lifecycle Contract

- Native viewer textures and drawable resources are disposable runtime state.
- Texture state must be scoped to a viewer instance, a request identity, and a drawable size.
- Replacing the current photo, closing the library, clearing cache, resizing across scale factors, or closing the window must release stale texture state.
- Viewer textures are never the source of truth for edits, sidecars, catalog state, or exports.
- Phase 15 full-resolution export must not depend on the viewer texture cache.

Task 14.7 defines the first disposable texture/cache lifecycle boundary.

### Path B Stop Gates

Stop Phase 14 Path B product work and preserve the SwiftUI/AppKit fallback option if any of these happen:

- Current Tauri, raw-window-handle, AppKit, MetalKit, or QuartzCore docs contradict the bridge assumptions.
- AppKit view access cannot be kept on the main thread.
- The native view cannot be constrained to the reserved viewer rectangle.
- Native view z-order hides web controls or text.
- Native input leaks outside the viewer or breaks normal web controls.
- Product viewer code requires the spike module as a production dependency.
- The `native-metal-viewer` feature is enabled by default.
- The bridge cannot remove/release native state on close, invalid geometry, or feature disablement.
- Render or texture code writes catalog state, sidecars, originals, or export outputs.

If a stop gate fires, do not keep building a better overlay. Record the evidence and open fallback planning for a SwiftUI/AppKit shell with Rust Core.

## Links

- [Spike 001 Report](../../spikes/001-tauri-metal-viewer.md)
- [Phase 14 Product Metal Viewer Bridge Plan](../roadmaps/phase-14-metal-viewer-bridge-plan.md)
- [Metal Render Pipeline Specification](../../08_Metal_Render_Pipeline_Specification.md)
- [Architecture Patch](../../20_v1_1_Architecture_Patch.md)
- [System Architecture](../../03_System_Architecture.md)
- [Architecture Risks](../risks/architecture-risks.md)

## Notes for LLM Agents

Do not add a fake viewer or broad UI shell that assumes the Metal bridge is solved. Path B means the next viewer work must define the native AppKit/Metal bridge boundary first. Use M004/M005 as UI references only when an explicit viewer/UI task is active.
