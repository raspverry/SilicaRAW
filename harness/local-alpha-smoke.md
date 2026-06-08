# Local Alpha Smoke Checklist

Use this checklist when testing a SilicaRAW DMG intended for local alpha validation.

Phase 2 packaging artifacts are developer-only. They validate app and DMG generation, but they are not user-ready local alpha releases.

## Artifact

- [ ] DMG filename:
- [ ] SHA256 checksum:
- [ ] Git tag or commit:
- [ ] macOS version:
- [ ] Machine model:
- [ ] Apple Silicon chip:

## Install

- [ ] Download DMG from GitHub Release or workflow artifact.
- [ ] Verify SHA256 checksum.
- [ ] Open DMG.
- [ ] Drag `SilicaRAW.app` to `/Applications`.
- [ ] Launch app from `/Applications`.
- [ ] Confirm the app launches offline.

## Phase 2 Packaging Smoke

Use this section for unsigned developer DMGs generated before product workflows exist.

- [ ] Build command: `cargo tauri build --bundles app,dmg --ci --no-sign`
- [ ] Output includes `SilicaRAW.app`.
- [ ] Output includes `SilicaRAW_0.1.0_aarch64.dmg`.
- [ ] `SilicaRAW.app` launch request succeeds.
- [ ] App process can be closed cleanly.
- [ ] Artifact is clearly marked unsigned, ad-hoc, and developer-only.
- [ ] Artifact is not published as a user-ready release.

## Minimal Workflow

This section applies only after the app implements the local alpha product workflows.

- [ ] Create or open a local SilicaRAW library.
- [ ] Import a folder by reference.
- [ ] Confirm original files remain in place.
- [ ] Show a library grid.
- [ ] Rate a photo.
- [ ] Pick or reject a photo.
- [ ] Open a preview.
- [ ] Apply exposure or contrast.
- [ ] Confirm edit state persists after restart.
- [ ] Export JPEG sRGB to a chosen export location.
- [ ] Confirm the original file hash did not change.

## Failure States

- [ ] Unsupported file shows a clear unsupported state.
- [ ] Missing file shows a clear missing state.
- [ ] Export failure shows a clear error and does not modify originals.

## Release Gate

The DMG is not user-ready if any of these fail:

- App launch.
- Library create/open.
- Import by reference.
- Rating/pick/reject persistence.
- Preview.
- Exposure/contrast persistence.
- JPEG sRGB export.
- Original-file hash safety.
