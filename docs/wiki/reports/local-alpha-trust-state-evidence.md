---
title: Local Alpha Trust-State Evidence
status: active
audience: maintainers
updated: 2026-07-08
source_of_truth: docs/wiki/roadmaps/local-alpha-quality-closure-plan.md
---

# Local Alpha Trust-State Evidence

## Scope

This report records Q5.5 evidence that the local desktop runtime distinguishes supported PNG sources from unsupported sources, downgrades deleted originals, and clears only disposable cache directories.

It does not prove manual WebView interaction, `/Applications` install, DMG behavior, Gatekeeper acceptance, signed/notarized behavior, offline behavior, or clean-Mac downloaded-artifact behavior.

## Test Record

| Field | Value |
| --- | --- |
| Date | 2026-07-08 |
| Source commit at evidence run | `4659805a1c5e8eef2b7ca3d8e4986875226837ba` |
| App artifact path | `target/release/bundle/macos/SilicaRAW.app` |
| App artifact SHA256 | `89ab925b523bc2b7943fcfa8ad3318e3266b542edc5d8f7967addf8ff60da95d` |
| Evidence command | `scripts/harness/check-trust-state-evidence.py` |
| Evidence report | `.tmp/q5-trust-states/trust-state-evidence.json` |
| Evidence report SHA256 | `e6c827642a25ac7bd287ea25057402e36ffaa9a0f1c8f272f42a87b49846b3dd` |
| Fixture manifest | `.tmp/q5-trust-states/fixtures/fixture-manifest.json` |
| Fixture manifest SHA256 | `6d80609118265135cabdf3e35bc7b323a149d1f98acd0d4e18bcb8bda538a5c6` |
| Catalog DB | `.tmp/q5-trust-states/run/SilicaRAW Library/catalog.db` |
| Catalog DB SHA256 | `819ba9ff56672e3b39b27586214747a5a3559fd28b4cf79e3febc152c1fd8d6c` |
| Result | `pass` |

## Command

```bash
python3 scripts/harness/check-trust-state-evidence.py \
  --app target/release/bundle/macos/SilicaRAW.app \
  --scratch .tmp/q5-trust-states \
  --output .tmp/q5-trust-states/trust-state-evidence.json
```

## Supported Source Sanity

| Source | File Type | Grid State | Preview State | Bytes |
| --- | --- | --- | --- | --- |
| `supported-png.png` | `PNG` | supported, not missing, thumbnail present | `Ready` | preview bytes present |

This is included because a prior manual GUI check exposed a PNG row as unsupported. The Q5.5 evidence keeps that regression class visible without broadening the task into full image-format QA.

## Unsupported States

| Source | File Type | Grid State | Preview State | Thumbnail |
| --- | --- | --- | --- | --- |
| `blocked-raw.DNG` | `DNG` | unsupported, not missing | `Unsupported` | absent |
| `notes.txt` | `TXT` | unsupported, not missing | `Unsupported` | absent |

Both preview messages report unsupported file type. They remain cataloged by reference for review, but they are not presented as preview-ready, develop-ready, or export-ready sources.

## Missing Original State

| Check | Value |
| --- | --- |
| Thumbnail existed before source deletion | true |
| Source intentionally deleted during smoke | true |
| Grid marks row missing after deletion | true |
| Grid thumbnail bytes after deletion | absent |
| Preview status | `BlockedByDecode` |
| Histogram status | `Missing` |
| Histogram pixel count | `0` |
| Develop preview status | `BlockedByDecode` |
| Develop preview bytes | absent |
| Commit edit error kind | `unsupportedEdit` |
| Export error kind | `exportBlocked` |
| Blocked export output exists | false |

## Cache-Clear State

| Check | Value |
| --- | --- |
| Cleared directories | `thumbnails`, `previews`, `render-cache`, `ai-cache` |
| Recreated directories | `thumbnails`, `previews`, `render-cache`, `ai-cache` |
| Removed cache records | `5` |
| Disposable sentinel files removed | true |
| Protected files under `sidecars`, `exports`, `logs`, `backups` preserved | true |
| Tracked originals unchanged | true |

## Evidence Boundary

This pass proves:

- PNG source rows stay supported and preview-ready in the desktop command runtime
- RAW-like and non-photo text rows stay unsupported without thumbnails or preview bytes
- deleted originals are downgraded before preview, histogram, Develop, edit commit, or export write paths proceed
- disposable cache clear touches only `thumbnails`, `previews`, `render-cache`, and `ai-cache`
- protected library subdirectories and original files remain unchanged

This pass does not prove manual GUI interaction, native file picker behavior, `/Applications` launch, DMG install, Gatekeeper, notarization, offline behavior, or clean-Mac behavior.

## Next Gate

Q6.1 unsigned DMG inspection is recorded in [Local Alpha Unsigned DMG Inspection](local-alpha-unsigned-dmg-inspection.md). Q6.2 should copy the app from the mounted DMG to `/Applications`, launch `/Applications/SilicaRAW.app`, and record that the app is not running from the repository checkout or mounted image.
