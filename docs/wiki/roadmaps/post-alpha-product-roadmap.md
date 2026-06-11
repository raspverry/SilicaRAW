---
title: Post-Alpha Product Roadmap
status: active
audience: all
updated: 2026-06-11
source_of_truth: docs/13_Development_Roadmap.md
---

# Post-Alpha Product Roadmap

## Summary

This roadmap starts after the local DMG alpha distribution track is complete.

The local DMG plan proves that a user can download a GitHub Release DMG, install `SilicaRAW.app`, and complete the minimal local alpha workflow. This post-alpha roadmap defines the atomic product tasks needed to grow that alpha into a credible RAW editor.

The order is deliberate:

```txt
evidence and trust gates
-> real library/session behavior
-> RAW, color, and Metal foundations
-> editor depth, masks, and export breadth
-> permissioned AI, plugins, and MCP
-> public beta and v1.0
```

Do not treat MLX, plugins, or MCP as shortcuts around the editor core. They are optional extension layers after the editor is trustworthy.

## Relationship to Local DMG Distribution

This page continues after [Local DMG Distribution Plan](local-dmg-distribution-plan.md) Phase 9.

Phase 6 through Phase 9 remain focused on install, signing, notarization, GitHub Release assets, and release runbooks. This page is not a reason to add RAW, Metal, MLX, plugin, or MCP behavior before the local alpha release gate unless a maintainer explicitly changes scope.

## Global Rules

- Keep each task atomic and committable.
- Preserve original photo files. Never edit, delete, or move originals without explicit user action and a dedicated approved task.
- Use existing crate boundaries from [Architecture Overview](../overview/architecture.md) and `docs/03_System_Architecture.md`.
- Use current `MockupUI/` references only. Do not use `docs/archive/` for implementation.
- Do not add dependencies without updating `docs/DEPENDENCIES.md`.
- Do not create fake/demo product state.
- Do not broaden color, RAW, or export claims before fixture evidence exists.
- Keep MLX, plugin, and MCP work off by default and permissioned.
- Run `scripts/harness/check.sh` before claiming a task is complete unless the task explicitly documents a narrower validation reason.

## Agent Consultation Notes

This roadmap was prepared after splitting review across three axes:

- RAW decode, Metal viewer, and color pipeline.
- Library, Develop UI, export, metadata, history, and visual QA.
- MLX, plugins, MCP, permissioning, OSS trust, beta, and v1.0 gates.

The combined conclusion is that product breadth should not start with flashy AI or broad controls. It should start with evidence, sidecars, backup/restore, session truth, and fixture-backed RAW/color/Metal work.

## Phase 10: Evidence and Trust Gate

**Goal:** Create the evidence and recovery layer required before broad RAW/editor claims.

**Demo/Validation:**

- A maintainer can inspect legal fixture manifests, tolerance policy, sidecar behavior, backup/restore behavior, and OSS trust docs before enabling deeper product features.

### Task 10.1: Legal RAW and Color Fixture Manifest Contract

- **Location:** `docs/wiki/topics/raw-decoding.md`, `docs/wiki/topics/color-management.md`, `scripts/harness/`
- **Description:** Define the manifest format and validation checks for legal RAW fixture classes and tagged color fixtures.
- **Dependencies:** Local DMG Phase 9
- **Acceptance Criteria:**
  - RAW fixture classes record source, license, hash, dimensions, camera/model metadata when available, and expected decode support state.
  - Color Class F records sRGB, Display P3, and untagged raster fixtures with source, license, hash, and ICC/profile expectations.
  - User photos and unlicensed samples remain uncommitted.
- **Status:** Completed on 2026-06-11. Added the fixture manifest schema, example, and harness contract for legal RAW classes and Color Class F fixture expectations. This records provenance, license, privacy, integrity, expected app states, and future probe expectations only; it does not add RAW decoding, Core Image probing, real fixture files, ICC parsing, or color correctness proof.
- **Validation:**
  - `python3 scripts/harness/check-fixture-manifest-contract.py`
  - `python3 scripts/harness/check-qa-fixtures.py`
  - Future task-specific RAW/color fixture checks once added.

### Task 10.2: Golden Image and Tolerance Policy

- **Location:** `docs/wiki/topics/color-management.md`, `docs/wiki/topics/raw-decoding.md`, `checklists/`
- **Description:** Record what can be automatically compared, what needs manual Preview.app or Photos review, and what claims remain forbidden.
- **Dependencies:** Task 10.1
- **Acceptance Criteria:**
  - Policy separates byte equality, perceptual tolerance, ICC/profile inspection, and manual visual review.
  - Color correctness claims remain blocked until fixture-backed proof exists.
  - RAW support claims are tied to fixture classes, not marketing language.
- **Status:** Completed on 2026-06-11. Added the golden image and tolerance policy baseline and linked it from RAW/color topics. This records evidence categories and forbidden claims only; it does not add golden images, RAW decoding, Core Image probing, ICC parsing, pixel comparison, or color correctness proof.
- **Validation:**
  - `python3 scripts/harness/check-golden-tolerance-policy.py`
  - `python3 scripts/harness/check-md-links.py`

### Task 10.3: Sidecar v1 Read/Write Foundation

- **Location:** `crates/silica-storage`, `crates/silica-core`, `schemas/sidecar.schema.json`, `docs/wiki/topics/catalog.md`
- **Description:** Write and read validated sidecars for edit graph state and portable culling flags.
- **Dependencies:** Task 10.2
- **Design Gate:** [Phase 10 Evidence and Recovery Design](../../superpowers/specs/2026-06-11-phase-10-evidence-recovery-design.md) defines the Task 10.3 through Task 10.6 boundaries before sidecar implementation begins.
- **Acceptance Criteria:**
  - Sidecars validate against `schemas/sidecar.schema.json`.
  - `sidecar.flags` mirrors rating, picked, rejected, and color label only.
  - Catalog-only `edited` and `exported` flags are not written into `sidecar.flags`.
  - Original photo files remain unchanged.
- **Status:** Completed on 2026-06-11. Added explicit library-local sidecar v1 read/write behavior for catalog photo state, active/default edit graph payloads, portable rating/picked/rejected/color-label flags, nested edit graph validation, atomic file writes, validated reads, and `sidecar_status` updates after successful writes. This does not add automatic sidecar sync, catalog rebuild, backup/restore, conflict UI, RAW decoding, color proof, or export proof.
- **Validation:**
  - `python3 scripts/harness/check-sidecar-contract.py`
  - `cargo test -p silica-storage -p silica-core`
  - `scripts/harness/check.sh`

### Task 10.4: Catalog Rebuild Dry-Run from Sidecars

- **Location:** `crates/silica-storage`, `crates/silica-core`, `checklists/QA_CHECKLIST.md`
- **Description:** Add a deterministic dry-run that reports how a catalog would rebuild from sidecars without mutating the live catalog.
- **Dependencies:** Task 10.3
- **Design Gate:** [Phase 10 Evidence and Recovery Design](../../superpowers/specs/2026-06-11-phase-10-evidence-recovery-design.md) requires dry-run rebuild semantics before backup/restore work.
- **Acceptance Criteria:**
  - Rebuild precedence follows `sidecar.flags`, then `edit_graph.metadata`, then defaults.
  - Conflicts are reported, not silently overwritten.
  - Dry-run output is stable enough for tests.
- **Status:** Completed on 2026-06-11. Added deterministic sidecar rebuild dry-run reports through `silica-storage` and thin `silica-core` wrappers. The report scans library-local sidecars in stable order, resolves portable flags by `sidecar.flags` then `edit_graph.metadata` then defaults, reports malformed/schema-invalid sidecars, photo-id mismatches, flag/metadata disagreements, and catalog reconciliation conflicts, and leaves the live catalog unchanged. This does not apply restore actions, overwrite catalog state, add conflict UI, rescan originals, or add backup archive behavior.
- **Validation:**
  - `cargo test -p silica-storage -p silica-core`
  - `scripts/harness/check.sh`

### Task 10.5: Backup, WAL, Checkpoint, and Restore Policy

- **Location:** `crates/silica-storage`, `docs/wiki/topics/data-safety.md`, `checklists/QA_CHECKLIST.md`
- **Description:** Define and test backup/restore behavior for catalogs, sidecars, edit states, export records, and migrations.
- **Dependencies:** Task 10.4
- **Design Gate:** [Phase 10 Evidence and Recovery Design](../../superpowers/specs/2026-06-11-phase-10-evidence-recovery-design.md) separates durable recovery data from disposable caches and original referenced files.
- **Acceptance Criteria:**
  - Catalog backup does not include disposable caches.
  - Restore preserves edit states, flags, sidecar status, and export history.
  - Migration failures have a documented recovery path.
- **Status:** Completed on 2026-06-11. Task 10.5.1 added the [Backup and Restore](../topics/backup-restore.md) backup/WAL/checkpoint/restore policy and static recovery-policy harness. Task 10.5.2 added checkpointed backup artifacts with `catalog.db`, `sidecars/`, and `backup-manifest.json` under `backups/`. Task 10.5.3 added staged restore, existing-target rollback copies, restored catalog/sidecar preservation, newer-schema rejection before target mutation, and original-file safety tests. User-facing restore UI and conflict UI remain later product work.
- **Validation:**
  - `python3 scripts/harness/check-recovery-policy.py`
  - `cargo test -p silica-storage backup`
  - `cargo test -p silica-storage restore`
  - `cargo test -p silica-storage`
  - Manual QA checklist entry.

### Task 10.6: Public OSS Trust Package

- **Location:** `LICENSE`, `README.md`, `CONTRIBUTING.md`, `SECURITY.md`, `.github/`, `docs/DEPENDENCIES.md`
- **Description:** Complete the public open-source trust package before inviting broad contributors or public beta users.
- **Dependencies:** Task 10.5
- **Design Gate:** [Phase 10 Evidence and Recovery Design](../../superpowers/specs/2026-06-11-phase-10-evidence-recovery-design.md) defines which public claims remain allowed or forbidden after the recovery layer is implemented.
- **Acceptance Criteria:**
  - Final project license is selected.
  - Dependency/license inventory is current.
  - Security policy, contribution guide, issue templates, and PR templates reflect local-first non-destructive scope.
  - Known limitations are explicit.
- **Validation:**
  - `python3 scripts/harness/check-md-links.py`
  - `python3 scripts/harness/check-cargo-deps.py`
- **Status:** Completed on 2026-06-11. Task 10.6.1 selected the MIT License, added the root `LICENSE`, added [ADR 0008](../decisions/adr-0008-project-license.md), and documented current public trust boundaries in [Public Trust](../topics/public-trust.md). Task 10.6.2 completed the public `CONTRIBUTING.md`, `SECURITY.md`, issue templates, PR template public-trust checks, and `scripts/harness/check-public-trust-package.py` static regression check.

## Phase 11: Session, Library, and Metadata Foundation

**Goal:** Make launch, relaunch, browsing, metadata, and selection behavior real and scalable.

**Design Gate:** [Phase 11 Session, Library, and Metadata Design](../../superpowers/specs/2026-06-11-phase-11-session-library-metadata-design.md) defines the app-session boundary, atomic task order, bounded offset query gate, page-scoped thumbnail gate, metadata truth gate, import-error-before-recursive gate, and validation strategy before Phase 11 implementation begins.

### Task 11.1: App Session and Recents Contract

- **Location:** `docs/wiki/topics/catalog.md`, `crates/silica-core`, `apps/desktop/src-tauri`
- **Description:** Define where app-level recent libraries, last library, last mode, selected photo, and layout preferences live.
- **Dependencies:** Phase 10
- **Acceptance Criteria:**
  - App-level state is separate from library catalog state.
  - Missing recent paths degrade to disabled/error states.
  - No fictional recent rows are introduced.
- **Validation:** `python3 scripts/harness/check-md-links.py`
- **Status:** Completed on 2026-06-11. Task 11.1 established the app-session boundary, added core app-session v1 types and JSON read/write helpers, and exposed desktop read/write/reset/inspect command handlers using the Tauri app config path. Real recent recording and Welcome recents UI remain Task 11.2.

### Task 11.2: Persist Real Recent Libraries

- **Location:** `crates/silica-core`, `apps/desktop/src-tauri`, `apps/desktop/static/`
- **Description:** Record recent libraries only after successful create/open and show them on Welcome.
- **Dependencies:** Task 11.1
- **Acceptance Criteria:**
  - Welcome lists only real prior libraries.
  - Missing paths are clearly unavailable.
  - Selecting a recent library opens the same local catalog path.
- **Validation:**
  - `cargo test -p silica-core`
  - UI workflow smoke.
- **Status:** Completed on 2026-06-11. Task 11.2 records recent libraries only after successful create/open, dedupes and caps them in app-session state, and renders real Welcome recents with honest empty and unavailable states. Relaunch restore, selected-photo restore, and layout persistence remain Task 11.3 and Task 11.4.

### Task 11.3: Restore Last Library, Mode, and Selection

- **Location:** `crates/silica-core`, `apps/desktop/src-tauri`, `apps/desktop/static/`
- **Description:** Restore the last valid library and UI mode on relaunch, with safe fallback to Welcome.
- **Dependencies:** Task 11.2
- **Acceptance Criteria:**
  - Relaunch restores selected photo only if it still exists.
  - Missing library or missing photo does not crash the app.
  - Static/demo rows remain absent.
- **Validation:** Connected runtime smoke.
- **Status:** Completed on 2026-06-11. Task 11.3 now resolves launch restore read-only, restores the last valid library shell state, records user-driven selected-photo and mode state in app-session JSON, restores the saved selected photo only when the catalog row still exists, and falls back to Library when selection is missing. Layout preference persistence remains Task 11.4.

### Task 11.4: Persist Workspace Layout Preferences

- **Location:** `apps/desktop/static/`, `crates/silica-core`
- **Description:** Persist sidebar, inspector, filmstrip, thumbnail size, sort, and filter preferences.
- **Dependencies:** Task 11.1
- **Acceptance Criteria:**
  - Preferences restore after relaunch.
  - Reset layout returns to documented defaults.
  - Responsive layout remains stable at compact, desktop, and large widths.
- **Validation:**
  - Static UI check.
  - Visual responsive QA.
- **Status:** Completed on 2026-06-11. Task 11.4 established the core layout default/reset model, wired desktop/sidebar/inspector/filmstrip/thumbnail/sort/filter persistence to app-session state, and extended visual QA with sidebar-collapsed, inspector-collapsed, and reset layout states across `1280x800`, `1440x900`, and `1728x965`.

### Task 11.5: Paged, Sorted, and Filtered Library Query API

- **Location:** `crates/silica-catalog`, `crates/silica-storage`, `crates/silica-core`
- **Description:** Add page-based library queries for large catalogs.
- **Dependencies:** Task 11.4
- **Acceptance Criteria:**
  - Grid does not require loading every photo at once.
  - Product grid thumbnail hydration is page- or viewport-scoped, not whole-catalog eager work.
  - Sort/filter fields use columns and indexes where appropriate.
  - Query uses bounded offset pagination with deterministic tie breakers.
  - Query shape is documented.
- **Validation:** `cargo test -p silica-storage -p silica-core`

### Task 11.6: Virtualized Grid, Keyboard, and Multi-Select

- **Location:** `apps/desktop/static/`, `scripts/harness/`
- **Description:** Implement scalable grid navigation and selection.
- **Dependencies:** Task 11.5
- **Acceptance Criteria:**
  - Grid supports keyboard focus, range selection, and multi-select.
  - No horizontal overflow or control clipping at `1280x800`, `1440x900`, or `1728x965`.
  - Selection state is visually coherent and never fake.
- **Validation:** Visual QA across current mockup viewports.

### Task 11.7: Metadata Extraction and Storage

- **Location:** `crates/silica-catalog`, `crates/silica-storage`, `crates/silica-core`
- **Description:** Extract and persist basic image metadata.
- **Dependencies:** Task 11.5
- **Acceptance Criteria:**
  - Width, height, orientation, capture time, camera, lens, and file metadata are stored when available.
  - Missing metadata is represented honestly.
  - Originals remain unchanged.
- **Validation:**
  - `cargo test -p silica-storage -p silica-core`
  - Original hash checks.

### Task 11.8: Metadata Inspector, Search, and Filters

- **Location:** `apps/desktop/static/`, `crates/silica-catalog`, `crates/silica-storage`, `crates/silica-core`
- **Description:** Wire real metadata into Library and Loupe inspector/search/filter surfaces.
- **Dependencies:** Task 11.7
- **Acceptance Criteria:**
  - Inspector displays real metadata only.
  - Search/filter behavior never implies unavailable metadata exists.
  - Empty and missing states are clear.
- **Validation:** UI workflow smoke.

### Task 11.9: Reviewable Import Errors and Recursive Import

- **Location:** `crates/silica-catalog`, `crates/silica-storage`, `crates/silica-core`, `apps/desktop/static/`
- **Description:** Add reviewable import errors first, then an explicit recursive import option.
- **Dependencies:** Task 11.8
- **Acceptance Criteria:**
  - Structured import errors exist for the current non-recursive path before recursive scanning lands.
  - Unsupported and failed files are visible in an error review surface.
  - Recursive import is user-selected, not silent.
  - Browsing can continue after recoverable import errors.
- **Validation:** Connected runtime smoke.

## Phase 12: Core Image RAW Decode Proof

**Goal:** Prove Core Image RAW support on legal fixtures before showing product RAW pixels.

### Task 12.1: Feature-Gated Core Image RAW Probe

- **Location:** `crates/silica-decode`, `docs/DEPENDENCIES.md`
- **Description:** Add a macOS-only, non-default Core Image RAW probe.
- **Dependencies:** Task 10.1
- **Acceptance Criteria:**
  - Probe is feature-gated and not part of normal local alpha behavior.
  - No LibRaw dependency is added.
  - Any macOS binding dependency is documented.
- **Validation:** `cargo test -p silica-decode --features core-image-raw-probe`

### Task 12.2: RAW Fixture Probe Harness

- **Location:** `crates/silica-decode`, `scripts/harness/`, `docs/wiki/topics/raw-decoding.md`
- **Description:** Run legal RAW fixtures through the probe and record structured results.
- **Dependencies:** Task 12.1
- **Acceptance Criteria:**
  - Results include backend, macOS version, source hash, dimensions, orientation, success/failure, and error category.
  - Originals remain unchanged.
  - Unsupported fixture classes stay explicitly blocked.
- **Validation:** `SILICARAW_RAW_FIXTURE_MANIFEST=... cargo test -p silica-decode --features core-image-raw-probe -- --ignored`

### Task 12.3: Core Image Support Matrix and LibRaw Gate

- **Location:** `docs/wiki/topics/raw-decoding.md`, `docs/wiki/decisions/`
- **Description:** Record which fixture classes graduate from RAW-blocked to Core Image-supported.
- **Dependencies:** Task 12.2
- **Acceptance Criteria:**
  - Support matrix is fixture-backed.
  - LibRaw remains deferred unless a concrete fixture-backed gap is recorded.
  - If LibRaw is revisited, dependency and distribution impact are documented first.
- **Validation:**
  - `python3 scripts/harness/check-md-links.py`
  - `python3 scripts/harness/check-cargo-deps.py`

### Task 12.4: Product RAW Decode API Contract

- **Location:** `crates/silica-decode`, `crates/silica-core`
- **Description:** Define the product decode API for supported RAW fixture classes without wiring UI pixels yet.
- **Dependencies:** Task 12.3
- **Acceptance Criteria:**
  - API returns decoded metadata, dimensions, orientation, decoder backend, and explicit blocked states.
  - Render and catalog layers do not own decoder-specific decisions.
  - Unsupported RAWs still surface clear blocked states.
- **Validation:** `cargo test -p silica-decode -p silica-core`

## Phase 13: Color Pipeline Proof

**Goal:** Prove the Core Image/ColorSync-compatible path before expanding export and preview claims.

### Task 13.1: Tagged Raster Color Probe

- **Location:** `crates/silica-render`, `scripts/harness/`, `docs/wiki/topics/color-management.md`
- **Description:** Probe sRGB, Display P3, and untagged raster fixtures.
- **Dependencies:** Tasks 10.1 and 10.2
- **Acceptance Criteria:**
  - Probe records input profile, working space, output profile, transform path, and fixture hash.
  - Untagged raster behavior is explicit.
  - Color correctness remains a gated claim.
- **Validation:** `SILICARAW_COLOR_FIXTURE_MANIFEST=... cargo test -p silica-render --features color-probe -- --ignored`

### Task 13.2: ICC Embedding Proof for sRGB and Display P3

- **Location:** `crates/silica-export`, `crates/silica-render`, `checklists/`
- **Description:** Prove exported files embed the expected ICC profiles.
- **Dependencies:** Task 13.1
- **Acceptance Criteria:**
  - sRGB remains default.
  - Display P3 is explicit, not accidental.
  - Manual Preview.app/Photos comparison checklist exists.
- **Validation:**
  - Color probe tests.
  - Manual color QA checklist.

### Task 13.3: Wire Color Metadata to Existing Schemas

- **Location:** `crates/silica-edit`, `crates/silica-render`, `crates/silica-export`
- **Description:** Use existing edit graph profile fields for decoder backend, input profile, and working space.
- **Dependencies:** Tasks 12.3 and 13.2
- **Acceptance Criteria:**
  - No hidden schema fields are invented.
  - `profile.input_profile`, `profile.working_space`, and `profile.decoder_backend` remain the contract.
  - Export records retain relevant color metadata.
- **Validation:** `cargo test -p silica-edit -p silica-render -p silica-export`

### Task 13.4: Explicit Export Color Options

- **Location:** `crates/silica-export`, `apps/desktop/static/`, `MockupUI/M007_Export_Dialog.png`
- **Description:** Enable explicit sRGB and Display P3 export choices only after proof.
- **Dependencies:** Task 13.3
- **Acceptance Criteria:**
  - Export dialog makes color space explicit.
  - Unsupported combinations are disabled with clear copy.
  - ICC behavior is tested.
- **Validation:** Export UI smoke and color export tests.

## Phase 14: Product Metal Viewer Bridge

**Goal:** Replace the Spike 001 proof with a product viewer bridge boundary.

### Task 14.1: AppKit/Metal Viewer Bridge Contract

- **Location:** `docs/wiki/topics/metal-rendering.md`, `apps/desktop/src-tauri`
- **Description:** Define reserved layout, lifecycle ownership, event ownership, render request boundaries, and stop conditions.
- **Dependencies:** Phase 10
- **Acceptance Criteria:**
  - Contract explicitly continues Spike 001 Path B.
  - Native viewer region is reserved, not overlaid on arbitrary web UI.
  - Failure conditions preserve the SwiftUI/AppKit fallback rule.
- **Validation:** `python3 scripts/harness/check-md-links.py`

### Task 14.2: Feature-Gated Product Native Viewer Module

- **Location:** `apps/desktop/src-tauri`
- **Description:** Add the product native viewer module separately from `metal_host_spike.rs`.
- **Dependencies:** Task 14.1
- **Acceptance Criteria:**
  - Module is feature-gated.
  - Resize, Retina scale, frame timing, and lifecycle are proven.
  - Web UI controls do not overlap the native viewer.
- **Validation:**
  - `cargo check -p silica-desktop --features native-metal-viewer`
  - `cargo test -p silica-desktop --features native-metal-viewer`

### Task 14.3: Render Request and Texture Lifecycle Boundary

- **Location:** `crates/silica-render`, `apps/desktop/src-tauri`
- **Description:** Define how decoded or raster image data becomes viewer textures.
- **Dependencies:** Task 14.2
- **Acceptance Criteria:**
  - Latest preview request wins during slider interaction.
  - Texture cache is disposable.
  - Render does not write catalog state.
- **Validation:** Render/core tests and viewer lifecycle tests.

### Task 14.4: Viewer Input and Manual QA Checklist

- **Location:** `checklists/`, `scripts/harness/`, `docs/wiki/topics/metal-rendering.md`
- **Description:** Add manual and automated viewer QA for input and layout.
- **Dependencies:** Task 14.3
- **Acceptance Criteria:**
  - QA covers mouse, drag, scroll, magnify, resize, Retina, external display, and UI responsiveness.
  - Screenshots cover `1280x800`, `1440x900`, and `1728x965`.
- **Validation:** Manual checklist and visual QA output.

## Phase 15: RAW, Color, and Metal Vertical Slice

**Goal:** Complete one fixture-backed RAW path from decode to preview, edit, and export.

### Task 15.1: Decoded Image Handoff Contract

- **Location:** `crates/silica-decode`, `crates/silica-render`, `crates/silica-core`
- **Description:** Define the decoded image contract between decoder, renderer, and core.
- **Dependencies:** Tasks 12.4, 13.3, and 14.3
- **Acceptance Criteria:**
  - Contract includes dimensions, orientation, source fingerprint, input profile, working space, decoder backend, and cache identity.
  - Render does not own catalog state.
- **Validation:** `cargo test -p silica-decode -p silica-render -p silica-core`

### Task 15.2: Core Image RAW Preview in Native Viewer

- **Location:** `crates/silica-core`, `crates/silica-render`, `apps/desktop/src-tauri`, `apps/desktop/static/`
- **Description:** Show fixture-proven RAW classes as real pixels in the native viewer.
- **Dependencies:** Task 15.1
- **Acceptance Criteria:**
  - Supported RAWs show preview pixels.
  - Unsupported RAWs remain clearly blocked.
  - Disposable previews live under library cache paths.
  - Originals remain unchanged.
- **Validation:** Runtime smoke with RAW fixture manifest and original hash checks.

### Task 15.3: Exposure and Contrast on Metal Preview Path

- **Location:** `crates/silica-render`, `crates/silica-core`, `apps/desktop/static/`
- **Description:** Move exposure/contrast preview interaction to the Metal preview path.
- **Dependencies:** Task 15.2
- **Acceptance Criteria:**
  - Develop sliders update the Metal preview.
  - No DB write happens per slider tick.
  - Commit persists the edit graph once.
- **Validation:**
  - `cargo test --workspace`
  - No-draft-write test.
  - Viewer performance checklist.

### Task 15.4: RAW-Derived JPEG sRGB Export with ICC

- **Location:** `crates/silica-export`, `crates/silica-core`, `apps/desktop/src-tauri`
- **Description:** Export full-resolution RAW-derived JPEG sRGB with ICC embedding.
- **Dependencies:** Tasks 13.2 and 15.3
- **Acceptance Criteria:**
  - Export path is separate from preview.
  - sRGB ICC is embedded.
  - Decoder/color metadata is recorded.
  - Original RAW files remain unchanged.
- **Validation:**
  - Export inspection.
  - Original hash protection.
  - `scripts/harness/check.sh`

### Task 15.5: Installed-App RAW Vertical Smoke

- **Location:** `scripts/harness/`, `checklists/LOCAL_DMG_INSTALL_CHECKLIST.md`
- **Description:** Extend installed-app QA to cover the fixture-backed RAW vertical slice.
- **Dependencies:** Task 15.4
- **Acceptance Criteria:**
  - Installed app imports supported RAW fixtures, shows preview, edits exposure/contrast, exports JPEG sRGB, and preserves originals.
  - Unsupported RAWs remain blocked and reviewable.
- **Validation:** Clean-Mac install QA record.

## Phase 16: Undo, History, and Action Trust

**Goal:** Protect non-destructive editing before adding more Develop controls.

### Task 16.1: Undo, History, and Action Semantics

- **Location:** `docs/wiki/topics/edit-graph.md`, `docs/wiki/topics/catalog.md`
- **Description:** Define which actions are undoable, which are logged, and which are never silently reversed.
- **Dependencies:** Phase 15
- **Acceptance Criteria:**
  - Edit commits and flags are undoable.
  - Exports are logged but not undone by deleting files.
  - Cache clearing is logged as disposable-data removal.
- **Validation:** `python3 scripts/harness/check-md-links.py`

### Task 16.2: Persist Edit History Snapshots

- **Location:** `crates/silica-storage`, `crates/silica-core`
- **Description:** Store edit history checkpoints on commit.
- **Dependencies:** Task 16.1
- **Acceptance Criteria:**
  - Active edit state and history survive reopen.
  - Slider draft updates do not create history rows.
  - History records reference validated edit graphs.
- **Validation:** `cargo test -p silica-storage -p silica-core`

### Task 16.3: Undo and Redo Core Commands

- **Location:** `crates/silica-core`, `apps/desktop/src-tauri`
- **Description:** Add undo/redo commands for edit and culling actions.
- **Dependencies:** Task 16.2
- **Acceptance Criteria:**
  - Cmd+Z and Shift+Cmd+Z restore valid prior states.
  - Undo/redo disabled states are correct.
  - Exports are not deleted by undo.
- **Validation:** Core and Tauri command tests.

### Task 16.4: Develop History Panel

- **Location:** `apps/desktop/static/`, `MockupUI/M005_Develop_default.png`
- **Description:** Add a real Develop history panel backed by catalog history.
- **Dependencies:** Task 16.3
- **Acceptance Criteria:**
  - Panel lists real checkpoints only.
  - Keyboard focus and disabled states are coherent.
  - Selecting a checkpoint previews and commits according to the documented semantics.
- **Validation:** UI smoke and visual QA.

### Task 16.5: Action Log Storage API

- **Location:** `crates/silica-storage`, `crates/silica-core`
- **Description:** Add an action log API for sensitive actions, future permissions, plugins, MCP, exports, and cache maintenance.
- **Dependencies:** Task 16.3
- **Acceptance Criteria:**
  - Log entries are written through Core APIs, not direct SQLite from extension layers.
  - Action log records actor, action, target, timestamp, and side-effect category.
  - Original file mutation remains forbidden.
- **Validation:** `cargo test -p silica-storage -p silica-core`

## Phase 17: Develop P0 Expansion

**Goal:** Complete the documented P0 Develop baseline for supported image paths.

### Task 17.1: Basic Edit Graph Mutators

- **Location:** `crates/silica-edit`
- **Description:** Add mutators for white balance, temperature, tint, highlights, shadows, whites, blacks, vibrance, and saturation.
- **Dependencies:** Phase 16
- **Acceptance Criteria:**
  - Values obey `schemas/edit_graph.schema.json`.
  - Invalid ranges are rejected.
  - Serialization round-trips through the existing schema.
- **Validation:** `cargo test -p silica-edit`

### Task 17.2: Visible Preview for Basic Controls

- **Location:** `crates/silica-render`, `crates/silica-core`, `apps/desktop/static/`
- **Description:** Render visible previews for enabled P0 controls on supported sources.
- **Dependencies:** Task 17.1
- **Acceptance Criteria:**
  - Supported JPEG/JPG and supported RAW paths visibly update.
  - Unsupported sources remain blocked.
  - Draft updates do not write catalog state.
- **Validation:** Render/core tests.

### Task 17.3: Real Histogram Cache and Display

- **Location:** `crates/silica-render`, `crates/silica-storage`, `apps/desktop/static/`
- **Description:** Generate and cache histograms based on image pixels.
- **Dependencies:** Task 17.2
- **Acceptance Criteria:**
  - Histogram cache is disposable.
  - Histogram reflects current preview state where supported.
  - Missing/blocked histograms are represented honestly.
- **Validation:** Core/storage tests and visual QA.

### Task 17.4: Reset, Before/After, and Basic Presets

- **Location:** `crates/silica-edit`, `crates/silica-core`, `apps/desktop/static/`
- **Description:** Add reset actions, before/after view, and built-in basic presets.
- **Dependencies:** Task 17.2
- **Acceptance Criteria:**
  - Reset and preset apply are undoable checkpoints.
  - Before/after does not create edit history by itself.
  - Built-in presets use validated edit graph changes.
- **Validation:** UI smoke and history tests.

### Task 17.5: Develop P0 Visual QA

- **Location:** `scripts/harness/`, `docs/wiki/topics/ui-visual-responsive-qa.md`
- **Description:** Expand responsive visual QA for the full P0 Develop screen.
- **Dependencies:** Tasks 17.3 and 17.4
- **Acceptance Criteria:**
  - Develop fits compact, desktop, and large mockup widths.
  - Controls use consistent typography, spacing, and tokenized styling.
  - No text or controls overlap.
- **Validation:** `python3 scripts/harness/run-final-visual-qa.py`

## Phase 18: Professional Editing Baseline

**Goal:** Add P1 professional editing tools in controlled vertical slices.

### Task 18.1: Tone Curve Panel

- **Location:** `crates/silica-edit`, `crates/silica-render`, `apps/desktop/static/`
- **Description:** Add tone curve persistence, preview, and UI.
- **Dependencies:** Phase 17
- **Acceptance Criteria:**
  - Curve values validate and round-trip.
  - Preview updates where renderer supports it.
  - Commit creates an undo checkpoint.
- **Validation:** Edit/render/core tests.

### Task 18.2: HSL and Color Mixer Panel

- **Location:** `crates/silica-edit`, `crates/silica-render`, `apps/desktop/static/`
- **Description:** Add per-channel hue, saturation, and luminance controls.
- **Dependencies:** Task 18.1
- **Acceptance Criteria:**
  - Per-channel values validate.
  - UI keeps controls compact and consistent.
  - Export honors committed state where supported.
- **Validation:** Edit/render/export tests.

### Task 18.3: Detail Baseline

- **Location:** `crates/silica-edit`, `crates/silica-render`, `apps/desktop/static/`
- **Description:** Add sharpening and noise reduction baseline controls.
- **Dependencies:** Task 18.1
- **Acceptance Criteria:**
  - Controls persist through edit graph.
  - Disabled preview states are explicit if a renderer path is not ready.
  - No MLX denoise structure is invented.
- **Validation:** Edit/UI tests.

### Task 18.4: Lens, Geometry, Crop, and Rotate Baseline

- **Location:** `crates/silica-edit`, `crates/silica-render`, `crates/silica-export`, `apps/desktop/static/`
- **Description:** Add non-destructive crop, rotate, and baseline geometry handling.
- **Dependencies:** Task 18.1
- **Acceptance Criteria:**
  - Geometry remains non-destructive.
  - Preview and export honor committed geometry where supported.
  - Original orientation metadata is not overwritten.
- **Validation:** Core/export tests and original hash checks.

### Task 18.5: Copy/Paste Edits and Batch Sync

- **Location:** `crates/silica-core`, `crates/silica-edit`, `apps/desktop/static/`
- **Description:** Apply validated edit graph subsets across selected photos.
- **Dependencies:** Tasks 16.3 and 18.1
- **Acceptance Criteria:**
  - Batch sync records history per affected photo.
  - User chooses which edit sections sync.
  - Originals remain unchanged.
- **Validation:** Batch smoke and original hash checks.

## Phase 19: Masks and Local Mask Pipeline

**Goal:** Add mask editing before AI-generated masks.

### Task 19.1: Mask Schema and Edit Graph Audit

- **Location:** `schemas/edit_graph.schema.json`, `crates/silica-edit`, `docs/wiki/topics/edit-graph.md`
- **Description:** Audit existing mask fields before implementing mask UI or render behavior.
- **Dependencies:** Phase 18
- **Acceptance Criteria:**
  - No hidden mask format is invented.
  - Any schema expansion is explicit and versioned if breaking.
  - Manual masks and future AI masks have separate provenance.
- **Validation:** `cargo test -p silica-edit`

### Task 19.2: Linear and Radial Manual Masks

- **Location:** `crates/silica-edit`, `crates/silica-render`, `apps/desktop/static/`, `MockupUI/M006_Develop_mask_active.png`
- **Description:** Add simple manual gradient masks with preview support.
- **Dependencies:** Task 19.1
- **Acceptance Criteria:**
  - Mask geometry persists in the edit graph.
  - Mask preview is visible where render path supports it.
  - Unsupported mask rendering states are explicit.
- **Validation:** Edit/render/UI tests.

### Task 19.3: Brush Mask Storage and Rasterization

- **Location:** `crates/silica-edit`, `crates/silica-render`, `crates/silica-storage`
- **Description:** Add brush mask data and rasterization behavior.
- **Dependencies:** Task 19.2
- **Acceptance Criteria:**
  - Brush data is non-destructive.
  - Large brush data does not bloat per-slider writes.
  - Cache artifacts are disposable.
- **Validation:** Storage/render tests.

### Task 19.4: Mask Compositing in Preview and Export

- **Location:** `crates/silica-render`, `crates/silica-export`
- **Description:** Apply committed masks consistently in preview and export.
- **Dependencies:** Task 19.3
- **Acceptance Criteria:**
  - Preview and export agree within documented tolerance.
  - Masked edits are recorded in history.
  - Unsupported export combinations are blocked.
- **Validation:** Render/export fixture tests.

### Task 19.5: Mask Editor Visual QA

- **Location:** `apps/desktop/static/`, `scripts/harness/`, `MockupUI/M006_Develop_mask_active.png`
- **Description:** Implement and verify the mask editor screen state.
- **Dependencies:** Task 19.4
- **Acceptance Criteria:**
  - Mask controls match the design system.
  - Compact and large Develop mockups remain coherent.
  - No control overlap or hidden active state.
- **Validation:** Visual QA.

## Phase 20: Export and Delivery Expansion

**Goal:** Make export professional without silently wrong color or metadata behavior.

### Task 20.1: Export Settings Model and Presets

- **Location:** `crates/silica-export`, `crates/silica-storage`, `apps/desktop/static/`
- **Description:** Add persistent export settings and presets separate from edit graph state.
- **Dependencies:** Phase 17
- **Acceptance Criteria:**
  - Export settings do not modify develop edits.
  - Presets are editable and persisted.
  - Defaults remain conservative.
- **Validation:** Storage/core/export tests.

### Task 20.2: PNG and TIFF Export

- **Location:** `crates/silica-export`, `crates/silica-core`, `apps/desktop/static/`
- **Description:** Add PNG and TIFF export after codec behavior is tested.
- **Dependencies:** Task 20.1
- **Acceptance Criteria:**
  - Outputs are separate files.
  - Overwrite guard remains active.
  - Export records are stored in the catalog.
- **Validation:** `cargo test -p silica-export -p silica-core`

### Task 20.3: Export Metadata Policy

- **Location:** `crates/silica-export`, `crates/silica-core`, `apps/desktop/static/`
- **Description:** Add preserve metadata, remove GPS, and remove all metadata options.
- **Dependencies:** Tasks 11.7 and 20.2
- **Acceptance Criteria:**
  - Metadata policy is explicit in UI.
  - Exported metadata behavior is tested.
  - Originals are not rewritten.
- **Validation:** Export metadata tests.

### Task 20.4: Batch Export Progress and Recent Exports

- **Location:** `crates/silica-core`, `apps/desktop/src-tauri`, `apps/desktop/static/`
- **Description:** Add multi-photo export progress, failures, and recent export records.
- **Dependencies:** Task 20.1
- **Acceptance Criteria:**
  - Progress reflects real selected photos.
  - Failures are reviewable.
  - Recent exports do not imply files still exist if missing.
- **Validation:** Connected runtime smoke.

### Task 20.5: Display P3 Export Enablement

- **Location:** `crates/silica-export`, `apps/desktop/static/`
- **Description:** Enable Display P3 export only after color proof.
- **Dependencies:** Tasks 13.4 and 20.2
- **Acceptance Criteria:**
  - P3 export embeds expected ICC.
  - UI distinguishes sRGB default from P3 explicit choice.
  - Color QA checklist is updated.
- **Validation:** Color/export fixture suite.

## Phase 21: Preferences and App Settings

**Goal:** Replace minimal maintenance UI with a complete preferences surface.

### Task 21.1: Preferences Information Architecture

- **Location:** `docs/wiki/topics/ui-mockups.md`, `apps/desktop/static/`, `MockupUI/M008_Preferences_Appearance.png`
- **Description:** Define preferences sections for Appearance, Library, Cache, Color, Export, and Advanced.
- **Dependencies:** Phase 11
- **Acceptance Criteria:**
  - Preferences are discoverable and compact.
  - Advanced agent access remains off by default.
  - No unimplemented setting appears enabled.
- **Validation:** Static UI and visual QA.

### Task 21.2: Appearance Preferences

- **Location:** `apps/desktop/static/`, `crates/silica-core`
- **Description:** Persist theme, density, and UI scale preferences where supported.
- **Dependencies:** Task 21.1
- **Acceptance Criteria:**
  - Appearance settings persist and reset.
  - Text remains legible and unclipped.
  - Design tokens remain the styling source.
- **Validation:** Visual QA.

### Task 21.3: Library and Cache Preferences

- **Location:** `crates/silica-core`, `apps/desktop/static/`
- **Description:** Add cache size/status, cache clear, and library storage preferences.
- **Dependencies:** Task 21.1
- **Acceptance Criteria:**
  - Cache clear remains limited to disposable cache directories.
  - Preferences never expose dangerous original-file operations.
  - Status reflects real cache paths and sizes.
- **Validation:** Cache clear smoke and original hash checks.

### Task 21.4: Color and Export Defaults

- **Location:** `crates/silica-core`, `crates/silica-export`, `apps/desktop/static/`
- **Description:** Persist default export format, quality, and color space choices.
- **Dependencies:** Tasks 13.4 and 20.1
- **Acceptance Criteria:**
  - Defaults are validated against supported formats.
  - Unsupported defaults cannot be saved.
  - Export dialog reflects current defaults.
- **Validation:** Export UI smoke.

### Task 21.5: Advanced Agent Access Preferences

- **Location:** `apps/desktop/static/`, `crates/silica-core`, `docs/wiki/topics/plugins-and-mcp.md`
- **Description:** Add disabled-by-default advanced preferences for future plugin/MCP access.
- **Dependencies:** Phase 23
- **Acceptance Criteria:**
  - Agent access is off by default.
  - UI explains permissions and side effects when later enabled.
  - No MCP server or plugin runtime is started by this task.
- **Validation:** Scope guardrails and static UI check.

## Phase 22: Performance, Migration, and Visual Hardening

**Goal:** Keep the larger editor stable and coherent.

### Task 22.1: Expanded Visual QA Surface Set

- **Location:** `scripts/harness/run-final-visual-qa.py`, `docs/wiki/topics/ui-visual-responsive-qa.md`
- **Description:** Add Library filters, metadata, History, expanded Develop, Mask Editor, Preferences, and Export surfaces to visual QA.
- **Dependencies:** Phases 17 through 21 as relevant
- **Acceptance Criteria:**
  - Screens are checked at compact, desktop, and large widths.
  - Typography, spacing, and controls remain consistent.
  - No overlapping or clipped text.
- **Validation:** `python3 scripts/harness/run-final-visual-qa.py`

### Task 22.2: Library Scale Benchmarks

- **Location:** `scripts/harness/`, `crates/silica-storage`, `docs/wiki/topics/catalog.md`
- **Description:** Add 1k, 10k, and 50k catalog benchmark reports.
- **Dependencies:** Task 11.5
- **Acceptance Criteria:**
  - Reports include machine metadata and dataset shape.
  - Query and render-adjacent timings are recorded.
  - Results are not marketed as universal performance guarantees.
- **Validation:** Benchmark report.

### Task 22.3: Migration and Backup Failure Tests

- **Location:** `crates/silica-storage`, `scripts/harness/`
- **Description:** Test migration, restore, missing files, conflict states, and partial failure recovery.
- **Dependencies:** Tasks 10.5, 16.2, and 20.1
- **Acceptance Criteria:**
  - Edit/history/export data survives migrations.
  - Corrupt or missing states produce recoverable errors.
  - Disposable caches can always be regenerated.
- **Validation:** Storage integration tests.

### Task 22.4: RAW and Metal Performance Profiling

- **Location:** `crates/silica-render`, `apps/desktop/src-tauri`, `scripts/harness/`
- **Description:** Profile decode, preview, slider interaction, and export paths.
- **Dependencies:** Phase 15
- **Acceptance Criteria:**
  - Reports separate decode time, render time, UI latency, and export time.
  - Memory pressure behavior is recorded.
  - Performance regressions are actionable.
- **Validation:** Profiling report and viewer performance checklist.

### Task 22.5: Manual Photographer QA Checklist

- **Location:** `checklists/`
- **Description:** Add a real workflow checklist for culling, metadata, undo, Develop, masks, export, and responsiveness.
- **Dependencies:** Phases 17 through 21
- **Acceptance Criteria:**
  - Checklist uses licensed or user-provided local test assets.
  - Data safety and color/export checks are included.
  - Known limitations are recorded.
- **Validation:** Completed checklist record.

## Phase 23: Permission and Audit Foundation

**Goal:** Build the permission layer before MLX, plugins, or MCP.

### Task 23.1: Core Permission Enum and Policy

- **Location:** `crates/silica-core`, `crates/silica-plugin`, `crates/silica-mcp`, `docs/wiki/topics/plugins-and-mcp.md`
- **Description:** Define default-deny permissions for future extension actors.
- **Dependencies:** Phase 16
- **Acceptance Criteria:**
  - Permissions cover metadata, edit suggestions, export, filesystem, AI results, and MCP modes.
  - No raw SQL permission exists.
  - Original mutation remains forbidden.
- **Validation:** `cargo test -p silica-core -p silica-plugin -p silica-mcp`

### Task 23.2: Permission Prompt UI Contract

- **Location:** `apps/desktop/static/`, `docs/wiki/topics/plugins-and-mcp.md`
- **Description:** Define how permission prompts present actor, permission, side effects, confirmation, and undo availability.
- **Dependencies:** Task 23.1
- **Acceptance Criteria:**
  - Denial is handled cleanly.
  - Prompts are explicit and not promotional.
  - Dangerous permissions remain unavailable unless a future ADR approves them.
- **Validation:** UI smoke and visual QA.

### Task 23.3: Permissioned Action Log Integration

- **Location:** `crates/silica-core`, `crates/silica-storage`
- **Description:** Connect permission decisions and sensitive actions to the action log.
- **Dependencies:** Tasks 16.5 and 23.1
- **Acceptance Criteria:**
  - Permission grants, denials, plugin applies, AI approvals, MCP reads, and export attempts are logged.
  - Extension layers cannot bypass Core APIs.
- **Validation:** Permission bypass tests.

## Phase 24: MLX and AI Preview

**Goal:** Add local AI as an optional enhancement after editor trust gates.

### Task 24.1: MLX Runtime Spike

- **Location:** `crates/silica-mlx`, `docs/wiki/topics/mlx.md`, `docs/DEPENDENCIES.md`
- **Description:** Revisit MLX runtime feasibility, binding choice, memory behavior, cancellation, and packaging.
- **Dependencies:** Phase 23 and Phase 22 trust checks
- **Acceptance Criteria:**
  - Spike records runtime choice and no-model behavior.
  - Model packaging and license impact are documented.
  - No model is bundled without manifest.
- **Validation:** Spike report and `scripts/harness/check.sh`

### Task 24.2: Model Manifest Validation

- **Location:** `crates/silica-mlx`, `schemas/model_manifest.schema.json`
- **Description:** Validate model manifests before any model can be enabled.
- **Dependencies:** Task 24.1
- **Acceptance Criteria:**
  - Missing license, source, hash, preprocessing, or output metadata is rejected.
  - Hash checks are deterministic.
  - Models remain optional.
- **Validation:** `cargo test -p silica-mlx`

### Task 24.3: AI Result Store and Read Path

- **Location:** `crates/silica-storage`, `crates/silica-core`, `crates/silica-mlx`
- **Description:** Store AI outputs separately from edit graph and catalog flags.
- **Dependencies:** Tasks 23.3 and 24.2
- **Acceptance Criteria:**
  - AI results are unapproved by default.
  - AI output cannot directly mutate edit graph.
  - Results are local-only and permissioned.
- **Validation:** `cargo test -p silica-storage -p silica-core -p silica-mlx`

### Task 24.4: First Non-Mutating AI Review Feature

- **Location:** `crates/silica-mlx`, `crates/silica-core`, `apps/desktop/static/`, `MockupUI/M010_AI_Review.png`
- **Description:** Add a first AI review feature such as blur or quality review before masks or auto tone.
- **Dependencies:** Task 24.3
- **Acceptance Criteria:**
  - App works when model is missing.
  - Output is presented as review information, not an edit.
  - Original files remain unchanged.
- **Validation:** Model manifest tests, original hash tests, and visual QA.

### Task 24.5: Explicit AI Suggestion Approval

- **Location:** `crates/silica-core`, `crates/silica-edit`, `apps/desktop/static/`
- **Description:** Convert approved AI suggestions into edit graph changes or mask data only through explicit user approval.
- **Dependencies:** Tasks 19.4 and 24.4
- **Acceptance Criteria:**
  - Approval creates an undoable checkpoint.
  - Rejection leaves edit state unchanged.
  - Suggestion provenance is recorded.
- **Validation:** Core/edit/history tests.

## Phase 25: Plugin Foundation

**Goal:** Add a safe declarative plugin path before any executable plugin model.

### Task 25.1: Plugin Manifest Validation

- **Location:** `crates/silica-plugin`, `schemas/plugin_manifest.schema.json`
- **Description:** Validate plugin manifests before enabling plugins.
- **Dependencies:** Phase 23
- **Acceptance Criteria:**
  - Missing license, minimum app version, or permissions are rejected.
  - Plugins are disabled by default.
  - Plugin manifests cannot request raw SQL.
- **Validation:** `cargo test -p silica-plugin`

### Task 25.2: Declarative Preset Plugin

- **Location:** `crates/silica-plugin`, `crates/silica-core`, `crates/silica-edit`
- **Description:** Support a data-only preset plugin format.
- **Dependencies:** Task 25.1
- **Acceptance Criteria:**
  - No arbitrary executable plugin code runs.
  - Preset data goes through edit graph validation.
  - Applying a preset requires explicit approval and creates history.
- **Validation:** `cargo test -p silica-plugin -p silica-core -p silica-edit`

### Task 25.3: Plugin Permission Review and Action Log

- **Location:** `crates/silica-plugin`, `crates/silica-core`, `apps/desktop/static/`
- **Description:** Add plugin enable/apply review and action logging.
- **Dependencies:** Tasks 23.3 and 25.2
- **Acceptance Criteria:**
  - Enabling and applying plugin data is logged.
  - Plugin code cannot directly access SQLite.
  - Permission denial is handled cleanly.
- **Validation:**
  - Permission bypass tests.
  - `scripts/harness/check-scope-guardrails.sh`

## Phase 26: MCP Read-Only First

**Goal:** Add MCP only as a disabled-by-default, read-only, permissioned interface first.

### Task 26.1: MCP Transport and Session ADR

- **Location:** `docs/wiki/decisions/`, `docs/wiki/topics/plugins-and-mcp.md`
- **Description:** Decide MCP transport, session lifetime, permission posture, and default disabled state.
- **Dependencies:** Phase 23
- **Acceptance Criteria:**
  - MCP is off by default.
  - No dangerous mode or permission self-escalation exists.
  - Mutating tools are explicitly out of scope.
- **Validation:** `python3 scripts/harness/check-md-links.py`

### Task 26.2: Read-Only MCP Tool Manifests

- **Location:** `crates/silica-mcp`, `schemas/mcp_tool_manifest.schema.json`
- **Description:** Define read-only tool manifests for selection, metadata, catalog listing, and export record inspection.
- **Dependencies:** Task 26.1
- **Acceptance Criteria:**
  - Side effects are empty.
  - Confirmation is false only for read-only tools.
  - Tool manifests validate against schema.
- **Validation:** `cargo test -p silica-mcp`

### Task 26.3: Read-Only MCP Adapter Through Core APIs

- **Location:** `crates/silica-mcp`, `crates/silica-core`
- **Description:** Implement read-only MCP adapter calls through Core APIs.
- **Dependencies:** Task 26.2
- **Acceptance Criteria:**
  - MCP cannot access storage directly.
  - Mutation and export tools are absent.
  - Reads are action-logged where policy requires it.
- **Validation:**
  - `cargo test -p silica-mcp -p silica-core`
  - Scope guardrails.

## Phase 27: Public Beta Gate

**Goal:** Decide whether SilicaRAW is ready for public beta users.

### Task 27.1: Public Beta Readiness Audit

- **Location:** `docs/wiki/roadmaps/`, `checklists/`
- **Description:** Audit P0/P1 stability, data safety, color/export evidence, OSS docs, and signed/notarized release path.
- **Dependencies:** Phases 10 through 26 as scoped for beta
- **Acceptance Criteria:**
  - No known data-loss bugs.
  - Final license selected.
  - Dependency license inventory complete.
  - Sample asset license manifest complete.
  - Model license manifest complete if models ship.
  - README and release notes are honest about limitations.
- **Validation:**
  - `scripts/harness/check.sh`
  - Clean-Mac install QA.
  - Color/export fixture suite.

### Task 27.2: Public Beta Release Candidate

- **Location:** GitHub Releases, release workflow, `checklists/`
- **Description:** Produce and verify a beta release candidate.
- **Dependencies:** Task 27.1
- **Acceptance Criteria:**
  - Signed and notarized DMG is available.
  - Checksums are published.
  - Gatekeeper accepts the downloaded artifact.
  - Local workflow and beta-scoped product workflow complete.
- **Validation:**
  - `spctl --assess --type execute --verbose /Applications/SilicaRAW.app`
  - `spctl --assess --type open --verbose SilicaRAW.dmg`
  - `shasum -a 256 SilicaRAW.dmg`

## Phase 28: v1.0 Stable Gate

**Goal:** Ship a credible v1.0 RAW editor baseline.

### Task 28.1: v1.0 Scope Freeze

- **Location:** `docs/wiki/roadmaps/`, GitHub milestones
- **Description:** Freeze the v1.0 feature set and push incomplete work to post-v1 milestones.
- **Dependencies:** Public beta feedback
- **Acceptance Criteria:**
  - P0/P1 editor baseline is explicit.
  - Deferred features are documented without hidden commitments.
  - All release-blocking bugs are tracked.
- **Validation:** Milestone review.

### Task 28.2: v1.0 Stability Matrix

- **Location:** `checklists/`, `scripts/harness/`
- **Description:** Run the final stability matrix for data safety, migration, color, export, RAW support, viewer, and install.
- **Dependencies:** Task 28.1
- **Acceptance Criteria:**
  - Original hash suite passes.
  - Migration restore matrix passes.
  - Color/export fixture suite passes.
  - Clean-Mac install passes.
  - Extension layers are off by default unless explicitly shipped.
- **Validation:** Full harness and manual QA records.

### Task 28.3: v1.0 Release

- **Location:** GitHub Releases, release workflow, docs
- **Description:** Publish the v1.0 release.
- **Dependencies:** Task 28.2
- **Acceptance Criteria:**
  - Signed, notarized, stapled artifact is published.
  - Checksums and release notes are published.
  - Known issues, supported formats, and privacy posture are clear.
- **Validation:** Downloaded artifact verification and release checklist.

## Parallelization Map

- Phase 10 should complete before product breadth begins.
- Phase 11 can start after Phase 10 and can proceed while RAW/color/Metal proofs are being prepared.
- Phases 12 and 13 can proceed in parallel after fixture manifests and tolerance policy exist.
- Phase 14 can proceed in parallel with Phases 12 and 13 after the bridge contract is accepted.
- Phase 15 waits for RAW, color, and Metal gates.
- Phase 16 should precede most advanced Develop work.
- Phases 17 through 21 can proceed in vertical slices once their dependencies are satisfied.
- Phase 23 must precede MLX, plugin, and MCP runtime work.
- Phase 24, Phase 25, and Phase 26 should remain optional and off by default until the editor core is trustworthy.

## Links

- [Local DMG Distribution Plan](local-dmg-distribution-plan.md)
- [Roadmap Overview](../overview/roadmap.md)
- [RAW Decoding](../topics/raw-decoding.md)
- [Metal Rendering](../topics/metal-rendering.md)
- [Color Management](../topics/color-management.md)
- [Catalog](../topics/catalog.md)
- [Edit Graph](../topics/edit-graph.md)
- [MLX](../topics/mlx.md)
- [Plugins and MCP](../topics/plugins-and-mcp.md)
- [Architecture Patch](../../20_v1_1_Architecture_Patch.md)
- [Schema Reference](../../19_Schema_Reference.md)
- [Edit Graph Schema](../../../schemas/edit_graph.schema.json)

## Notes for LLM Agents

Pick the next task by dependency order. Do not jump to MLX, plugins, MCP, masks, or broad RAW support because this roadmap mentions them. A task name is permission only for that task's explicit scope.

When implementing a task from this roadmap:

- Read the linked topic/spec files first.
- Keep the PR small.
- Update the relevant wiki topic and log entry.
- Run the smallest useful checks plus `scripts/harness/check.sh` before claiming completion.
