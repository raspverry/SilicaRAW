# silica-desktop

Minimal desktop application shell for SilicaRAW.

This package contains the Tauri shell, packaging skeleton, Phase 4.2 minimal local library create/open entry point, Phase 4.4 photo flag command entry points, a Phase 5.1 preview status command, and Phase 5.3 exposure/contrast preview/commit command entry points.

It does not include broad product UI screens, a Metal viewer, RAW decoding, pixel rendering, sidecar writing, MLX, plugin behavior, or MCP behavior.

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
