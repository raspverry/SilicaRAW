---
title: Local Alpha Built App Launch
status: active
audience: maintainers
updated: 2026-07-08
source_of_truth: docs/wiki/roadmaps/local-alpha-quality-closure-plan.md
---

# Local Alpha Built App Launch

## Scope

This report records Q5.1 evidence that a built `SilicaRAW.app` bundle launches as a native app bundle, not as static HTML in a browser.

It does not prove the `/Applications` installed workflow, DMG install behavior, Gatekeeper acceptance, clean-Mac behavior, offline workflow, or the full local alpha workflow. Those remain Q5.2 through Q7 evidence gates.

## Test Record

| Field | Value |
| --- | --- |
| Date | 2026-07-08 |
| Source commit at build time | `8ddb3b22cbda8f696935f3996c02972cefb303cc` |
| App path | `target/release/bundle/macos/SilicaRAW.app` |
| Executable path | `target/release/bundle/macos/SilicaRAW.app/Contents/MacOS/silica-desktop` |
| App tree SHA256 | `89ab925b523bc2b7943fcfa8ad3318e3266b542edc5d8f7967addf8ff60da95d` |
| Executable SHA256 | `af39bbec2a441763509526168380fb461502f3e4703950e619d8f214dfe2cfba` |
| File count | `3` |
| Size bytes | `15058030` |
| Host | `macOS 26.4`, `arm64` |
| Signing state | ad-hoc/linker-signed, no TeamIdentifier |
| Launch result | `pass` |
| Observed process path | `/Users/hansol/dev/personal/SilicaRAW/target/release/bundle/macos/SilicaRAW.app/Contents/MacOS/silica-desktop` |

## Commands

Build unsigned app bundle:

```bash
cd apps/desktop/src-tauri
cargo tauri build --bundles app --ci --no-sign
```

Preflight fixtures and artifact hash report:

```bash
python3 scripts/harness/generate-legal-fixtures.py \
  --output .tmp/q5-built-app-launch/legal-qa-fixtures \
  --include-raw-placeholders

python3 scripts/harness/installed-app-preflight.py \
  --app target/release/bundle/macos/SilicaRAW.app \
  --fixtures .tmp/q5-built-app-launch/legal-qa-fixtures \
  --output .tmp/q5-built-app-launch/installed-app-preflight.json
```

Launch and process-path check:

```bash
open -n target/release/bundle/macos/SilicaRAW.app
ps -axo pid,command | rg "SilicaRAW.app|silica-desktop"
kill 83584
ps -axo pid,command | rg -v rg | rg "SilicaRAW.app/Contents/MacOS/silica-desktop"
```

The final process check returned no app process after cleanup.

## Preflight Evidence

| Field | Value |
| --- | --- |
| Preflight report | `.tmp/q5-built-app-launch/installed-app-preflight.json` |
| Preflight report SHA256 | `acc8d516b415087415556b9f09edb80ce2f733d7819e8ad1fc0bb3157984de69` |
| Fixture manifest | `.tmp/q5-built-app-launch/legal-qa-fixtures/fixture-manifest.json` |
| Fixture manifest SHA256 | `3e71efb672533ce2c28cc372b397797dbe3ba3eb030bd39046cf6d6451e2465c` |
| Fixture count | `8` |
| Fixture hash result | all pass |
| Supported fixture roles | JPEG, PNG, TIFF |
| Blocked fixture roles | RAW placeholders, WebP, text |

## Codesign Evidence

```txt
Executable=/Users/hansol/dev/personal/SilicaRAW/target/release/bundle/macos/SilicaRAW.app/Contents/MacOS/silica-desktop
Format=app bundle with Mach-O thin (arm64)
CodeDirectory flags=0x20002(adhoc,linker-signed)
Signature=adhoc
TeamIdentifier=not set
```

## Evidence Boundary

This pass proves:

- `cargo tauri build --bundles app --ci --no-sign` produces a local `.app` bundle.
- The generated `.app` has a stable artifact hash and legal fixture preflight report.
- macOS can launch the generated `.app` as a GUI app bundle.
- The running process path is the built `.app` executable, not a browser/static HTML path.

This pass does not prove:

- install by dragging from DMG to `/Applications`
- launch from `/Applications`
- full library/import/edit/export workflow
- offline workflow
- Gatekeeper acceptance
- signed or notarized behavior
- clean-Mac downloaded-artifact behavior

## Next Gate

Q5.2 should use an installed app workflow pass to create or open a library, import by reference, and verify original source hashes before and after import.
