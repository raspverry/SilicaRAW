# SilicaRAW Dependencies Policy

Status: REQUIRED FOR CODEX IMPLEMENTATION

## Rule

Codex / Claude Code must not add a new dependency without updating this file.

## Required Entry Format

```txt
Name:
Version:
Purpose:
License:
Repository/Homepage:
Used by:
Why needed:
Alternatives considered:
Risk notes:
Binary size impact:
Security notes:
Verification source:
```

## Initial Expected Dependencies

These are expected but still require version-specific confirmation during implementation.

### Tauri

```txt
Name: tauri
Version: TBD
Purpose: Desktop application shell
License: MIT or MIT/Apache-2.0 where applicable
Repository/Homepage: https://github.com/tauri-apps/tauri
Used by: apps/desktop
Why needed: macOS desktop shell with Rust backend and web frontend
Alternatives considered: SwiftUI/AppKit shell, Electron
Risk notes: Tauri + Metal viewer integration requires Spike 001. If it fails, prefer SwiftUI/AppKit shell + Rust Core.
Binary size impact: Lower than Electron, verify actual build size.
Security notes: Use minimal permissions/capabilities. No network/telemetry by default.
Verification source: Tauri repository license section.
```

### SQLite binding

```txt
Name: rusqlite or sqlx
Version: TBD
Purpose: SQLite catalog
License: rusqlite is MIT; sqlx license must be verified if chosen
Repository/Homepage: https://github.com/rusqlite/rusqlite
Used by: crates/silica-catalog, crates/silica-storage
Why needed: local-first SQLite catalog, migrations, query persistence
Alternatives considered: sled, redb, direct sqlite3 bindings
Risk notes: Must support WAL, migrations, parameterized queries, and robust error handling.
Binary size impact: Verify after selection.
Security notes: No raw SQL access from plugins/MCP.
Verification source: rusqlite repository license statement.
```

### Serialization

```txt
Name: serde / serde_json
Version: TBD
Purpose: Edit graph, sidecar, manifest serialization
License: MIT OR Apache-2.0
Repository/Homepage: https://github.com/serde-rs/serde and https://github.com/serde-rs/json
Used by: silica-edit, silica-storage, silica-plugin, silica-mcp
Why needed: typed serialization/deserialization for JSON schemas
Alternatives considered: simd-json, schemars-only workflows
Risk notes: Schema validation still required; serde alone is not schema validation.
Binary size impact: Low/typical Rust ecosystem dependency.
Security notes: Validate untrusted plugin/model/MCP manifests.
Verification source: serde and serde_json repository license sections.
```

### RAW Decode — Core Image

```txt
Name: Core Image RAW backend
Version: Apple platform framework
Purpose: macOS-native RAW decode path
License: Apple platform framework, not bundled as third-party OSS
Repository/Homepage: Apple Developer documentation
Used by: crates/silica-decode
Why needed: Apple-native RAW / ProRAW / DNG path and ColorSync/Core Image experiments
Alternatives considered: LibRaw primary
Risk notes: Supported formats depend on Apple. Less low-level control.
Binary size impact: platform framework
Security notes: Decode failures must be non-fatal.
Verification source: Apple Developer documentation.
```

### RAW Decode — LibRaw

```txt
Name: LibRaw / Rust binding TBD
Version: TBD
Purpose: Broad RAW format support fallback
License: Must verify LibRaw and selected Rust binding license before adding
Repository/Homepage: https://www.libraw.org/
Used by: crates/silica-decode
Why needed: broader camera support fallback
Alternatives considered: Core Image RAW only
Risk notes: FFI/distribution/color pipeline complexity; binding maturity risk.
Binary size impact: TBD
Security notes: Treat decoder input as untrusted; handle corrupt RAW safely.
Verification source: must be completed during Spike 002.
```

### MLX

```txt
Name: MLX
Version: TBD
Purpose: Local Apple Silicon ML features
License: MIT for Apple's mlx repository as currently published; verify selected package/binding before adding
Repository/Homepage: https://github.com/ml-explore/mlx
Used by: crates/silica-mlx
Why needed: Apple Silicon-local MLX features: masks, auto tone, culling, denoise/upscale later
Alternatives considered: Core ML, Python sidecar, no ML features
Risk notes: Model weights have separate licenses. Rust bindings may have separate licenses.
Binary size impact: TBD
Security notes: Model downloads must be opt-in, license/source/hash recorded.
Verification source: MLX repository license; selected binding must be checked separately.
```

## Prohibited Without Review

```txt
Telemetry libraries
Cloud sync SDKs
Analytics SDKs
Network upload libraries
Unlicensed model loaders
Arbitrary plugin runtimes
```

## Review Rule

Even when a license is prefilled above, the exact crate/package version must be checked before adding it to the project.
