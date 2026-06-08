---
title: Metal Rendering
status: active
audience: all
updated: 2026-06-08
source_of_truth: docs/08_Metal_Render_Pipeline_Specification.md
---

# Metal Rendering

## Summary

Metal rendering is central to SilicaRAW's product identity. The app should be Metal-first, not merely Metal-accelerated.

## Current Stance

- Interactive preview and adjustment rendering must use a Metal render path.
- Spike 001 recorded Path B: Tauri can host a native Metal view, but the viewer must be isolated behind a stronger AppKit/Metal bridge.
- Tauri remains viable as the shell for now, but product viewer work must not proceed through a naive overlay.

## Blocked Work

- Product native Metal viewer implementation.
- Shader passes.
- Texture manager implementation.
- Final viewer event ownership.
- Full render loop coordination beyond the proof delegate.

## Spike 001 Result

- Result: Path B - partial success.
- Native `MTKView` output, Retina backing scale, resize behavior, render timing, and Rust-controlled clear/present loop were proven.
- Mouse down, drag, trackpad scroll, and magnify events were manually verified on the native surface.
- The first proof surface showed why native viewer layout must be reserved explicitly rather than overlaid on arbitrary web content.

## Links

- [Spike 001 Report](../../spikes/001-tauri-metal-viewer.md)
- [Metal Render Pipeline Specification](../../08_Metal_Render_Pipeline_Specification.md)
- [Architecture Patch](../../20_v1_1_Architecture_Patch.md)
- [System Architecture](../../03_System_Architecture.md)
- [Architecture Risks](../risks/architecture-risks.md)

## Notes for LLM Agents

Do not add a fake viewer or broad UI shell that assumes the Metal bridge is solved. Path B means the next viewer work must define the native AppKit/Metal bridge boundary first.
