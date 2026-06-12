# SilicaRAW

SilicaRAW is an early-stage, open-source RAW photo editor for Apple Silicon.

The current delivery target is a local macOS alpha that can be installed from a DMG and complete a minimal JPEG/JPG editor workflow without modifying original photo files. Broad RAW decoding, the product Metal viewer, MLX, MCP, plugins, cloud sync, telemetry, auto-update, Homebrew, and Mac App Store distribution are intentionally out of the current alpha scope.

SilicaRAW is not production-ready. Current repository claims are limited to fixture-backed behavior in the harness and documented manual checks.

## Current Scope

- Rust workspace root
- Tauri desktop shell for the local alpha app
- Local library folder create/open path
- Non-recursive folder import scanner for catalog candidates
- Rating, pick, reject, and color label persistence in SQLite `photo_flags`
- JPEG/JPG thumbnail, loupe preview, Develop preview, exposure/contrast, edit-state persistence, and JPEG sRGB export path
- Undo/redo for edit and culling checkpoints plus a real Develop history panel backed by catalog history
- Append-only action log evidence for sensitive local actions
- Product cache clear command for disposable cache directories
- Connected static UI vertical slice for the local alpha workflow
- Clear RAW/unsupported/missing-file blocked states without RAW decoding claims
- Developer-only unsigned `.app` and `.dmg` preview artifact path
- Core crate boundaries from the architecture documents
- SQLite migration foundation for empty catalog schema, required indexes, and catalog schema contract
- MLX boundary crate is explicitly deferred from local alpha
- Per-crate responsibility notes

## Distribution Status

The user-ready local distribution target is a signed and notarized GitHub Release DMG containing `SilicaRAW.app`. That path is currently blocked until Apple Developer Program funding, a Developer ID Application certificate, and notarization credentials are available.

While blocked, maintainers may build unsigned developer-preview DMG artifacts for internal testing. These artifacts are not signed, not notarized, and not user-ready; Gatekeeper warnings are expected.

Release docs:

- [Local DMG Distribution Plan](docs/wiki/roadmaps/local-dmg-distribution-plan.md)
- [Local DMG Release Runbook](docs/wiki/roadmaps/local-dmg-release-runbook.md)
- [Developer Preview Artifact Runbook](docs/wiki/roadmaps/developer-preview-artifact-runbook.md)
- [Post-Alpha Master Execution Plan](docs/wiki/roadmaps/post-alpha-master-execution-plan.md)

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

## Public Trust

- License: [MIT](LICENSE).
- Dependency and third-party license inventory: [`docs/DEPENDENCIES.md`](docs/DEPENDENCIES.md).
- Public trust boundaries: [`docs/wiki/topics/public-trust.md`](docs/wiki/topics/public-trust.md).
- Contribution guide: [`CONTRIBUTING.md`](CONTRIBUTING.md).
- Security policy: [`SECURITY.md`](SECURITY.md).

## Known Limitations

- RAW decoding is not implemented in the product workflow yet.
- Color correctness is not claimed until tagged fixture evidence and tolerance checks exist.
- The product Metal viewer is not implemented yet.
- MLX, MCP, plugins, cloud sync, telemetry, auto-update, Homebrew, and Mac App Store distribution are deferred.
- Signed and notarized release DMGs are blocked until Developer ID funding and credentials are available.

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
- Treat `MockupUI/` as the high-fidelity UI target reference when implementing screens.
- Do not expose raw SQLite access outside typed storage/core APIs.
- Do not add MLX, MCP, plugin behavior, telemetry, cloud sync, or network upload by default.
- Document every new dependency in `docs/DEPENDENCIES.md`.
