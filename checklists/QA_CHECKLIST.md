# QA Checklist

## Task 6.2 Original Safety QA Record

Status: automated core hash QA added on 2026-06-09. Manual DMG-installed execution is recorded with Task 6.3 when a release candidate artifact exists.

Automated validation command:

```bash
cargo test -p silica-core local_alpha_workflow_preserves_original_file_hash
```

Automated coverage:

- [x] Creates a disposable local library.
- [x] Creates a generated local JPEG fixture outside the library folder.
- [x] Records the original fixture hash before import.
- [x] Imports by reference.
- [x] Applies rating and Pick state.
- [x] Opens preview/loupe through the core preview path.
- [x] Runs draft exposure/contrast preview.
- [x] Commits exposure/contrast edit state.
- [x] Exports JPEG sRGB to a separate output folder.
- [x] Simulates cache clearing by deleting current disposable cache directories: `thumbnails`, `previews`, `render-cache`, and `ai-cache`.
- [x] Reopens the local library.
- [x] Verifies original file hash is unchanged after import, flag updates, preview, draft edit, committed edit, export, simulated cache clear, and restart/reopen.
- [x] Verifies exported JPEG output path differs from the original source path.

Current cache note:

- [x] The local alpha does not yet expose a product cache-clear command or UI. The automated QA simulates the current cache safety surface by deleting disposable library cache directories only.

Manual QA record:

- Tester:
- Date:
- Git commit:
- Artifact type: `local build` / `developer unsigned DMG` / `signed notarized DMG`
- macOS version:
- Machine model:
- Source sample folder:
- Source file hash before workflow:
- Source file hash after workflow:
- Export output path:
- Result: `pass` / `fail`

Manual checklist:

- [ ] Import a folder by reference.
- [ ] Confirm UI states originals stay in place or are not modified.
- [ ] Rate, Pick, and Reject at least one photo.
- [ ] Open preview/loupe.
- [ ] Apply exposure/contrast and commit.
- [ ] Export JPEG sRGB to a separate output folder.
- [ ] Confirm export output path differs from the original source path.
- [ ] Clear cache if a cache-clear product control exists; otherwise record `not implemented in current alpha`.
- [ ] Quit and relaunch the app.
- [ ] Reopen the library and confirm edits/flags persist.
- [ ] Recompute the original source file hash and confirm it matches the pre-workflow hash.
- [ ] Confirm no exported file overwrote an original source path.

## Task 10.4 Sidecar Rebuild Dry-Run QA Record

Status: automated storage/core dry-run tests added on 2026-06-11. Applied restore behavior remains out of scope until Task 10.5.

Automated validation commands:

```bash
cargo test -p silica-storage rebuild_dry_run
cargo test -p silica-core sidecar_rebuild
```

Automated coverage:

- [x] Dry-run output is deterministic for repeated scans.
- [x] Dry-run does not mutate live `photo_flags`.
- [x] `sidecar.flags` wins over `edit_graph.metadata`.
- [x] `edit_graph.metadata` is used when sidecar flags are absent or invalid by the dry-run rule.
- [x] Defaults are used when no valid portable flags exist.
- [x] Malformed sidecars are reported without producing rebuild entries.
- [x] Schema-invalid sidecars are reported.
- [x] Photo-id path/payload mismatches are reported.
- [x] Flag/metadata disagreements are reported instead of silently resolved.
- [x] Catalog original-path or fingerprint reconciliation conflicts are reported.

## General QA Backlog

- [ ] Import 1,000 mixed images
- [ ] Rate/reject 100 photos
- [ ] Edit 20 RAW files
- [ ] Export JPEG sRGB
- [ ] Export Display P3
- [ ] Clear cache
- [ ] Restart app and verify edits persist
- [ ] Move source file and verify missing state
- [ ] Relink file
- [ ] Try unsupported RAW
- [ ] Try corrupt file
- [ ] Test 13-inch layout
- [ ] Test 16-inch layout
- [ ] Test external display
- [ ] Test offline mode
