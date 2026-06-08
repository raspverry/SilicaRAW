# SilicaRAW

Open-source RAW photo editing for Apple Silicon.

**Metal-first editing. Local-first workflow. MLX-powered tools when you need them.**

> Status: early planning / pre-alpha. SilicaRAW is not ready for production photo work yet.

## Why SilicaRAW?

Lightroom is powerful but subscription-based. Many open-source RAW tools are powerful but not Mac-native enough. SilicaRAW aims to build a modern, Apple Silicon-first, local-first RAW editor with a beautiful Apple-like UX.

## What It Is

- A macOS RAW photo editor
- A non-destructive editor
- A Metal-first rendering project
- A Rust/Tauri desktop app
- A future MLX-assisted editor
- A local-first open-source creative tool

## What It Is Not

- Not an AI image generator
- Not a cloud photo service
- Not a Photoshop replacement
- Not a Lightroom killer today
- Not production-ready yet

## Planned Features

- [ ] Library grid
- [ ] RAW import
- [ ] Ratings / Pick / Reject
- [ ] Metal preview viewer
- [ ] Basic Develop controls
- [ ] Edit graph persistence
- [ ] Color-managed export
- [ ] Tone curve / HSL
- [ ] Sidecar JSON
- [ ] MLX Subject/Sky Mask
- [ ] MCP read-only automation

## Tech Stack

- Rust
- Tauri
- Metal
- MLX
- SQLite
- Core Image / LibRaw exploration

## Privacy

- Photos stay local.
- Edits stay local.
- No telemetry by default.
- No cloud sync.
- MLX features run locally.
- MCP is off by default.

## Current Roadmap

See `ROADMAP_DRAFT.md` and `docs/18_Final_Master_Plan.md`.

## Contributing

See `github/CONTRIBUTING_DRAFT.md`.

## License

License is not finalized yet. Do not public-launch before license decision.

---

## v1.1 Planning Status

This documentation bundle includes external-review fixes:

- Tauri + Metal fallback strategy
- RAW decoder decision gate
- Edit Graph JSON Schema v0.1
- SQLite index requirements
- Benchmark fixture specification
- License/dependency gates

SilicaRAW remains in planning / pre-implementation stage. Do not treat it as production software.
