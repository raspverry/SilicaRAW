---
title: "ADR 0003: App Shell Packaging Path"
status: accepted
audience: all
updated: 2026-06-08
source_of_truth: docs/20_v1_1_Architecture_Patch.md
---

# ADR 0003: App Shell Packaging Path

## Context

The architecture currently proposes a Tauri app shell with Rust Core and a native Metal viewer. The highest-risk early question is whether Tauri can host or coordinate the required native Metal rendering path without compromising the editor.

Metal-first editing has priority over the shell preference.

## Decision

Use Tauri v2 for the first app shell and packaging spike.

This is not a final commitment to Tauri for the whole product. It is a controlled spike path to verify:

- Tauri app launch and bundle generation.
- Minimal macOS `.app` and developer DMG packaging.
- Native Metal viewer feasibility through Spike 001.

If Spike 001 records Path C, Tauri-dependent product UI work stops and planning switches to SwiftUI/AppKit shell plus Rust Core.

## Consequences

- Tauri dependencies may be added only with `docs/DEPENDENCIES.md` updates.
- The first Tauri shell should be minimal and must not implement RAW decoding, Metal viewer, MLX, MCP, plugin behavior, or broad UI screens.
- Packaging skeleton work may proceed before the Metal spike, but Develop/editor UI work must wait for the spike result.

## Alternatives Considered

- SwiftUI/AppKit first: likely best for native Metal certainty, but it bypasses the documented Tauri-first spike path.
- Electron: not preferred; only considered after SwiftUI/AppKit plus Rust Core is proven impractical.
- Continue with placeholder binary: insufficient because local DMG distribution requires a real app bundle path.

## Links

- [Architecture Patch](../../20_v1_1_Architecture_Patch.md)
- [Metal Rendering Topic](../topics/metal-rendering.md)
- [Architecture Risks](../risks/architecture-risks.md)
- [Local DMG Distribution Plan](../roadmaps/local-dmg-distribution-plan.md)

## Notes for LLM Agents

Do not build broad Tauri UI before the Metal viewer spike. Keep the first shell minimal and reversible.

