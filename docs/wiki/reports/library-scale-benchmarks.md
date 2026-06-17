---
title: Library Scale Benchmarks
status: active
audience: agents
updated: 2026-06-17
source_of_truth: scripts/harness/run-library-scale-benchmarks.py
---

# Library Scale Benchmarks

## Scope

Local benchmark evidence for this machine only; results are not universal performance guarantees.

The benchmark seeds synthetic catalog rows in a temporary local library, then measures the existing typed `silica-storage::query_library_photos` path. It does not create original photo files, decode RAW files, generate thumbnails, or measure the native viewer.

## Machine

| Field | Value |
|---|---|
| OS | `macos` |
| Arch | `aarch64` |
| CPU Count | `18` |
| Rust | `rustc 1.95.0 (59807616e 2026-04-14)` |

## Dataset Shape

| Photos | JPEG | RAW | Unsupported | With Dimensions | Picked | Rejected |
|---:|---:|---:|---:|---:|---:|---:|
| 1000 | 800 | 150 | 50 | 400 | 143 | 59 |
| 10000 | 8000 | 1500 | 500 | 4000 | 1429 | 589 |
| 50000 | 40000 | 7500 | 2500 | 20000 | 7143 | 2942 |

## Timings

All values are median milliseconds over the recorded query runs. The render-adjacent row is a lightweight page-model shaping pass over the queried page, not GPU rendering.

| Photos | Imported Page | JPEG Filter | Metadata Filter | Search | Render-Adjacent Page Model | Seed Catalog |
|---:|---:|---:|---:|---:|---:|---:|
| 1000 | 1.651 | 2.094 | 1.742 | 1.708 | 1.424 | 34.493 |
| 10000 | 7.071 | 13.424 | 8.532 | 6.184 | 6.878 | 193.078 |
| 50000 | 38.291 | 71.514 | 46.148 | 26.78 | 38.266 | 1000.282 |

## Reproduce

```bash
python3 scripts/harness/run-library-scale-benchmarks.py
python3 scripts/harness/check-library-scale-benchmark.py
```
