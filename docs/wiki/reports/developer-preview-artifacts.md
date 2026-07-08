---
title: Developer Preview Artifacts
status: active
audience: maintainers
updated: 2026-07-08
source_of_truth: docs/wiki/roadmaps/developer-preview-artifact-runbook.md
---

# Developer Preview Artifacts

## Scope

This report records unsigned developer-preview DMG evidence only. These artifacts are internal testing artifacts, not user-ready local distribution, public beta, or v1.0 release assets.

User-ready local distribution remains blocked until Developer ID signing, notarization, stapling, checksums, GitHub Release publication, and clean-Mac downloaded-artifact QA are complete.

## Current Main Local Inspection

The current-main local build-machine unsigned DMG inspection is recorded separately in [Local Alpha Unsigned DMG Inspection](local-alpha-unsigned-dmg-inspection.md).

| Field | Value |
| --- | --- |
| Commit | `a2f66bfec44306bff290172ef4be10954d16463a` |
| DMG | `target/release/bundle/dmg/SilicaRAW_0.1.0_aarch64.dmg` |
| DMG SHA256 | `c4c053834bbc9d148d6108b40bd443e07114ebae0a978c67213f0f6b7f242ff3` |
| Local smoke | `.tmp/q6-unsigned-dmg-inspection/local-dmg-artifact-smoke.json` |
| Clean-Mac gate | `not run` |

## Latest Record

| Field | Value |
| --- | --- |
| Tag | `developer-preview-20260630.1` |
| Commit | `d8305260c24b5f6625334176339bd5bd3d922f95` |
| Workflow | `Developer Preview macOS DMG` |
| Run | https://github.com/raspverry/SilicaRAW/actions/runs/28434695717 |
| Run result | `success` |
| Artifact | `silicaraw-unsigned-developer-preview-macos` |
| Artifact API digest | `sha256:01fe90e99944364c75c807d261e0d76b59db51d14d35fb55645ef007e01e5641` |
| DMG | `SilicaRAW_0.1.0_aarch64.dmg` |
| DMG SHA256 | `665f1998cc7d7d148eecb458cafa0af508d39e33d9fe1f4170221de3f0de4aac` |
| Artifact expires | `2026-07-14T09:46:26Z` |
| Local smoke host | `macOS 26.4`, `arm64`, `Apple M5 Max` |
| Local artifact smoke | `pass` |
| Clean-Mac gate | `not run` |

## Verification Performed

- GitHub Actions run completed successfully for the stable preview tag.
- Downloaded artifact contained:
  - `SilicaRAW_0.1.0_aarch64.dmg`
  - `SHA256SUMS.txt`
  - `UNSIGNED-DEVELOPER-PREVIEW.txt`
- `SHA256SUMS.txt` matched the downloaded DMG SHA256.
- `UNSIGNED-DEVELOPER-PREVIEW.txt` stated that the artifact is not signed or notarized and must not be described as user-ready local distribution.
- `scripts/harness/local-dmg-artifact-smoke.py` verified the DMG, mounted it, and found `SilicaRAW.app` in the mounted image.

## Release Notes Draft

### Included

- Local macOS developer-preview DMG containing `SilicaRAW.app`.
- Supported local raster source workflow for JPEG/JPG, PNG, TIF, and TIFF.
- Library grid, Loupe preview, Develop exposure/contrast commit, rating/pick/reject state, and JPEG sRGB export for supported raster sources.
- Legacy catalog migration that reclassifies old PNG/TIF/TIFF rows as supported raster rows without mutating originals.

### Known Limitations

- The artifact is unsigned and not notarized; Gatekeeper warnings are expected.
- This is not a public beta or user-ready release.
- Clean-Mac downloaded-artifact QA was not run for this record.
- RAW, HEIC, WebP, database files, and sidecar-like rows remain blocked or unsupported in the local alpha source path unless a later task adds fixture-backed end-to-end support.
- MLX, plugin runtime, MCP server/runtime, cloud sync, telemetry, auto-update, Homebrew, and Mac App Store distribution remain out of scope.

## Reproduce

```bash
gh run download 28434695717 --dir .tmp/developer-preview-28434695717
cat .tmp/developer-preview-28434695717/silicaraw-unsigned-developer-preview-macos/SHA256SUMS.txt
shasum -a 256 .tmp/developer-preview-28434695717/silicaraw-unsigned-developer-preview-macos/SilicaRAW_0.1.0_aarch64.dmg
python3 scripts/harness/local-dmg-artifact-smoke.py \
  --dmg .tmp/developer-preview-28434695717/silicaraw-unsigned-developer-preview-macos/SilicaRAW_0.1.0_aarch64.dmg \
  --output .tmp/developer-preview-28434695717/local-dmg-artifact-smoke.json
```
