# Photographer Workflow QA

Status: active
Updated: 2026-06-17
Source of truth: docs/wiki/tasks/22.5-manual-photographer-qa-checklist.md

This checklist is the Task 22.5 manual workflow record for a local macOS alpha after Phases 17 through 21. It is intended for licensed or user-provided local assets only. No private photos are committed to the repository.

## Licensed or User-Provided Assets

- [x] Use generated or local-only assets for automated harness checks; repository fixtures do not include private user photos.
- [x] Legal RAW/color fixture paths remain gated by `SILICARAW_RAW_FIXTURE_MANIFEST`.
- [ ] Record tester name or handle.
- [ ] Record macOS version, Mac model, chip, memory, and display/profile notes.
- [ ] Record asset source, license or user permission, fixture class, and privacy notes.
- [ ] Record pre-test SHA-256 for every original file used in the manual pass.

## Culling

- [ ] Import a folder by reference and confirm originals stay in place.
- [ ] Rate at least three photos with different ratings.
- [ ] Mark one photo picked and one rejected.
- [ ] Reopen the library and confirm ratings, pick, reject, and color labels persist.

## Metadata

- [ ] Confirm known JPEG/JPG dimensions display when available.
- [ ] Confirm RAW or unsupported metadata shows unavailable/blocked states honestly.
- [ ] Confirm missing metadata does not trigger original-file mutation.

## Undo

- [ ] Commit a Develop edit and confirm one history checkpoint appears.
- [ ] Use undo and redo for culling or Develop state and confirm catalog state changes as expected.
- [ ] Confirm undo does not delete export files, sidecars, backups, or cache bytes.

## Develop

- [ ] Apply exposure and contrast adjustments and confirm preview/readback match committed state.
- [ ] Exercise white balance, tone recovery, color presence, tone curve, HSL, and geometry controls on supported JPEG/JPG assets.
- [ ] Confirm unsupported Detail or RAW-only paths show blocked states instead of silent no-ops.

## Masks

- [ ] Create or inspect manual linear/radial/brush mask state on a supported JPEG/JPG asset.
- [ ] Confirm AI/MLX mask paths remain unavailable in the local alpha.
- [ ] Confirm RAW-derived masked export blocks before output when unsupported.

## Export

- [ ] Export JPEG sRGB and record output path, output SHA-256, and visible result.
- [ ] Export PNG and TIFF when the workflow requires them, recording output paths.
- [ ] Test metadata policy choices: minimal, preserve, remove GPS, and remove all.
- [ ] Confirm original files unchanged by comparing pre/post SHA-256.

## Responsiveness

- [ ] Record grid scroll responsiveness with the target library size.
- [ ] Record Loupe photo switching responsiveness.
- [ ] Record Develop slider drag responsiveness.
- [ ] Record export progress responsiveness for selected batch size.

## Data Safety

- [x] Automated local alpha workflow checks cover original hash preservation for generated fixtures.
- [x] Backup/restore checks cover staged restore, rollback, corrupt backup failure, and cache regeneration boundaries.
- [ ] Confirm original files unchanged after import, culling, Develop edits, masks, export, backup, restore, and cache clear.
- [ ] Confirm cache clear removes disposable cache state only.
- [ ] Confirm sidecar writes occur only when explicitly requested.

## Color and Export

- [x] Color/export harness checks cover JPEG sRGB and explicit Display P3 policy boundaries.
- [x] RAW/Metal profile report records fixture-gated RAW timing boundaries without broad RAW claims.
- [ ] Visually inspect JPEG sRGB output in Preview.app or Photos.
- [ ] Record ICC/profile observations and any visible color shifts.
- [ ] Confirm release/public language does not exceed recorded evidence.

## Known Limitations

- The checklist is not a substitute for clean-Mac DMG install QA.
- Full fixture-backed RAW decode/export QA requires legal local RAW fixture assets.
- Full native Metal pixel throughput remains feature-gated and is not proven by this checklist.
- Full interactive UI latency requires installed-app profiling and should not be inferred from cargo test timing.
- MLX, MCP, plugin runtime, cloud sync, telemetry, auto-update, Homebrew, and Mac App Store distribution are outside this local alpha scope.

## Result Record

- [ ] Attach or record manual QA notes.
- [ ] Record final pass/fail decision.
- [ ] File follow-up issues for defects, evidence gaps, or unclear limitations.
