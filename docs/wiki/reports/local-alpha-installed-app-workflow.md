---
title: Local Alpha Installed App Workflow
status: active
audience: maintainers
updated: 2026-07-08
source_of_truth: docs/wiki/roadmaps/local-alpha-quality-closure-plan.md
---

# Local Alpha Installed App Workflow

## Scope

This report records Q6.2 evidence that `/Applications/SilicaRAW.app/Contents/MacOS/silica-desktop` can run the local alpha workflow from the installed app bundle.

It proves installed executable workflow behavior, not WebView click automation, native path picker behavior, offline behavior, Gatekeeper acceptance, signed/notarized behavior, GitHub Release download behavior, or clean-Mac behavior.

## Test Record

| Field | Value |
| --- | --- |
| Date | 2026-07-08 |
| Evidence source commit | `e0ddfa8919c86930cfc3297d02967214b0a6e5e2` |
| Artifact type | unsigned developer-preview DMG |
| Build command | `cargo tauri build --bundles app,dmg --ci --no-sign` |
| DMG | `target/release/bundle/dmg/SilicaRAW_0.1.0_aarch64.dmg` |
| DMG SHA256 | `cc75a60e35a09410dc681f66ce0b23890ca62d7a4243a5b4b0d9a9829986a266` |
| Installed app | `/Applications/SilicaRAW.app` |
| Installed app tree SHA256 | `680510cd60a972acd495acea53b15c0c50a0ce51df41ea9e8a7239e9e96b00a1` |
| Executable SHA256 | `af7c88667403b79169b803215667a522f0b10996cf1228bb70ce6ec313d96631` |
| Mounted/install comparison report | `.tmp/q6-installed-workflow/local-dmg-artifact-smoke.json` |
| Mounted/install comparison report SHA256 | `323f5f75d9799fd66fbda3dac8525bad0e7b19590e49648d8449c6d4c16e0208` |
| Installed launch report | `.tmp/q6-installed-workflow/installed-app-launch-smoke.json` |
| Installed launch report SHA256 | `7e5dfc25ef28102c7722a9248b22e031309723dc6aee15d5afc35a727a471473` |
| Installed workflow report | `.tmp/q6-installed-workflow/installed-app-workflow-evidence.json` |
| Installed workflow report SHA256 | `aa7820d270d69b797553018b419c6203a5d8e1597f319d55da57f9a06d0ea9c9` |
| Result | `pass` for installed executable workflow |

## Commands

```bash
cd apps/desktop/src-tauri
cargo tauri build --bundles app,dmg --ci --no-sign

cd ../../..
hdiutil attach target/release/bundle/dmg/SilicaRAW_0.1.0_aarch64.dmg -nobrowse -readonly
rm -rf /Applications/SilicaRAW.app
ditto /Volumes/SilicaRAW/SilicaRAW.app /Applications/SilicaRAW.app
hdiutil detach /Volumes/SilicaRAW

python3 scripts/harness/local-dmg-artifact-smoke.py \
  --dmg target/release/bundle/dmg/SilicaRAW_0.1.0_aarch64.dmg \
  --installed-app /Applications/SilicaRAW.app \
  --output .tmp/q6-installed-workflow/local-dmg-artifact-smoke.json

python3 scripts/harness/installed-app-launch-smoke.py \
  --app /Applications/SilicaRAW.app \
  --output .tmp/q6-installed-workflow/installed-app-launch-smoke.json

python3 scripts/harness/check-installed-app-workflow-evidence.py \
  --app /Applications/SilicaRAW.app \
  --scratch .tmp/q6-installed-workflow \
  --output .tmp/q6-installed-workflow/installed-app-workflow-evidence.json
```

## Evidence

| Check | Value |
| --- | --- |
| Installed app matches mounted DMG app | true |
| Launched process path | `/Applications/SilicaRAW.app/Contents/MacOS/silica-desktop` |
| Workflow executable path | `/Applications/SilicaRAW.app/Contents/MacOS/silica-desktop` |
| Workflow executable from repository checkout | false |
| Catalog schema version | `12` |
| Catalog rows | `4` |
| Catalog paths reference import folder | true |
| Catalog paths outside library folder | true |
| Catalog fingerprints match source files | true |
| Rating/Pick/Reject persisted | true |
| Exposure/contrast persisted | `0.4`, `12.0` |
| JPEG sRGB export recorded | true |
| Export opens with `sips` | `jpeg`, `720x480`, `RGB` |
| Original SHA256/size/partial hashes unchanged | true |
| Failures | none |

## Covered Workflow

- create library
- open library
- import folder by reference
- populate library grid
- verify unsupported RAW placeholder stays blocked
- set rating, Pick, Reject-derived final state, and color label
- open Loupe preview
- preview and commit exposure/contrast
- export JPEG sRGB to a separate output path
- record and resolve restart selection state
- reopen the library and verify flags/edit state persisted
- verify original files remain unchanged

## Evidence Boundary

This pass proves the installed app executable can run the local alpha command workflow from `/Applications`.

It does not prove manual GUI click paths, native file picker behavior, keyboard shortcuts, offline operation, Gatekeeper acceptance, notarization, GitHub Release download behavior, or clean-Mac behavior.

## Next Gate

Q6.3 should verify offline behavior for the installed app workflow without adding cloud, telemetry, auto-update, MLX runtime, MCP runtime, or plugin runtime.
