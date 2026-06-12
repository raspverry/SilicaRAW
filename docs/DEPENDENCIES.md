# SilicaRAW Dependencies Policy

Status: REQUIRED FOR CODEX IMPLEMENTATION

## Rule

Codex / Claude Code must not add a new dependency without updating this file.

## Project License and Inventory Status

SilicaRAW source code and project documentation are licensed under the MIT License unless a file states otherwise.

This file is the dependency and third-party license inventory for the repository. As of Task 10.6.1, the current Rust/Tauri dependency set is recorded below, no model weights are bundled, and no sample assets are committed as redistributable product fixtures.

Future model weights, sample assets, binary tools, or bundled runtime components must add their own license/source/hash records before they are committed or shipped.

## Deferred Dependency Decisions

- Task 11.7.1 does not add an EXIF or metadata parser dependency. Camera make, camera model, lens model, orientation, and capture-time metadata remain explicitly unavailable until a later task adds a parser and records it in this file.

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

## Internal Workspace Dependencies

Internal path dependencies between `crates/silica-*` packages are allowed when they preserve the architecture boundaries and do not introduce an external package. They are still recorded here when added so agents can distinguish workspace coupling from third-party dependency growth.

```txt
Name: silica-catalog
Version: workspace
Purpose: Catalog domain schema contract for local alpha storage.
License: project internal
Repository/Homepage: this repository
Used by: crates/silica-storage
Why needed: Phase 4.1 keeps the required catalog table/index/version contract in the catalog domain crate while `silica-storage` owns SQLite migration execution.
Alternatives considered: duplicate table/index lists in `silica-storage`, storage-only contract ownership.
Risk notes: Keep the dependency direction one-way from storage to catalog. Do not let catalog depend on storage or rusqlite.
Binary size impact: None meaningful; internal constants only.
Security notes: No runtime I/O or SQL access is added by the contract dependency.
Verification source: local workspace Cargo metadata and Phase 4.1 tests.
```

```txt
Name: silica-storage
Version: workspace
Purpose: Local library create/open persistence API.
License: project internal
Repository/Homepage: this repository
Used by: crates/silica-core
Why needed: Phase 4.2 routes local library create/open commands through Core while keeping SQLite details inside `silica-storage`.
Alternatives considered: direct desktop-to-storage calls, duplicate filesystem/catalog logic in Core.
Risk notes: Keep SQLite connection and migration details behind storage APIs. Do not expose raw database access through Core.
Binary size impact: Internal workspace code only.
Security notes: Core must preserve original-file safety and pass mutations through typed APIs.
Verification source: local workspace Cargo metadata and Phase 4.2 core/storage tests.
```

```txt
Name: silica-core
Version: workspace
Purpose: Desktop command boundary for local library, edit, and export workflows.
License: project internal
Repository/Homepage: this repository
Used by: apps/desktop/src-tauri
Why needed: Phase 4.2 exposes minimal Tauri commands through Core instead of letting the desktop shell call storage directly; Phase 5.4 reuses the same boundary for command-level JPEG sRGB export.
Alternatives considered: desktop shell calling `silica-storage` directly.
Risk notes: Keep app shell thin. Do not add RAW, Metal viewer, import scanner, plugin, MCP, or MLX behavior through this dependency.
Binary size impact: Internal workspace code only.
Security notes: Desktop commands accept local paths and must not mutate original photo folders.
Verification source: local workspace Cargo metadata, Phase 4.2 desktop command test, and Phase 5.4 export command test.
```

```txt
Name: silica-edit
Version: workspace
Purpose: Typed edit graph construction, validation, and exposure/contrast update contract.
License: project internal
Repository/Homepage: this repository
Used by: crates/silica-storage, crates/silica-core
Why needed: Phase 5.3 persists schema-valid active edit graphs and lets Core validate draft exposure/contrast updates before render requests or commit.
Alternatives considered: duplicate edit graph JSON construction in storage/core, untyped JSON strings across the command boundary.
Risk notes: Keep edit graph ownership inside `silica-edit`; storage should persist validated graphs, not invent schema fields.
Binary size impact: Internal workspace code only.
Security notes: Imported or stored edit graph JSON remains untrusted and must validate before use.
Verification source: local workspace Cargo metadata and Phase 5.3 edit flow tests.
```

```txt
Name: silica-export
Version: workspace
Purpose: Local alpha JPEG sRGB export boundary.
License: project internal
Repository/Homepage: this repository
Used by: crates/silica-core
Why needed: Phase 5.4 keeps JPEG file writing and export-specific validation out of Core, Render, and Storage while still allowing Core to orchestrate the local alpha workflow.
Alternatives considered: direct JPEG encoding in `silica-core`, postponing export until UI screens, placeholder output files.
Risk notes: This crate handles already-rendered raster inputs only. It does not implement RAW decoding, a Metal renderer, ICC fixture validation, or broad fallback export paths.
Binary size impact: Internal workspace code only; external image codec impact is tracked under `image`.
Security notes: Reject exporting over the original source path and treat image inputs as untrusted files.
Verification source: local workspace Cargo metadata and Phase 5.4 `silica-export` tests.
```

### Tauri Runtime

```txt
Name: tauri
Version: 2.11.2
Purpose: Desktop application shell
License: Apache-2.0 OR MIT
Repository/Homepage: https://github.com/tauri-apps/tauri
Used by: apps/desktop/src-tauri
Why needed: macOS desktop shell with Rust backend and local static frontend for the Phase 2 packaging spike.
Alternatives considered: SwiftUI/AppKit shell, Electron
Risk notes: Tauri + Metal viewer integration requires Spike 001. If it fails, prefer SwiftUI/AppKit shell + Rust Core.
Binary size impact: Lower than Electron in principle; measure actual `.app` and `.dmg` after Phase 2 bundle output exists.
Security notes: Use minimal permissions/capabilities. No network/telemetry by default.
Verification source: `cargo info tauri` for 2.11.2 metadata; Tauri v2 documentation at https://v2.tauri.app/.
```

### Tauri Build

```txt
Name: tauri-build
Version: 2.6.2
Purpose: Tauri build-time code generation and configuration handling.
License: Apache-2.0 OR MIT
Repository/Homepage: https://github.com/tauri-apps/tauri
Used by: apps/desktop/src-tauri
Why needed: Required build dependency for the Tauri application crate.
Alternatives considered: None while Tauri is selected for the Phase 2 shell spike.
Risk notes: Keep version compatible with the selected Tauri runtime and CLI.
Binary size impact: Build-time only; no direct runtime bundle impact expected.
Security notes: Build-time code generation should remain limited to the local Tauri config and capabilities.
Verification source: `cargo info tauri-build` for 2.6.2 metadata; Tauri v2 documentation at https://v2.tauri.app/.
```

### Tauri Dialog Plugin

```txt
Name: tauri-plugin-dialog
Version: 2.7.1
Purpose: Native macOS folder and save-file dialogs for the local alpha path picker workflow.
License: Apache-2.0 OR MIT
Repository/Homepage: https://github.com/tauri-apps/plugins-workspace
Used by: apps/desktop/src-tauri in Phase 5.6.3
Why needed: Lets installed-alpha users choose a library folder, import folder, and JPEG export output path without typing absolute paths.
Alternatives considered: manual text-only path entry, custom HTML file inputs, Rust-only blocking dialogs behind custom commands
Risk notes: Keep capability permissions limited to `dialog:allow-open` and `dialog:allow-save`; do not broaden to unrelated message dialogs unless the product needs them.
Binary size impact: Adds the Tauri dialog plugin and its runtime file dialog support; measure final `.app` and `.dmg` size during Phase 6 packaging QA.
Security notes: Dialog selection only fills existing path fields. File/catalog mutation remains behind typed Rust commands that preserve original files.
Verification source: `cargo info tauri-plugin-dialog` for 2.7.1 metadata; Tauri v2 dialog plugin documentation at https://v2.tauri.app/plugin/dialog/.
```

### Tauri CLI

```txt
Name: tauri-cli
Version: 2.11.2
Purpose: Local development command for `cargo tauri build --no-bundle` and bundle generation.
License: Apache-2.0 OR MIT
Repository/Homepage: https://github.com/tauri-apps/tauri
Used by: local developer machines and CI/release workflows when packaging is enabled.
Why needed: Required to validate and produce Tauri app and DMG artifacts.
Alternatives considered: npm `@tauri-apps/cli`, direct bundler invocation.
Risk notes: Keep CLI version compatible with the selected Tauri runtime.
Binary size impact: Development tool only; not bundled in the app.
Security notes: Use local build commands only. Do not add updater, signing, notarization, or network publishing behavior in Phase 2.
Verification source: `cargo info tauri-cli` for 2.11.2 metadata; Tauri v2 documentation at https://v2.tauri.app/.
```

### Objective-C Runtime Bindings

```txt
Name: objc2
Version: 0.6.4
Purpose: Objective-C runtime interface for the macOS-only Spike 001 native view bridge.
License: MIT
Repository/Homepage: https://github.com/madsmtm/objc2
Used by: apps/desktop/src-tauri behind the `metal-host-spike` feature.
Why needed: Required to define a minimal MTKView subclass and retain the Metal delegate during the Tauri + native Metal viewer spike.
Alternatives considered: Swift/AppKit spike code, raw Objective-C FFI, no native bridge spike.
Risk notes: Keep isolated behind a non-default feature. Do not spread Objective-C runtime calls into product code without a follow-up bridge design.
Binary size impact: No default app impact while the feature is disabled. Feature builds link native framework bridge code for the spike.
Security notes: Uses local macOS runtime APIs only. Avoid exposing these handles to webview IPC or plugins.
Verification source: `cargo info objc2@0.6.4 --verbose`; repository license and docs at https://github.com/madsmtm/objc2.
```

### AppKit Bindings

```txt
Name: objc2-app-kit
Version: 0.3.2
Purpose: AppKit NSWindow, NSView, NSEvent, and autoresizing APIs for Spike 001.
License: Zlib OR Apache-2.0 OR MIT
Repository/Homepage: https://github.com/madsmtm/objc2
Used by: apps/desktop/src-tauri behind the `metal-host-spike` feature; crates/silica-decode behind the `core-image-raw-probe` feature.
Why needed: Required to access the Tauri window content view, attach a native MTKView, and log mouse/trackpad event routing.
Alternatives considered: Swift/AppKit shim, raw Objective-C message sends only, Tauri webview-only proof.
Risk notes: Feature-gated proof only. Event mapping from this spike does not finalize the product viewer architecture.
Binary size impact: No default app impact while the feature is disabled. Spike builds link AppKit, already present on macOS.
Security notes: Native view handles must remain internal and must not be exposed through IPC.
Verification source: `cargo info objc2-app-kit` for 0.3.2 metadata; repository license and docs at https://github.com/madsmtm/objc2.
```

### Foundation Bindings

```txt
Name: objc2-foundation
Version: 0.3.2
Purpose: Foundation NSObject and geometry types used by the Spike 001 AppKit/Metal bridge.
License: MIT
Repository/Homepage: https://github.com/madsmtm/objc2
Used by: apps/desktop/src-tauri behind the `metal-host-spike` feature.
Why needed: Required by AppKit and MetalKit wrapper types used in the native view proof.
Alternatives considered: Raw CoreGraphics structs and raw Objective-C FFI.
Risk notes: Keep scoped to platform bridge code. Do not introduce broader Foundation usage until a native bridge design is accepted.
Binary size impact: No default app impact while the feature is disabled. Spike builds use macOS system frameworks.
Security notes: No file, network, or user data access is introduced by this dependency.
Verification source: `cargo info objc2-foundation@0.3.2 --verbose`; repository license and docs at https://github.com/madsmtm/objc2.
```

### CoreGraphics Bindings

```txt
Name: objc2-core-graphics
Version: 0.3.2
Purpose: CoreGraphics framework linkage for Metal device creation during Spike 001.
License: Zlib OR Apache-2.0 OR MIT
Repository/Homepage: https://github.com/madsmtm/objc2
Used by: apps/desktop/src-tauri behind the `metal-host-spike` feature; crates/silica-decode behind the `core-image-raw-probe` feature.
Why needed: `objc2-metal` documents that `MTLCreateSystemDefaultDevice` requires CoreGraphics linkage.
Alternatives considered: Manual `#[link(name = "CoreGraphics", kind = "framework")]` declaration.
Risk notes: Linkage helper only for the spike; keep feature-gated.
Binary size impact: No default app impact while the feature is disabled. CoreGraphics is a macOS system framework.
Security notes: No image capture or display enumeration behavior is added by this use.
Verification source: `cargo info objc2-core-graphics@0.3.2 --verbose`; `objc2-metal` crate note for `MTLCreateSystemDefaultDevice`.
```

### Metal Bindings

```txt
Name: objc2-metal
Version: 0.3.2
Purpose: Metal device, command queue, command buffer, drawable, and render pass APIs for Spike 001.
License: Zlib OR Apache-2.0 OR MIT
Repository/Homepage: https://github.com/madsmtm/objc2
Used by: apps/desktop/src-tauri behind the `metal-host-spike` feature.
Why needed: Required to prove a native Metal render loop can present inside the Tauri app window.
Alternatives considered: Metal-rs, raw Objective-C FFI, Swift shim.
Risk notes: This spike only clears and presents an MTKView drawable. It is not the final renderer or shader pipeline.
Binary size impact: No default app impact while the feature is disabled. Spike builds link the macOS Metal framework.
Security notes: Do not accept untrusted shaders or GPU resources in this spike path.
Verification source: `cargo info objc2-metal` for 0.3.2 metadata; Apple Metal documentation at https://developer.apple.com/documentation/metal.
```

### MetalKit Bindings

```txt
Name: objc2-metal-kit
Version: 0.3.2
Purpose: MTKView binding for the Spike 001 native Metal host proof.
License: Zlib OR Apache-2.0 OR MIT
Repository/Homepage: https://github.com/madsmtm/objc2
Used by: apps/desktop/src-tauri behind the `metal-host-spike` feature.
Why needed: Required to attach a native Metal-backed view to the Tauri/AppKit window hierarchy.
Alternatives considered: Direct CAMetalLayer bridge, Swift AppKit shim, full SwiftUI/AppKit shell.
Risk notes: The proof validates host feasibility but does not finalize viewer layout, event ownership, or render engine architecture.
Binary size impact: No default app impact while the feature is disabled. Spike builds link the macOS MetalKit framework.
Security notes: Keep MTKView internals behind Rust native code and do not expose raw view/device handles to IPC.
Verification source: `cargo info objc2-metal-kit` for 0.3.2 metadata; Apple MetalKit documentation at https://developer.apple.com/documentation/metalkit.
```

### QuartzCore Bindings

```txt
Name: objc2-quartz-core
Version: 0.3.2
Purpose: CAMetalLayer-related support used by the MetalKit Spike 001 bridge.
License: Zlib OR Apache-2.0 OR MIT
Repository/Homepage: https://github.com/madsmtm/objc2
Used by: apps/desktop/src-tauri behind the `metal-host-spike` feature.
Why needed: Required by the selected MetalKit wrapper feature set for MTKView drawable/layer support.
Alternatives considered: Avoid MetalKit and manage CAMetalLayer directly.
Risk notes: Keep feature-gated until the native viewer bridge design is selected.
Binary size impact: No default app impact while the feature is disabled. Spike builds link QuartzCore/CoreAnimation system frameworks.
Security notes: No animation or screen capture behavior is added; usage is limited to Metal layer support.
Verification source: `cargo info objc2-quartz-core` for 0.3.2 metadata; Apple QuartzCore documentation at https://developer.apple.com/documentation/quartzcore.
```

### SQLite binding

```txt
Name: rusqlite
Version: 0.40.1
Purpose: SQLite catalog
License: MIT
Repository/Homepage: https://github.com/rusqlite/rusqlite
Used by: crates/silica-storage
Why needed: local-first SQLite catalog, embedded migrations, parameterized queries, and migration verification.
Alternatives considered: sqlx, refinery + rusqlite, direct sqlite3 bindings, sled, redb.
Risk notes: Synchronous local database access must stay off latency-sensitive UI/render loops. Migrations must be tested on empty and existing databases.
Binary size impact: Uses bundled SQLite through `libsqlite3-sys`; measure final `.app` and `.dmg` size during packaging phases.
Security notes: No raw SQL access from plugins/MCP. Treat catalog paths and sidecar payloads as untrusted input.
Verification source: `cargo info rusqlite` for 0.40.1 metadata; rusqlite docs at https://docs.rs/rusqlite/; SQLite docs at https://www.sqlite.org/docs.html.
Status after Spike 004: selected and added to `crates/silica-storage` with `default-features = false` and `features = ["bundled"]`.
```

### SQLite Native Binding

```txt
Name: libsqlite3-sys
Version: 0.38.1
Purpose: Native SQLite FFI used transitively by rusqlite.
License: MIT
Repository/Homepage: https://github.com/rusqlite/rusqlite
Used by: crates/silica-storage through rusqlite.
Why needed: Provides the SQLite C API binding and bundled SQLite build path.
Alternatives considered: system SQLite linkage, sqlx SQLite driver, direct sqlite3 bindings.
Risk notes: Bundled SQLite improves build determinism but adds native build work and binary size. Recheck before notarized release packaging.
Binary size impact: Bundles SQLite into the app binary path; measure during packaging phases.
Security notes: Keep SQLite access behind typed storage APIs; enable foreign key enforcement per connection.
Verification source: `cargo info libsqlite3-sys` for 0.38.1 metadata; rusqlite repository at https://github.com/rusqlite/rusqlite.
```

### Serialization

```txt
Name: serde
Version: 1.0.228
Purpose: Derive-backed typed serialization and deserialization.
License: MIT OR Apache-2.0
Repository/Homepage: https://github.com/serde-rs/serde
Used by: crates/silica-edit in Phase 5.2 and apps/desktop/src-tauri in Phase 5.6.2; expected later for silica-storage, silica-plugin, and silica-mcp when their schema-backed JSON tasks are reached.
Why needed: typed serialization/deserialization for the edit graph JSON schema boundary and structured Tauri command response envelopes.
Alternatives considered: manual JSON parsing, schemars-only workflows
Risk notes: Schema-aware validation is still required; serde derives alone are not full JSON Schema validation.
Binary size impact: Low/typical Rust ecosystem dependency.
Security notes: Validate untrusted edit graph, plugin, model, and MCP manifest JSON before accepting it.
Verification source: `Cargo.lock` after Phase 5.2 and serde repository license section.
```

```txt
Name: serde_json
Version: 1.0.150
Purpose: JSON value, number, map, parsing, and serialization support.
License: MIT OR Apache-2.0
Repository/Homepage: https://github.com/serde-rs/json
Used by: crates/silica-edit in Phase 5.2, crates/silica-storage in Phases 5.3/5.4, crates/silica-core in Phase 5.4, and crates/silica-decode behind `core-image-raw-probe` in Phase 12.2; expected later for silica-plugin and silica-mcp when their schema-backed JSON tasks are reached.
Why needed: edit graph example round-tripping, `extensions` storage, schema-owned JSON values, numeric representation preservation, active edit graph JSON persistence in SQLite, export settings JSON validation, export settings JSON construction at the Core orchestration boundary, and RAW fixture manifest parsing for probe evidence.
Alternatives considered: simd-json, manual JSON parsing, schemars-only workflows
Risk notes: JSON Schema validation rules still need explicit validation or a schema validator; serde_json only parses and serializes JSON.
Binary size impact: Low/typical Rust ecosystem dependency.
Security notes: Treat imported edit graph JSON and stored export settings JSON as untrusted and validate before accepting or reusing them.
Verification source: `cargo info serde_json@1.0.150` during Phase 12.2 and serde_json repository license section.
```

### Raster Image I/O

```txt
Name: image
Version: 0.25.6
Purpose: JPEG decode/encode for the local alpha JPEG sRGB export path, JPEG thumbnail, Loupe preview, Develop preview cache generation, import-time JPEG dimension inspection, and JPEG fixture inspection in integration tests.
License: MIT OR Apache-2.0
Repository/Homepage: https://github.com/image-rs/image
Used by: crates/silica-export at runtime; crates/silica-core and apps/desktop/src-tauri as dev-dependencies for JPEG test fixture generation and inspection.
Why needed: Task 5.4 must produce a real JPEG file, inspect the exported JPEG, and verify original files remain unchanged without implementing RAW decoding or the Metal viewer. Tasks 5.6.4, 5.6.5, and 5.6.6 reuse the same JPEG-only runtime image path to create disposable grid thumbnails, Loupe previews, and adjusted Develop previews for JPEG/JPG originals. Task 11.7.3 reuses `image::image_dimensions` to persist JPEG/JPG width and height during import without adding an EXIF parser.
Alternatives considered: placeholder export bytes, direct `zune-jpeg` use, Core Image export bridge, postponing export until UI implementation.
Risk notes: Pinned exactly to 0.25.6 because it declares Rust 1.70 compatibility while the workspace targets Rust 1.80. Default features are disabled and only the `jpeg` feature is enabled. This does not prove final ICC/color correctness.
Binary size impact: Adds the JPEG-only subset of `image` and its transitive codec support; measure final `.app` and `.dmg` during packaging QA.
Security notes: Treat decoded image files as untrusted. Export path protection is enforced before writing so original source files are not overwritten.
Verification source: `cargo info image@0.25.6`, local Cargo metadata after Phase 5.4, Task 5.4 export tests, Task 5.6.4 thumbnail cache tests, Task 5.6.5 Loupe preview cache tests, Task 5.6.6 Develop preview tests, and Task 11.7.3 metadata import tests.
```

### RAW Decode — Core Image

```txt
Name: objc2-core-image
Version: 0.3.2
Purpose: Feature-gated Core Image RAW probe bindings for Task 12.1.
License: Zlib OR Apache-2.0 OR MIT
Repository/Homepage: https://github.com/madsmtm/objc2
Used by: crates/silica-decode behind the `core-image-raw-probe` feature.
Why needed: Access Core Image RAW probe APIs without adding LibRaw.
Alternatives considered: raw Objective-C FFI, Swift shim, LibRaw, no probe.
Risk notes: Non-default macOS proof only; no product RAW pixels. Enables `objc2-image-io` transitively for Core Image URL/image source support.
Binary size impact: No default build impact while feature is disabled. Feature builds link Core Image, already present on macOS.
Security notes: Reads local fixture paths only; must not mutate originals.
Verification source: `cargo info objc2-core-image@0.3.2 --verbose`.
```

```txt
Name: sha2
Version: 0.10.9
Purpose: Compute SHA-256 source hashes for feature-gated RAW and color fixture probe evidence.
License: MIT OR Apache-2.0
Repository/Homepage: https://github.com/RustCrypto/hashes
Used by: crates/silica-decode behind the `core-image-raw-probe` feature; crates/silica-render behind the `color-probe` feature.
Why needed: Task 12.2 RAW evidence and Task 13.3 color profile evidence must verify source hashes and original-file preservation.
Alternatives considered: Python-only hash verification, existing partial FNV-style test hash, platform-specific hashing APIs.
Risk notes: Non-default probe features only; do not use partial hashes for fixture evidence.
Binary size impact: No default build impact while feature is disabled. Pure Rust hashing code is linked only into feature builds.
Security notes: Reads local fixture files only and does not mutate originals.
Verification source: `cargo info sha2@0.10.9`; Task 13.3 color-probe tests.
```

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
Status after Spike 002: selected as first implementation target. Task 12.1 adds a non-default `core-image-raw-probe` binding path for proof work only; product RAW pixels remain out of scope.
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
Verification source: LibRaw documentation at https://www.libraw.org/docs; selected Rust binding still TBD.
Status after Spike 002: deferred until legal RAW fixtures prove a camera-support gap. No LibRaw dependency has been added.
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
Status after ADR 0005: deferred from local alpha. No MLX dependency, model loader, model asset, or inference runtime has been added.
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
