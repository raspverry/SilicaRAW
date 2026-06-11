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

## Task 10.5 Backup and Restore QA Record

Status: Task 10.5.1 recovery policy added on 2026-06-11. Task 10.5.2 backup boundary implementation added on 2026-06-11. Restore execution remains pending.

Automated policy validation command:

```bash
python3 scripts/harness/check-recovery-policy.py
cargo test -p silica-storage backup
```

Policy coverage:

- [x] Backup uses checkpoint-before-copy policy.
- [x] Backup must exclude original referenced photo files.
- [x] Backup must exclude disposable cache directories.
- [x] Restore must target an empty directory or use a rollback copy first.
- [x] Restore must not write into original referenced photo folders.
- [x] Newer catalog schema backups are rejected by older app builds.
- [x] Migration failure behavior is explicit.

Backup implementation coverage:

- [x] Backup excludes `thumbnails/`, `previews/`, `render-cache/`, and `ai-cache`.
- [x] Backup includes checkpointed `catalog.db` and `sidecars/`.
- [x] Backup writes `backup-manifest.json` with schema/version, app version, catalog schema version, checkpoint mode, and relative file list.
- [x] Backup does not include original referenced photo files.
- [x] Backup does not follow export output paths.
- [x] Backup does not copy `exports/`, `logs/`, or existing `backups/` artifacts.
- [x] Backup does not copy temporary sidecar write files.
- [x] Backup copies latest WAL state through checkpoint before copying `catalog.db`.

Restore implementation QA still needed:

- [ ] Restore preserves edit states, flags, sidecar status, export records, and migration metadata.
- [ ] Restore does not write into original photo folders.
- [ ] Migration failure leaves the target recoverable.

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
