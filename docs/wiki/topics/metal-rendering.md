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
- Tauri is currently planned as the app shell, but the Tauri + native Metal viewer bridge is a gate.
- If Tauri blocks the Metal-first editor experience, the shell decision must be revisited.

## Blocked Work

- Native Metal viewer implementation.
- Shader passes.
- Texture manager implementation.
- Viewer event mapping.
- Full render loop coordination.

## Spike 001 Must Verify

- Native Metal-rendered view can be hosted or coordinated.
- Resize and Retina scaling work.
- Mouse and trackpad events map correctly.
- UI remains responsive.
- The render loop can be controlled from the Rust/Core side.

## Links

- [Metal Render Pipeline Specification](../../08_Metal_Render_Pipeline_Specification.md)
- [Architecture Patch](../../20_v1_1_Architecture_Patch.md)
- [System Architecture](../../03_System_Architecture.md)
- [Architecture Risks](../risks/architecture-risks.md)

## Notes for LLM Agents

Do not add a fake viewer or broad UI shell that assumes the Metal bridge is solved. Record the spike result before building viewer-dependent features.

