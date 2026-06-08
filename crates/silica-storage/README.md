# silica-storage

Storage and persistence boundary for SilicaRAW.

Spike 004 selected `rusqlite` with bundled SQLite and embedded SQL migrations.

This crate currently owns the catalog migration runner and initial empty catalog schema/index proof. It does not scan folders, import photos, mutate originals, write sidecars, manage caches, or expose database access to plugins/MCP.
