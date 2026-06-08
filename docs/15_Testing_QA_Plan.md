# 15 — SilicaRAW Testing & QA Plan

Status: GO WITH CONDITIONS

## Principle

Protect originals. Protect edits. Protect color. Protect trust.

## Test Layers

- Static checks
- Unit tests
- Integration tests
- Golden image tests
- Performance tests
- UI interaction tests
- Manual photographer QA
- Release candidate validation

## S0 Release Blockers

- Original file modified
- Edit data lost
- Catalog corrupted
- Export silently wrong color
- MCP/plugin unauthorized mutation
- App cannot launch

## Required Tests Early

- Original hash protection
- SQLite migration
- Edit graph serialization
- Sidecar read/write
- Cache clear safety
- sRGB/P3 ICC export
- Metal slider responsiveness
- MCP permission bypass

## RAW Test Suite

Needs legally usable samples:

- DNG
- Apple ProRAW
- CR2/CR3
- NEF
- ARW
- RAF
- RW2
- ORF

Scenes: skin, food, landscape, sky, night/high ISO, HDR, mixed lighting, color checker.

## Release Gates

Alpha: app launches, import/library/basic edit/export, original safe.

Beta: no original mutation or edit-loss bug, color-managed sRGB export, catalog backup, cache clear safe, README ready.

v1.0: P0/P1 stable, data safety pass, color QA pass, export QA pass, migration path tested.

## Final Verdict

GO WITH CONDITIONS.

Need test fixture manifest, golden tolerance policy, RAW sample licensing, benchmarks, RC checklist.

---

# v1.1 Patch — Benchmark Fixture Specification

Performance targets must specify machine, file type, image size, and operation.

## Baseline Test Machine

```txt
Machine: M1 MacBook Air
Memory: 16GB preferred, 8GB secondary stress target
Storage: Internal SSD
Display: Built-in Retina display
Power: Not low-power mode
Thermal: Normal room temperature
```

## Reference Test Machine

```txt
Machine: M3/M4 MacBook Pro or better
Memory: 16GB+
Display: Built-in Display P3 panel
External display: optional P3 display
```

## RAW Fixture Classes

```txt
Class A — 24MP RAW
Purpose: common enthusiast/pro workflow

Class B — 45MP RAW
Purpose: high-resolution stress

Class C — Fujifilm RAF
Purpose: high-risk decoder/color case

Class D — Apple ProRAW DNG
Purpose: Apple-native workflow

Class E — High ISO RAW
Purpose: detail/noise/render stress

Class F — Tagged raster images
sRGB JPEG, Display P3 HEIC/JPEG, untagged JPEG
Purpose: color management verification
Status after Spike 003: required but not committed
```

## Preview Benchmark

```txt
Fit-to-screen preview:
Target long edge: 2560px

Interactive slider:
Exposure drag from 0.00 to +1.00 over 1 second

Slider release:
Full preview refresh after release
```

## Reporting Format

Every performance report must include:

```txt
Git commit
App version
macOS version
Machine model
Chip
Memory
RAW fixture class
Image dimensions
Operation
Median
p95
Notes
```
