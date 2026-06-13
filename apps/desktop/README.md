# silica-desktop

Minimal desktop application shell for SilicaRAW.

This package contains the Tauri shell, packaging skeleton, Phase 4.2 minimal local library create/open entry point, Phase 4.4 photo flag command entry points, a Phase 5.1 preview status command, Phase 5.3 and Phase 17 Develop preview/commit command entry points, and the Phase 16 undo/history command surface used by the Develop history panel.

It does not include broad product UI screens, unfenced direct history mutation, a Metal viewer, RAW decoding, pixel rendering, sidecar writing, MLX, plugin behavior, or MCP behavior.

## Layout

- `static/`: local static frontend served by Tauri without a dev server.
- `src-tauri/`: Rust Tauri application crate and bundle configuration.

## Validation

From the repository root:

```bash
cargo build --workspace
```

From `apps/desktop/src-tauri` after installing `tauri-cli`:

```bash
cargo tauri build --no-bundle
```
