# silica-storage

Storage and persistence boundary for SilicaRAW.

Spike 004 selected `rusqlite` with bundled SQLite and embedded SQL migrations.

This crate currently owns the catalog migration runner, initial empty catalog schema/index proof, local library create/open, Phase 4.3 folder import scanner, Phase 4.4 photo flags persistence, Phase 5.3 active edit graph commit/read behavior, Task 10.3 library-local sidecar read/write behavior, Task 10.4 catalog rebuild dry-run reports from sidecars, Task 10.5.2 checkpointed backup boundary creation, and Task 10.5.3 staged restore boundaries. It does not decode photos, extract camera metadata, mutate originals, write sidecars next to originals, manage automatic sidecar sync, or expose database access to plugins/MCP.

Phase 4.1 aligns migration verification with the domain-facing schema contract in `silica-catalog`. `silica-storage` applies migrations and checks that the required alpha tables and indexes exist; `silica-catalog` defines the contract names.

Phase 4.2 adds local library folder create/open helpers. These helpers create the library support directories, initialize or reopen `catalog.db`, upsert the local library row, and preserve original photo directories outside the chosen library root.

Phase 4.3 adds a non-recursive folder import scanner. It records immediate child files by reference, stores file size, modified time, and partial hash, and marks unsupported extensions without crashing or copying originals.

Phase 4.4 stores rating, picked, rejected, and color label values in SQLite `photo_flags`. Imported photos receive default flag rows, updates do not write sidecars yet, and restart tests verify the catalog values survive reopen.

Phase 5.1 adds a typed photo preview candidate lookup. It reads only catalog fields needed for preview routing: photo id, file name, original path, and unsupported state.

Phase 5.3 adds active edit graph persistence in `edit_states`. Draft exposure/contrast preview updates load or build an edit graph without writing; only commit/release calls persist the final graph.

Task 10.3 adds explicit sidecar write/read APIs. Sidecars are written only under `sidecars/` inside the library root, validate the sidecar and nested edit graph payloads, mirror rating/picked/rejected/color-label state only, update `sidecar_status` after successful writes, and do not mutate original referenced files.

Task 10.4 adds `dry_run_catalog_rebuild_from_sidecars`. It scans library-local sidecars in deterministic order, reports resolved portable flag state and non-fatal issues, uses `sidecar.flags` before `edit_graph.metadata` before defaults, and does not mutate catalog tables or original referenced files.

Task 10.5.2 adds `create_library_backup`. It checkpoints SQLite WAL state, then writes a library-local backup artifact containing `catalog.db`, `sidecars/`, and `backup-manifest.json` under `backups/`. It excludes originals, disposable caches, export output files, logs, and nested backup artifacts.

Task 10.5.3 adds `restore_library_backup`. It validates backup manifests, restores through a staging directory, creates rollback copies before replacing existing target catalog/sidecar state, rejects newer catalog schema backups before target mutation, and preserves original referenced files.
