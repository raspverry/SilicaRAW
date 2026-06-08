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

## Phase 4.2 Library Create/Open Smoke

Use this section after local library create/open commands are available.

- [ ] Enter a new local library folder path.
- [ ] Create library succeeds.
- [ ] `catalog.db` exists inside the selected library folder.
- [ ] `sidecars/`, `thumbnails/`, `previews/`, `render-cache/`, `ai-cache/`, `exports/`, `logs/`, and `backups/` exist.
- [ ] Quit and relaunch the app.
- [ ] Open the same library path.
- [ ] Reopen reports the same library root and catalog path.
- [ ] A sibling original-photo test folder remains unchanged.

## Phase 4.3 Folder Import Scanner Smoke

Use this section after folder import scanner APIs are available.

- [ ] Prepare a folder with at least one supported photo extension such as `.dng` or `.jpg`.
- [ ] Include at least one unsupported file such as `.txt`.
- [ ] Import scanner records both files in the catalog by original path.
- [ ] Supported file has `unsupported = 0`.
- [ ] Unsupported file has `unsupported = 1`.
- [ ] File size, modified time, and partial hash are stored.
- [ ] Original files remain in the source folder unchanged.
- [ ] Original files are not copied into the library folder during import.

## Phase 4.4 Photo Flags Persistence Smoke

Use this section after rating, pick, reject, and color label command APIs are available.

- [ ] Import at least one supported photo candidate.
- [ ] Set rating to a non-zero value.
- [ ] Set picked or rejected state.
- [ ] Optionally set a color label.
- [ ] Quit and relaunch the app.
- [ ] Open the same library path.
- [ ] Confirm the same `photo_flags` values are returned from the catalog.
- [ ] Confirm no sidecar files are required for this persistence check yet.

## Phase 5.1 Preview Readiness Smoke

Use this section after preview status command APIs are available.

- [ ] Import a folder with a `.jpg` candidate.
- [ ] Open preview status for the `.jpg` catalog photo.
- [ ] Confirm preview status is ready by reference.
- [ ] Import or select an unsupported file.
- [ ] Confirm preview status returns a clear unsupported state.
- [ ] Import or select a RAW extension such as `.dng`.
- [ ] Confirm preview status clearly says Core Image RAW preview is blocked until fixture-backed probe coverage exists.
- [ ] Do not treat this as a Metal viewer, RAW decode, or color correctness smoke test.

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
