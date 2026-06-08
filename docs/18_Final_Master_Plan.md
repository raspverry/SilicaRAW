# 18 — SilicaRAW Final Master Plan / Consolidated Execution Plan

Status: FINAL PLANNING GO WITH CONDITIONS

## Executive Summary

SilicaRAW is worth pursuing as an open-source project if it stays focused:

```txt
A real RAW photo editor first.
Metal-first macOS performance.
MLX-powered intelligent editing as enhancement.
Plugin/MCP as optional extension.
Local-first and non-destructive by default.
```

## Product Definition

SilicaRAW is an open-source RAW photo editor built for Apple Silicon.

## Final Architecture

```txt
Tauri App Shell
↓
Rust Core
├─ Catalog
├─ Edit Graph
├─ Storage
├─ Render Requests
├─ Export Coordination
├─ Permission Layer
├─ RAW Decode Layer
├─ Metal Render Engine
├─ Color Pipeline
├─ MLX Engine
├─ Plugin Layer
└─ MCP Layer
```

## Final Execution Order

1. Feasibility spikes
2. Project foundation
3. Design foundation
4. Library foundation
5. Metal preview foundation
6. Basic Develop
7. Export MVP
8. Professional baseline
9. Trust layer
10. MLX differentiators
11. Plugin/MCP
12. Beta/release/OSS growth

## First 5 Mandatory Spikes

1. Tauri + Metal viewer spike
2. RAW decode comparison spike
3. Color-managed preview/export spike
4. SQLite catalog persistence spike
5. MLX runtime spike

## Critical GO/NO-GO Gates

Gate A: Architecture viability.  
Gate B: Editor viability.  
Gate C: Color viability.  
Gate D: Data safety viability.  
Gate E: OSS viability.

## Highest Risks

- Tauri + Metal viewer integration
- Color management correctness
- RAW decoder quality/support
- Metal preview responsiveness
- Data migration/safety
- Large catalog performance
- MLX memory pressure
- Plugin/MCP permission safety

## Final Verdict

SilicaRAW Master Plan v1.0: GO WITH CONDITIONS.

The project is valid and viable as an open-source project. It is not yet validated as a business.

Correct first goal:

```txt
Build a credible open-source RAW editor core and earn developer trust.
```

Final rule:

```txt
SilicaRAW is an open-source RAW photo editor built for Apple Silicon.
Everything else serves that.
```

---

# v1.1 Patch — Updated Gates

## Gate A — Architecture Viability now requires

```txt
Tauri + Metal viewer spike has fallback decision recorded.
RAW decoder spike has decoder path selected or explicitly deferred.
Provisional license strategy selected.
Dependency policy initialized in docs/DEPENDENCIES.md.
```

## Gate B — Editor Viability now requires

```txt
Edit graph v0.1 schema implemented.
Edit graph validation tests pass.
SQLite initial indexes exist.
No DB write per slider tick.
```

## Gate C — Color Viability now requires

```txt
Decoder-specific color assumptions documented.
Working color space decision recorded after spike.
Benchmark fixture class used for color/render tests recorded.
```

## Public Beta Gate now requires

```txt
Final license selected.
Third-party dependency license inventory complete.
Sample asset license manifest complete.
Model license manifest complete, if models are included.
Doc 06 single source verified.
```

## Critical Fallback Rule

If Tauri + Metal viewer spike fails:

```txt
Do not force Tauri.
Switch planning to SwiftUI/AppKit shell + Rust Core.
Metal-first editor identity takes priority over Tauri.
```
