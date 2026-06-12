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

Core delegates SQLite, filesystem details, sidecar JSON validation, sidecar status, and sidecar rebuild dry-run logic to `silica-storage`. It does not decode RAW files, render pixels, write sidecars next to originals, apply restore actions, expose plugins/MCP, run MLX behavior, or perform automatic sidecar writes.

Task 10.3 adds thin sidecar workflow wrappers for explicit sidecar write/read. Core does not duplicate sidecar path or schema logic.

Task 10.4 adds a thin sidecar rebuild dry-run wrapper. Core exposes the report to command-facing callers without duplicating precedence, conflict, or schema handling.

Task 16.4 adds a thin `list_photo_history` wrapper for the Develop history panel. Core keeps UI callers on the same undo/history boundary and does not expose raw SQLite or arbitrary checkpoint mutation.

Task 16.5 adds thin append/read wrappers for the action log and records sensitive local Core actions: import by reference, sidecar write, JPEG export, RAW-derived JPEG export, and disposable cache clear. Core keeps this as evidence, not undo behavior, and does not expose plugin/MCP raw database writes.

Task 16.6 adds a thin `get_photo_sidecar_status` wrapper. History-changing Core calls rely on storage to mark clean sidecars as `catalog_newer` without hidden sidecar file writes.

Task 17.2.1 adds white-balance-family preview and commit APIs. Core validates the edit graph, keeps draft previews non-persistent, commits one undoable history checkpoint, and passes the committed white-balance state into JPEG export settings.

Task 17.2.2 adds tone recovery preview and commit APIs for highlights, shadows, whites, and blacks under the same draft/commit/export boundary.

Task 17.2.3 adds color presence preview and commit APIs for vibrance and saturation under the same draft/commit/export boundary.
