# silica-core

Core coordination boundary for SilicaRAW.

This crate will own the high-level command surface that coordinates catalog, edit, storage, render, export, permission, MLX, plugin, and MCP boundaries through explicit APIs.

Phase 4.2 adds the first command-facing local library APIs:

- create a local library folder
- open an existing local library folder
- return the active library root, catalog path, and schema version

Phase 4.4 extends the command-facing boundary with folder import delegation and photo flag read/write APIs for rating, pick, reject, and color label persistence.

Core delegates SQLite and filesystem details to `silica-storage`. It does not decode RAW files, render previews, write sidecars, expose plugins/MCP, or run MLX behavior.
