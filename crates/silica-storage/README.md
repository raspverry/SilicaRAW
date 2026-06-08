# silica-storage

Storage and persistence boundary for SilicaRAW.

Spike 004 selected `rusqlite` with bundled SQLite and embedded SQL migrations.

This crate currently owns the catalog migration runner, initial empty catalog schema/index proof, local library create/open, and Phase 4.3 folder import scanner. It does not decode photos, extract camera metadata, mutate originals, write sidecars, manage caches, or expose database access to plugins/MCP.

Phase 4.1 aligns migration verification with the domain-facing schema contract in `silica-catalog`. `silica-storage` applies migrations and checks that the required alpha tables and indexes exist; `silica-catalog` defines the contract names.

Phase 4.2 adds local library folder create/open helpers. These helpers create the library support directories, initialize or reopen `catalog.db`, upsert the local library row, and preserve original photo directories outside the chosen library root.

Phase 4.3 adds a non-recursive folder import scanner. It records immediate child files by reference, stores file size, modified time, and partial hash, and marks unsupported extensions without crashing or copying originals.
