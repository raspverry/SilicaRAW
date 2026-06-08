# SilicaRAW

SilicaRAW is an early-stage, open-source RAW photo editor for Apple Silicon.

This repository currently contains the monorepo foundation, a minimal Tauri desktop shell, and the first local library create/open path. Product editing features are intentionally still out of scope.

## Current Scope

- Rust workspace root
- Minimal Tauri desktop shell
- Local library folder create/open path
- Developer-only unsigned `.app` and `.dmg` packaging path
- Core crate boundaries from the architecture documents
- SQLite migration foundation for empty catalog schema, required indexes, and catalog schema contract
- MLX boundary crate is explicitly deferred from local alpha
- Per-crate responsibility notes

## Workspace Layout

```txt
apps/
  desktop/          Minimal Tauri shell and packaging skeleton
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
scripts/harness/check.sh
```

Phase 2 desktop packaging checks:

```bash
cd apps/desktop/src-tauri
cargo tauri build --no-bundle
cargo tauri build --bundles app,dmg --ci --no-sign
```

## Guardrails

- Do not modify original photo files.
- Do not add RAW decoder behavior without explicit fixture-backed scope; Spike 002 selected Core Image primary and deferred LibRaw.
- Do not turn the Metal spike into a product viewer without the Path B native bridge contract.
- Do not claim color correctness without tagged fixtures and explicit color comparison evidence.
- Do not expose raw SQLite access outside typed storage/core APIs.
- Do not add MLX, MCP, plugin behavior, telemetry, cloud sync, or network upload by default.
- Document every new dependency in `docs/DEPENDENCIES.md`.
