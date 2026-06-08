# SilicaRAW

SilicaRAW is an early planning-stage, open-source RAW photo editor for Apple Silicon.

This repository currently contains the monorepo foundation only. Product implementation is intentionally out of scope for this scaffold.

## Current Scope

- Rust workspace root
- Desktop application placeholder
- Core crate boundaries from the architecture documents
- Per-crate responsibility notes

## Workspace Layout

```txt
apps/
  desktop/          Placeholder desktop app package
crates/
  silica-core/      Core coordination boundary
  silica-catalog/   Catalog domain boundary
  silica-storage/   Storage and persistence boundary
  silica-decode/    RAW decode abstraction boundary
  silica-render/    Render request and renderer boundary
  silica-edit/      Edit graph boundary
  silica-export/    Export coordination boundary
  silica-mlx/       MLX feature boundary
  silica-plugin/    Plugin boundary
  silica-mcp/       MCP boundary
docs/               Product and architecture specifications
schemas/            Authoritative JSON schemas
```

## Wiki

The public, LLM-readable project wiki starts at [`docs/wiki/index.md`](docs/wiki/index.md).

## Development

```bash
cargo metadata --format-version 1 --no-deps
cargo build --workspace
cargo test --workspace
```

## Guardrails

- Do not modify original photo files.
- Do not add RAW decoding before the decoder spike.
- Do not add the Metal viewer before the Tauri + Metal spike.
- Do not add MLX, MCP, plugin behavior, telemetry, cloud sync, or network upload by default.
- Document every new dependency in `docs/DEPENDENCIES.md`.
