---
title: Local Alpha Installed App Launch
status: active
audience: maintainers
updated: 2026-07-08
source_of_truth: docs/wiki/roadmaps/local-alpha-quality-closure-plan.md
---

# Local Alpha Installed App Launch

## Scope

This report records Q6.2 install/launch evidence for the unsigned developer-preview DMG built in Q6.1. It proves the app copied to `/Applications` matches the mounted DMG app and launches from `/Applications/SilicaRAW.app`.

It does not prove the full import, edit, export, restart, offline, Gatekeeper, signed/notarized, GitHub Release download, or clean-Mac workflow.

## Test Record

| Field | Value |
| --- | --- |
| Date | 2026-07-08 |
| Evidence source commit | `cc2f3418da6513c1cdd6222fb0bdf08ba5ff67b6` |
| DMG build source commit | `a2f66bfec44306bff290172ef4be10954d16463a` |
| DMG | `target/release/bundle/dmg/SilicaRAW_0.1.0_aarch64.dmg` |
| Installed app | `/Applications/SilicaRAW.app` |
| Installed app tree SHA256 | `89ab925b523bc2b7943fcfa8ad3318e3266b542edc5d8f7967addf8ff60da95d` |
| Executable SHA256 | `af39bbec2a441763509526168380fb461502f3e4703950e619d8f214dfe2cfba` |
| Mounted/install comparison report | `.tmp/q6-installed-app-launch/local-dmg-artifact-smoke.json` |
| Mounted/install comparison report SHA256 | `64e748bde1a518f69b347c694fc6327d8301bba085bddf9833e4f9067ec7b3be` |
| Installed launch report | `.tmp/q6-installed-app-launch/installed-app-launch-smoke.json` |
| Installed launch report SHA256 | `d783f8c1fe95e8c13ef8646c7f1184e1942a7639428c5f6c3001c159a99d1057` |
| Result | `partial pass`: install and launch proof recorded; full installed workflow not run |

## Commands

Install command used after confirming the existing `/Applications/SilicaRAW.app` differed from the Q6.1 DMG:

```bash
hdiutil attach target/release/bundle/dmg/SilicaRAW_0.1.0_aarch64.dmg -nobrowse -readonly
rm -rf /Applications/SilicaRAW.app
ditto /Volumes/SilicaRAW/SilicaRAW.app /Applications/SilicaRAW.app
hdiutil detach /Volumes/SilicaRAW
```

Verification commands:

```bash
python3 scripts/harness/local-dmg-artifact-smoke.py \
  --dmg target/release/bundle/dmg/SilicaRAW_0.1.0_aarch64.dmg \
  --installed-app /Applications/SilicaRAW.app \
  --output .tmp/q6-installed-app-launch/local-dmg-artifact-smoke.json

python3 scripts/harness/installed-app-launch-smoke.py \
  --app /Applications/SilicaRAW.app \
  --output .tmp/q6-installed-app-launch/installed-app-launch-smoke.json
```

## Evidence

| Check | Value |
| --- | --- |
| Installed app matches mounted DMG app | true |
| Installed app tree SHA256 | `89ab925b523bc2b7943fcfa8ad3318e3266b542edc5d8f7967addf8ff60da95d` |
| Launched process path | `/Applications/SilicaRAW.app/Contents/MacOS/silica-desktop` |
| Process path matches installed app | true |
| Launched from `/Applications` | true |
| Launched from repo checkout | false |
| Process stopped after smoke | true |
| Code signature | ad-hoc |
| TeamIdentifier | not set |
| Clean-Mac gate remains | true |
| Full workflow from installed app | not run |

## Evidence Boundary

This pass proves:

- the installed `/Applications/SilicaRAW.app` matches the app inside the Q6.1 DMG
- macOS launches the installed app from `/Applications`
- the running process is not the repository checkout binary
- the installed app keeps the expected unsigned/ad-hoc signing state

This pass does not prove native UI import, edit, export, restart persistence, offline behavior, Gatekeeper acceptance, notarization, GitHub Release download behavior, or clean-Mac behavior.

## Next Gate

Q6.2 installed executable workflow evidence is recorded in [Local Alpha Installed App Workflow](local-alpha-installed-app-workflow.md). Q6.3 should verify offline behavior for the installed app workflow.
