# silica-core

Core coordination boundary for SilicaRAW.

This crate will own the high-level command surface that coordinates catalog, edit, storage, render, export, permission, MLX, plugin, and MCP boundaries through explicit APIs.

Phase 4.2 adds the first command-facing local library APIs:

- create a local library folder
- open an existing local library folder
- return the active library root, catalog path, and schema version

Phase 4.4 extends the command-facing boundary with folder import delegation and photo flag read/write APIs for rating, pick, reject, and color label persistence.

Phase 5.1 adds a preview session API that reads a catalog photo, asks `silica-decode` for preview readiness, and asks `silica-render` for the render-side readiness state.

Phase 5.3 adds exposure/contrast edit flow APIs. Draft preview updates validate an edited graph and return a render request without writing to SQLite; commit/release persists the final active edit graph through `silica-storage`.

Core delegates SQLite, filesystem details, sidecar JSON validation, and sidecar rebuild dry-run logic to `silica-storage`. It does not decode RAW files, render pixels, write sidecars next to originals, apply restore actions, expose plugins/MCP, run MLX behavior, or perform automatic sidecar sync.

Task 10.3 adds thin sidecar workflow wrappers for explicit sidecar write/read. Core does not duplicate sidecar path or schema logic.

Task 10.4 adds a thin sidecar rebuild dry-run wrapper. Core exposes the report to command-facing callers without duplicating precedence, conflict, or schema handling.
