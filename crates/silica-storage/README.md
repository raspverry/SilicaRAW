# silica-storage

Storage and persistence boundary for SilicaRAW.

Spike 004 selected `rusqlite` with bundled SQLite and embedded SQL migrations.

This crate currently owns the catalog migration runner and initial empty catalog schema/index proof. It does not scan folders, import photos, mutate originals, write sidecars, manage caches, or expose database access to plugins/MCP.

Phase 4.1 aligns migration verification with the domain-facing schema contract in `silica-catalog`. `silica-storage` applies migrations and checks that the required alpha tables and indexes exist; `silica-catalog` defines the contract names.

Phase 4.2 adds local library folder create/open helpers. These helpers create the library support directories, initialize or reopen `catalog.db`, upsert the local library row, and preserve original photo directories outside the chosen library root.
