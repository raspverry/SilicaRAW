# 03 — SilicaRAW System Architecture v2

Status: GO WITH CONDITIONS

## Architecture Summary

SilicaRAW uses a modular architecture:

```txt
Tauri UI Shell
↓
Rust Core
├─ Catalog
├─ Edit Graph
├─ Storage
├─ Render Requests
├─ Export Coordinator
├─ Permission Layer
│
├─ RAW Decode Layer
│  ├─ Core Image RAW Backend
│  └─ LibRaw Backend
│
├─ Metal Render Engine
│  ├─ Preview Renderer
│  ├─ Adjustment Passes
│  ├─ Histogram
│  ├─ Mask Compositor
│  └─ Export Render Path
│
├─ MLX Engine
├─ Plugin Layer
└─ MCP Layer
```

## Key Decisions

### Metal-first

Interactive preview and adjustment rendering must use Metal as a core layer, not as optional acceleration.

### MLX as intelligent editing layer

MLX handles masks, denoise, upscale, auto tone, blur scoring, quality scoring, duplicate grouping. It does not own the final image and does not mutate the edit graph without approval.

### Rust core owns state

Rust core owns catalog state, edit graph, permissions, storage commands, render requests, and export orchestration.

### Tauri is UI shell

Tauri UI owns controls, panels, gestures, and viewer container. It must not implement image processing logic.

### RAW decode abstraction

RAW decoding is abstracted so Core Image and LibRaw can be compared and swapped.

## Dependency Rules

Allowed:

```txt
UI → Rust Core → Render/Storage/Export/MLX
MCP → Permission Layer → Core Commands
Plugin → Permission Layer → Core Commands
MLX → AI Results → User Approval → Edit Graph
```

Forbidden:

```txt
UI directly processes pixels
MLX directly mutates edit graph
Plugin directly writes SQLite
MCP directly writes SQLite
Plugin/MCP modifies original files
Render engine owns catalog state
Storage owns rendering logic
```

## Core Crates

- `silica-core`
- `silica-catalog`
- `silica-storage`
- `silica-decode`
- `silica-render`
- `silica-edit`
- `silica-export`
- `silica-mlx`
- `silica-plugin`
- `silica-mcp`

## Required Spikes

1. Tauri + native Metal viewer
2. RAW decode comparison
3. Color-managed preview/export
4. SQLite persistence
5. MLX runtime

## Final Verdict

GO WITH CONDITIONS.

Do not skip feasibility spikes.
