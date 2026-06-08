# 13 — SilicaRAW Development Roadmap

Status: GO WITH CONDITIONS

## Principle

Kill technical risk before adding product breadth.

## Phases

0. Feasibility Spikes
1. Repository & App Foundation
2. Design System & Component Gallery
3. Catalog / Import / Library Grid
4. Metal Viewer & Preview Pipeline
5. Basic Develop Editing
6. Export MVP
7. Professional Editing Baseline
8. Sidecar / Backup / Stability
9. MLX Differentiators
10. Plugin Foundation
11. MCP Read-only / Controlled Automation
12. Public Beta
13. v1.0 Stable

## Mandatory Spikes

- Tauri + Metal viewer
- RAW decode comparison
- Color-managed preview/export
- SQLite catalog persistence
- MLX runtime

## GO Gates

Gate A: Architecture viability.
Gate B: Editor viability.
Gate C: Trust viability.
Gate D: OSS viability.

## Release Progression

- Internal alpha: developers only
- Private alpha: trusted testers
- Public beta: no data-loss bugs, strong README, signed/notarized preferred
- v1.0: credible RAW editor baseline

## Highest Risks

- Tauri + Metal bridge
- Color management
- RAW decode quality
- Slider responsiveness
- Storage/migration safety
- UI feeling non-Apple/open-source utility
- Overmarketing AI/MCP

## Final Verdict

GO WITH CONDITIONS.

Start with spikes, not broad implementation.

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
