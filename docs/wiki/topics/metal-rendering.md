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

- Product native Metal viewer implementation.
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

## Links

- [Spike 001 Report](../../spikes/001-tauri-metal-viewer.md)
- [Phase 14 Product Metal Viewer Bridge Plan](../roadmaps/phase-14-metal-viewer-bridge-plan.md)
- [Metal Render Pipeline Specification](../../08_Metal_Render_Pipeline_Specification.md)
- [Architecture Patch](../../20_v1_1_Architecture_Patch.md)
- [System Architecture](../../03_System_Architecture.md)
- [Architecture Risks](../risks/architecture-risks.md)

## Notes for LLM Agents

Do not add a fake viewer or broad UI shell that assumes the Metal bridge is solved. Path B means the next viewer work must define the native AppKit/Metal bridge boundary first. Use M004/M005 as UI references only when an explicit viewer/UI task is active.
