---
title: Local Alpha Unsigned DMG Inspection
status: active
audience: maintainers
updated: 2026-07-08
source_of_truth: docs/wiki/roadmaps/local-alpha-quality-closure-plan.md
---

# Local Alpha Unsigned DMG Inspection

## Scope

This report records Q6.1 evidence that the current local alpha source can build an unsigned developer-preview DMG, verify it, mount it, and expose the expected `SilicaRAW.app` bundle.

It does not prove `/Applications` install, launch from `/Applications`, Gatekeeper acceptance, signed/notarized behavior, offline behavior, GitHub Release download behavior, or clean-Mac downloaded-artifact behavior.

## Test Record

| Field | Value |
| --- | --- |
| Date | 2026-07-08 |
| Source commit at evidence run | `a2f66bfec44306bff290172ef4be10954d16463a` |
| Artifact type | unsigned developer-preview DMG |
| Build host | local macOS developer machine |
| Build command | `cargo tauri build --bundles app,dmg --ci --no-sign` |
| App bundle | `target/release/bundle/macos/SilicaRAW.app` |
| DMG | `target/release/bundle/dmg/SilicaRAW_0.1.0_aarch64.dmg` |
| DMG SHA256 | `c4c053834bbc9d148d6108b40bd443e07114ebae0a978c67213f0f6b7f242ff3` |
| DMG size bytes | `5264562` |
| Smoke command | `scripts/harness/local-dmg-artifact-smoke.py` |
| Smoke report | `.tmp/q6-unsigned-dmg-inspection/local-dmg-artifact-smoke.json` |
| Smoke report SHA256 | `1b2d0475c019812f29308ad7095546ca07225c6902df8add8e8e2be6c22de510` |
| Result | `pass` |

## Commands

```bash
cd apps/desktop/src-tauri
cargo tauri build --bundles app,dmg --ci --no-sign

cd ../../..
python3 scripts/harness/local-dmg-artifact-smoke.py \
  --dmg target/release/bundle/dmg/SilicaRAW_0.1.0_aarch64.dmg \
  --output .tmp/q6-unsigned-dmg-inspection/local-dmg-artifact-smoke.json
```

## DMG Smoke Evidence

| Check | Value |
| --- | --- |
| `hdiutil verify` | true |
| Mount point | `/Volumes/SilicaRAW` |
| Mounted app exists | true |
| Mounted app tree SHA256 | `89ab925b523bc2b7943fcfa8ad3318e3266b542edc5d8f7967addf8ff60da95d` |
| Mounted app file count | `3` |
| Mounted app size bytes | `15058030` |
| Installed app comparison | not run in this Q6.1 smoke; recorded later in [Local Alpha Installed App Launch](local-alpha-installed-app-launch.md) |
| Copied to install path | false |
| Clean-Mac gate remains | true |
| Failures | none |

## Bundle Metadata

| Field | Value |
| --- | --- |
| `CFBundleDisplayName` | `SilicaRAW` |
| `CFBundleExecutable` | `silica-desktop` |
| `CFBundleIdentifier` | `dev.silicaraw.desktop` |
| `CFBundleShortVersionString` | `0.1.0` |
| `CFBundleVersion` | `0.1.0` |
| `LSApplicationCategoryType` | `public.app-category.photography` |
| `LSMinimumSystemVersion` | `13.0` |
| Executable SHA256 | `af39bbec2a441763509526168380fb461502f3e4703950e619d8f214dfe2cfba` |

## Signing State

| Field | Value |
| --- | --- |
| `codesign` format | app bundle with Mach-O thin `arm64` |
| Signature | `adhoc` |
| CodeDirectory flags | `0x20002(adhoc,linker-signed)` |
| TeamIdentifier | not set |
| Sealed Resources | none |

## Evidence Boundary

This pass proves:

- the current source builds an unsigned `.app` and `.dmg`
- the DMG has a recorded SHA-256 checksum
- `hdiutil verify` accepts the local DMG
- the DMG mounts read-only and contains `SilicaRAW.app`
- the mounted app has expected bundle metadata and ad-hoc signing state

This pass does not prove drag-to-`/Applications`, installed-app launch, local workflow from `/Applications`, Gatekeeper acceptance, notarization, GitHub Release download behavior, offline behavior, or clean-Mac behavior. The later [Local Alpha Installed App Launch](local-alpha-installed-app-launch.md) report records the install/launch sub-proof only; the full installed workflow remains open.

## Next Gate

Q6.2 has an install/launch sub-proof. It still needs the full local alpha workflow from `/Applications/SilicaRAW.app`: create/open library, import by reference, review flags, preview, exposure/contrast edit, JPEG sRGB export, restart persistence, and original-file safety.
