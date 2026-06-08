# silica-desktop

Minimal desktop application shell for SilicaRAW.

This package contains the Tauri shell, packaging skeleton, and Phase 4.2 minimal local library create/open entry point.

It does not include broad product UI screens, a Metal viewer, RAW decoding, import scanning, edit controls, MLX, plugin behavior, or MCP behavior.

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
