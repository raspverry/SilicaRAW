# 04 — SilicaRAW UI/UX Design Specification

Status: GO WITH CONDITIONS

## UX Principle

SilicaRAW should feel like an Apple Pro App, not an open-source utility.

## Primary Layout

```txt
┌──────────────────────────────────────────────┐
│ Unified Toolbar                              │
├──────────────┬────────────────┬──────────────┤
│ Left Sidebar │ Main Viewer    │ Right Inspector│
├──────────────┴────────────────┴──────────────┤
│ Optional Filmstrip / Status                  │
└──────────────────────────────────────────────┘
```

## Region Rules

- Top: mode, commands, search, export, settings
- Left: navigation, folders, collections, presets
- Center: photo content
- Right: inspector, metadata, editing, export controls
- Bottom: filmstrip/status/progress/compare context

## Main Modes

1. Library: browse, import, cull, rate, reject, compare.
2. Develop: edit RAW, use histogram, sliders, masks, presets, before/after.
3. Export: choose format, quality, color space, metadata, destination.

## Apple-like Design Principles

- Content first
- Quiet surfaces
- Dark-first photo editing UI
- System typography
- Trackpad-friendly
- Keyboard-heavy workflows
- Consistent toolbar/sidebar/inspector model
- Progressive disclosure for advanced tools

## AI/MCP UI Rules

- No AI primary nav.
- No MCP primary toolbar item.
- MLX features appear as normal editing tools: Auto Tone, Subject Mask, Sky Mask, Denoise.
- MCP lives in Preferences → Advanced → Agent Access.

## Responsive Layout Tiers

- Compact desktop: 1280–1439px, 13-inch MacBook, collapsible sidebar, compact toolbar.
- Standard desktop: 1440–1719px, 14-inch MacBook, sidebar/inspector visible.
- Large desktop: 1720px+, 16-inch/external, expanded inspector and compare workflows.

## UX Gates

- Component gallery before screens
- High-fidelity mockups before full UI implementation
- Develop screen density test
- Trackpad gesture testing
- Keyboard-only test

## Final Verdict

GO WITH CONDITIONS.
