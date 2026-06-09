---
title: Local DMG Distribution Plan
status: active
audience: all
updated: 2026-06-09
source_of_truth: docs/16_Release_Distribution_Plan.md
---

# Local DMG Distribution Plan

## Summary

The local distribution goal is a GitHub-hosted macOS DMG that a user can download, install, and run locally.

On macOS, the actual application is a `.app` bundle. A `.dmg` is the distribution container that usually presents the `.app` for drag-and-drop installation into `/Applications`.

For a smooth download-and-run experience outside the Mac App Store, the target artifact is:

```txt
Signed and notarized DMG
  containing a signed SilicaRAW.app
  uploaded to a GitHub Release
  with checksums and release notes
```

Unsigned DMGs are acceptable only for early developer testing. They are not the final local distribution target because Gatekeeper may block or warn on downloaded unsigned or unnotarized apps.

## Sources

- Tauri v2 distribution docs: https://v2.tauri.app/distribute
- Tauri v2 DMG docs: https://v2.tauri.app/distribute/dmg
- Tauri v2 macOS signing docs: https://v2.tauri.app/distribute/sign/macos
- Apple macOS distribution: https://developer.apple.com/macos/distribution/
- Apple Developer ID: https://developer.apple.com/developer-id/
- Apple notarization docs: https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution
- GitHub Actions secrets: https://docs.github.com/en/actions/how-tos/security-for-github-actions/security-guides/using-secrets-in-github-actions
- GitHub workflow artifacts: https://docs.github.com/actions/using-workflows/storing-workflow-data-as-artifacts

## Definition of Done

Local distribution is complete when:

- A GitHub Release contains a macOS `.dmg`.
- The DMG contains `SilicaRAW.app`.
- The app launches from `/Applications` on a clean Apple Silicon Mac.
- Gatekeeper accepts the downloaded app without requiring command-line quarantine removal.
- The app can complete the local alpha workflow:
  - launch
  - create or open a local library
  - import a folder
  - show a library grid
  - rate, pick, or reject photos
  - open a preview
  - apply basic exposure/contrast edit
  - persist edit state
  - export JPEG sRGB
- No original photo files are modified.
- Checksums and release notes are published with the release.

## Non-Goals for Local DMG Alpha

- Mac App Store distribution.
- Homebrew Cask distribution.
- Auto-update.
- MLX features.
- Plugin runtime.
- MCP server or automation.
- Full Lightroom compatibility.
- Broad RAW camera support beyond the selected decoder path.
- Public beta polish.

## Phase 0: Release Target and Repository Baseline

**Goal:** Make the target explicit and prepare the repository for release work.

**Demo/Validation:**

- A maintainer can read one page and know exactly what "local distribution" means.
- The repository has a clear release target name and release gates.

### Task 0.1: Record Local Distribution ADR

- **Location:** `docs/wiki/decisions/adr-0002-local-dmg-distribution.md`
- **Description:** Record the decision that local distribution means GitHub Release DMG containing a signed and notarized `.app`.
- **Dependencies:** none
- **Acceptance Criteria:**
  - ADR states that `.app` is the executable bundle and `.dmg` is the distribution container.
  - ADR distinguishes unsigned developer DMG from signed/notarized user DMG.
  - ADR links to release and Apple/Tauri distribution docs.
- **Validation:** Local Markdown link check passes.

### Task 0.2: Initialize Git Repository and GitHub Remote

- **Location:** repository root
- **Description:** This workspace is currently not a git repository. Initialize it and connect it to the intended GitHub repository before CI/release work.
- **Dependencies:** none
- **Acceptance Criteria:**
  - `git status` works.
  - Remote points to the user's GitHub repository.
  - `.gitignore` excludes `target/`, `.serena/`, `.code-review-graph/`, and `.DS_Store`.
- **Validation:**
  - `git rev-parse --is-inside-work-tree`
  - `git remote -v`

### Task 0.3: Define Local Alpha Version

- **Location:** `Cargo.toml`, `README.md`, `docs/wiki/roadmaps/local-dmg-distribution-plan.md`
- **Description:** Set the first local-distribution milestone to `0.1.0-alpha.1` unless maintainers choose a different tag.
- **Dependencies:** Task 0.1
- **Acceptance Criteria:**
  - Version target appears in release docs.
  - Tag naming convention is documented.
- **Validation:** `rg -n "0.1.0-alpha.1|local distribution" README.md docs`

## Phase 1: CI, Formatting, and Release Guardrails

**Goal:** Make every later phase testable before packaging starts.

**Demo/Validation:**

- GitHub Actions runs formatting, build, and tests on every push and PR.

### Task 1.1: Add CI Workflow

- **Location:** `.github/workflows/ci.yml`
- **Description:** Add Rust workspace CI for formatting, build, and tests on macOS.
- **Dependencies:** Task 0.2
- **Acceptance Criteria:**
  - CI runs `cargo fmt --all --check`.
  - CI runs `cargo build --workspace`.
  - CI runs `cargo test --workspace`.
- **Validation:** GitHub Actions CI passes.

### Task 1.2: Add Dependency Guard Check

- **Location:** `.github/workflows/ci.yml`, `scripts/harness/check-cargo-deps.py`
- **Description:** Add a lightweight check that fails when dependencies are added without `docs/DEPENDENCIES.md` changes.
- **Dependencies:** Task 1.1
- **Acceptance Criteria:**
  - Script compares dependency metadata against documented policy.
  - CI runs the script.
- **Validation:** CI passes on current zero-dependency workspace.

### Task 1.3: Add Architecture Scope Check

- **Location:** `scripts/harness/check-scope-guardrails.sh`, `.github/workflows/ci.yml`
- **Description:** Add a conservative guard that scans for early MLX/MCP/plugin/telemetry/cloud additions.
- **Dependencies:** Task 1.1
- **Acceptance Criteria:**
  - CI fails on obvious prohibited strings in product code unless allowlisted.
  - Documentation files are excluded from false-positive failures.
- **Validation:** CI passes on current repository.

## Phase 2: Desktop Shell and Packaging Skeleton

**Goal:** Produce a minimal desktop app bundle without product features.

**Demo/Validation:**

- A developer can run a basic SilicaRAW app locally.
- The app can be bundled as `.app` and unsigned `.dmg` for internal testing.

**Developer Artifact Note:** Phase 2 DMGs are unsigned/ad-hoc and developer-only. They validate packaging mechanics, not user-ready local distribution.

Current Phase 2 commands:

```bash
cd apps/desktop/src-tauri
cargo tauri build --no-bundle
cargo tauri build --bundles app,dmg --ci --no-sign
```

### Task 2.1: Choose App Shell Path for the First Packaging Spike

- **Location:** `docs/wiki/decisions/adr-0003-app-shell-packaging-path.md`
- **Description:** Record whether the first packaging attempt uses Tauri v2 or a native SwiftUI/AppKit shell.
- **Dependencies:** Task 0.1
- **Acceptance Criteria:**
  - Decision preserves the Tauri + Metal fallback rule.
  - Decision does not claim the Metal viewer is solved.
- **Validation:** ADR links to [Architecture Patch](../../20_v1_1_Architecture_Patch.md).

### Task 2.2: Add Minimal Tauri Shell

- **Location:** `apps/desktop`, `apps/desktop/src-tauri`, root workspace files
- **Description:** Replace the placeholder binary with a minimal Tauri app shell only if Task 2.1 selects Tauri for the first packaging spike.
- **Dependencies:** Task 2.1
- **Acceptance Criteria:**
  - App launches a simple local window.
  - No RAW, Metal viewer, MLX, plugin, or MCP behavior is added.
  - Added Tauri dependencies are documented in `docs/DEPENDENCIES.md`.
- **Validation:**
  - `cargo build --workspace`
  - local app launch command succeeds.

### Task 2.3: Configure Bundle Metadata

- **Location:** Tauri config under `apps/desktop/src-tauri/`
- **Description:** Configure product name, bundle identifier, icons placeholder, copyright, and macOS category.
- **Dependencies:** Task 2.2
- **Acceptance Criteria:**
  - Bundle identifier is stable.
  - Bundle metadata is documented.
  - Placeholder icons are clearly marked as temporary.
- **Validation:** `cargo tauri build --no-bundle` succeeds.

### Task 2.4: Build Unsigned Developer DMG

- **Location:** local build output
- **Description:** Use Tauri's bundler to produce a developer-only DMG.
- **Dependencies:** Task 2.3
- **Acceptance Criteria:**
  - `cargo tauri bundle --bundles app,dmg` or equivalent succeeds.
  - Output includes `.app` and `.dmg`.
  - Release notes mark it as unsigned and developer-only.
- **Validation:** Mount DMG locally and launch app on the build Mac.

## Phase 3: Mandatory Feasibility Gates

**Goal:** Resolve the risks that determine whether the app can become a credible RAW editor.

**Demo/Validation:**

- Each spike produces a written result, evidence, and next decision.

### Task 3.1: Spike Tauri + Native Metal Viewer

- **Location:** `docs/spikes/001-tauri-metal-viewer.md`, `apps/desktop`
- **Description:** Verify whether the selected shell can host or coordinate a native Metal-rendered view.
- **Dependencies:** Phase 2
- **Acceptance Criteria:**
  - Metal output appears in the app window.
  - Resize works.
  - Retina scaling works.
  - Mouse and trackpad events map correctly.
  - UI remains responsive.
  - Render timing is available.
  - Metal render loop can be controlled from Rust/Core.
  - Result is recorded as Path A, B, or C.
- **Validation:** Manual spike checklist and screenshot/video evidence.

### Task 3.2: Spike RAW Decoder Path

- **Location:** `docs/spikes/002-raw-decoder.md`, `crates/silica-decode`
- **Description:** Compare Core Image RAW, LibRaw, and hybrid feasibility.
- **Dependencies:** Task 3.1 can run in parallel if isolated.
- **Acceptance Criteria:**
  - Decoder path is selected or explicitly deferred.
  - Dependency license impact is documented.
  - Decoder-dependent features are tagged.
- **Validation:** Spike report with fixture results.

**Status:** Completed on 2026-06-08. Spike 002 selected Core Image RAW primary, deferred LibRaw until fixture evidence proves a gap, and recorded that the repository currently has no legally usable RAW fixtures.

### Task 3.3: Spike Color-Managed Preview and Export

- **Location:** `docs/spikes/003-color-managed-preview-export.md`
- **Description:** Verify basic sRGB and Display P3 assumptions for preview/export.
- **Dependencies:** Task 3.2 preferred
- **Acceptance Criteria:**
  - Working color space recommendation is recorded.
  - Fixture class and machine details are recorded.
  - Known color limitations are documented.
- **Validation:** Fixture-based report.

**Status:** Completed on 2026-06-08 as Path B. Spike 003 selected Core Image/ColorSync-compatible color management first, recommended a linear Display P3-compatible working space, kept sRGB as default export, and recorded that tagged color fixtures are still missing.

### Task 3.4: Spike SQLite Catalog Persistence

- **Location:** `crates/silica-storage`, `crates/silica-catalog`, `docs/spikes/004-sqlite-persistence.md`
- **Description:** Select SQLite binding and migration approach.
- **Dependencies:** Phase 1
- **Acceptance Criteria:**
  - SQLite dependency documented.
  - Migrations can create and upgrade an empty catalog.
  - Required indexes are included.
- **Validation:** Migration unit/integration tests pass.

**Status:** Completed on 2026-06-08. Spike 004 selected `rusqlite` with bundled SQLite and embedded SQL migrations. `silica-storage` can create and upgrade an empty catalog through schema version 2, and tests verify required indexes plus foreign key enforcement.

### Task 3.5: Decide MLX Deferral for Local DMG Alpha

- **Location:** `docs/wiki/decisions/adr-0005-mlx-deferral-for-local-alpha.md`
- **Description:** Explicitly defer MLX from local DMG alpha unless maintainers decide otherwise.
- **Dependencies:** none
- **Acceptance Criteria:**
  - Decision states MLX is not required for local alpha.
  - `silica-mlx` remains a boundary crate only.
- **Validation:** CI dependency guard confirms no MLX dependency.

**Status:** Completed on 2026-06-08. ADR 0005 defers MLX from local alpha, and `silica-mlx` remains a dependency-free boundary crate.

## Phase 4: Local Library and Data Safety MVP

**Goal:** Make the installed app useful without risking originals.

**Demo/Validation:**

- User can create/open a local library, import a folder, and persist catalog state.

### Task 4.1: Implement Catalog Migration Foundation

- **Location:** `crates/silica-storage`, `crates/silica-catalog`
- **Description:** Add migration runner and initial catalog schema.
- **Dependencies:** Task 3.4
- **Acceptance Criteria:**
  - Initial tables from `docs/10_Data_Model_and_Storage_Specification.md` are created as needed for alpha.
  - Required initial indexes exist.
  - Migration table records applied migrations.
- **Validation:** Migration tests pass on empty and existing DB.

**Status:** Completed on 2026-06-08. Spike 004 added the migration runner, initial schema, required indexes, and migration tests. Phase 4.1 then moved the domain-facing alpha schema contract into `silica-catalog` and made `silica-storage` verify migrations against that contract.

### Task 4.2: Implement Library Create/Open

- **Location:** `crates/silica-core`, `crates/silica-storage`, `apps/desktop`
- **Description:** Add local library create/open command and minimal UI entry point.
- **Dependencies:** Task 4.1
- **Acceptance Criteria:**
  - User can create a library folder.
  - App can reopen the same library.
  - No original photo directory is modified.
- **Validation:** Integration test and manual restart test.

**Status:** Completed on 2026-06-08. `silica-storage` can create/reopen a local library folder with `catalog.db` and support directories, `silica-core` exposes create/open APIs, and the Tauri shell has minimal create/open commands plus a path-based entry point. Tests verify reopen behavior and sibling original-directory preservation.

### Task 4.3: Implement Folder Import Scanner

- **Location:** `crates/silica-catalog`, `crates/silica-storage`
- **Description:** Scan a selected folder and record photo candidates by reference.
- **Dependencies:** Task 4.2
- **Acceptance Criteria:**
  - Stores paths and fingerprints.
  - Marks unsupported files without crashing.
  - Does not copy, delete, or mutate originals.
- **Validation:** Test fixture folder with mixed files.

**Status:** Completed on 2026-06-08. `silica-catalog` defines the alpha import candidate contract and supported extension list, and `silica-storage` records immediate child files by reference with file size, modified time, partial hash, and unsupported state. Tests use a mixed fixture folder and verify originals are not copied or mutated.

### Task 4.4: Implement Rating, Pick, Reject Persistence

- **Location:** `crates/silica-catalog`, `apps/desktop`
- **Description:** Persist `rating`, `picked`, `rejected`, and `color_label`.
- **Dependencies:** Task 4.3
- **Acceptance Criteria:**
  - Values survive app restart.
  - SQLite `photo_flags` is authoritative inside the app.
- **Validation:** Unit and restart integration tests.

**Status:** Completed on 2026-06-08. `silica-catalog` defines validated photo flags, `silica-storage` creates default `photo_flags` rows for imported photos and persists updates, `silica-core` exposes command-facing APIs, and the minimal Tauri shell exposes flag read/write commands. Tests verify catalog-authoritative values survive reopen. Sidecar flag mirroring and product UI controls are still later tasks.

## Phase 5: Preview, Basic Edit, and Export MVP

**Goal:** Make the local DMG alpha meaningfully usable as a photo editor.

**Demo/Validation:**

- User can preview a supported image, apply exposure/contrast, persist edits, and export JPEG sRGB.

### Task 5.1: Implement Preview Path for Selected Decoder

- **Location:** `crates/silica-decode`, `crates/silica-render`, `apps/desktop`
- **Description:** Implement the minimal preview path allowed by Spike 002 and Spike 001.
- **Dependencies:** Tasks 3.1, 3.2
- **Acceptance Criteria:**
  - Supported files render a preview.
  - Unsupported files show a clear state.
  - Decoder assumptions are documented.
- **Validation:** Manual preview test with fixture classes.

**Status:** Completed on 2026-06-08 as the minimal preview path contract. Raster candidates such as JPEG return a ready preview status by reference, unsupported catalog entries return a clear unsupported state, and RAW candidates return a Core Image RAW blocked state until fixture-backed probe coverage exists. The product Metal viewer, RAW decoding, color correctness proof, and M004/M005 UI screens remain later explicit tasks.

### Task 5.2: Implement Edit Graph Types and Validation

- **Location:** `crates/silica-edit`, `schemas/edit_graph.schema.json`
- **Description:** Implement typed Rust structures equivalent to the schema.
- **Dependencies:** Task 4.1
- **Acceptance Criteria:**
  - Serialization matches `schemas/edit_graph.example.json`.
  - JSON validates against schema.
  - Unknown experimental data stays under `extensions`.
- **Validation:** Serialization and schema validation tests.

**Status:** Completed on 2026-06-09. `silica-edit` now exposes typed Rust structures for the edit graph schema, preserves the example JSON shape through round-trip serialization, validates schema/version/range constraints, rejects unknown closed-field data through `serde(deny_unknown_fields)`, and keeps experimental top-level data under `extensions`. This task does not apply edits, persist sidecars, render previews, add UI controls, implement RAW decoding, or add MLX/MCP/plugin behavior.

### Task 5.3: Implement Exposure and Contrast Edit Flow

- **Location:** `crates/silica-edit`, `crates/silica-render`, `apps/desktop`
- **Description:** Add basic edit controls and render request updates.
- **Dependencies:** Tasks 5.1, 5.2
- **Acceptance Criteria:**
  - Exposure and contrast values update preview.
  - No DB write occurs per slider tick.
  - Final value persists on commit/release.
- **Validation:** Unit test for edit state and manual slider test.

**Status:** Completed on 2026-06-09 as a command/API-level edit flow. `silica-edit` can build a default graph and apply validated exposure/contrast values, `silica-render` returns draft preview adjustment requests, `silica-storage` persists active edit graphs only on commit, `silica-core` proves preview updates do not write `edit_states`, and the minimal Tauri shell exposes preview/commit commands. Product M005 Develop UI controls, actual pixel rendering, sidecar writing, RAW decoding, and Metal viewer behavior remain later explicit tasks.

### Task 5.4: Implement JPEG sRGB Export

- **Location:** `crates/silica-export`, `crates/silica-render`
- **Description:** Export the edited image as JPEG sRGB.
- **Dependencies:** Tasks 3.3, 5.3
- **Acceptance Criteria:**
  - Export writes only to the chosen export location.
  - Original source file remains unchanged.
  - Export record is stored in catalog.
- **Validation:** Original hash protection test and exported JPEG inspection.

**Status:** Completed on 2026-06-09 as a command/API-level local alpha export path. `silica-export` decodes an already-rendered raster source, applies persisted exposure/contrast values on the CPU, writes a separate JPEG output at fixed local-alpha quality, and refuses to overwrite the original source path. `silica-render` records the JPEG sRGB export request contract, `silica-storage` stores the export row and marks `photo_flags.exported`, `silica-core` orchestrates edit state, export, and catalog recording, and the minimal Tauri shell exposes an export command. This task does not implement RAW decoding, Metal rendering, UI export screens, broad color fixture validation, Display P3 export, MLX/MCP/plugin behavior, or auto-update/distribution changes.

## Phase 5.5: UI MVP Vertical Slice

**Goal:** Make the local alpha workflow usable through real app screens before local install QA.

**Demo/Validation:**

- A tester can use the app window to create/open a library, import a folder by reference, browse a grid, cull photos, adjust exposure/contrast, and export JPEG sRGB.
- UI follows `MockupUI/` information structure and the design-system/component specifications.
- UI QA runs on the connected workflow first, then responsive variants.

### Task 5.5.1: Establish UI Design System Baseline

- **Location:** `docs/wiki/topics/ui-mvp-baseline.md`, `apps/desktop/static/styles/`, `apps/desktop/static/index.html`
- **Description:** Record the UI MVP plan, define the source hierarchy, run `ui-ux-pro-max` against the product direction, and start tokenized static frontend styles.
- **Dependencies:** Tasks 5.4, `MockupUI/MANIFEST.md`, `docs/05_Design_System_Specification.md`, `docs/05_5_Component_Library_Specification.md`, `docs/06_Screen_Inventory_and_Wireframe_Specification.md`
- **Acceptance Criteria:**
  - Task 5.5 is atomized in the wiki before implementation proceeds.
  - Baseline states that `MockupUI/` is the product UI target.
  - `ui-ux-pro-max` output is recorded with accepted and rejected guidance.
  - Static frontend loads shared design tokens instead of inline hard-coded colors.
- **Validation:** Markdown link check, dependency guard, and harness pass.

**Status:** Completed on 2026-06-09. The UI MVP baseline is recorded in [UI MVP Baseline](../topics/ui-mvp-baseline.md), Task 5.5 is atomized below, `ui-ux-pro-max` guidance is explicitly filtered against SilicaRAW's Apple Pro App design direction, and the current static shell now loads token/base CSS from `apps/desktop/static/styles/`.

### Task 5.5.2: Build App Frame and Mode Navigation

- **Location:** `apps/desktop/static/`, `docs/wiki/topics/ui-mvp-baseline.md`
- **Description:** Replace the minimal static shell with the global app frame: toolbar, sidebar region, main content region, inspector region, status/progress area, and Library/Develop/Export mode navigation.
- **Dependencies:** Task 5.5.1
- **Acceptance Criteria:**
  - Frame follows `docs/06` global app frame rules.
  - No screen content is hidden under native viewer proof layers.
  - Navigation state is visible and keyboard reachable.
- **Validation:** Static app smoke test and screenshot check at 1280px and 1440px.

**Status:** Completed on 2026-06-09. The static desktop shell now has the shared app frame from the UI MVP baseline: top toolbar, Library/Develop/Export mode navigation, left sidebar, central work surface, right inspector, and bottom status bar. Mode switching updates `data-active-mode`, `aria-pressed`, visible sidebar/main/inspector panels, and the status output without implementing the deeper Welcome, Import, Grid, Develop, or Export dialog workflows. The create/open library command buttons remain wired to the existing Tauri commands.

### Task 5.5.3: Implement Welcome and Library Open/Create UI

- **Location:** `apps/desktop/static/`, `MockupUI/M001_Welcome.png`
- **Description:** Implement the first-launch/welcome screen and connect create/open library commands.
- **Dependencies:** Task 5.5.2
- **Acceptance Criteria:**
  - User can create or open a local library from the app.
  - Success/error states are visible in the app frame.
  - Screen matches M001 information structure.
- **Validation:** Tauri command smoke test through the UI.

**Status:** Completed on 2026-06-09. Library mode now defaults to an M001-style welcome state with full-width welcome content, Open Folder, Create Library, Open Recent affordance, disabled sample project affordance, recent library rows, inline status output, and existing Tauri `open_library` / `create_library` command wiring. The welcome state hides the sidebar, inspector, and bottom status bar to match the first-launch mockup; successful Tauri command completion switches the app frame to the normal library workbench state. Native folder picking, real recent-library persistence, and sample project loading remain outside this task.

### Task 5.5.4: Implement Import Flow UI

- **Location:** `apps/desktop/static/`, `MockupUI/M009_Import_Progress.png`
- **Description:** Add import-by-reference controls and import progress/status feedback.
- **Dependencies:** Task 5.5.3
- **Acceptance Criteria:**
  - User can enter/select an import folder path and import by reference.
  - UI states that original files stay in place.
  - Unsupported file count and errors are visible.
- **Validation:** Import a mixed folder and verify catalog rows without copying originals.

**Status:** Completed on 2026-06-09. Library workbench now exposes an M009-style import-by-reference flow with an import folder path, original-file safety note, overall progress, step status rows, unsupported/error summary, View Errors affordance, and a modal-style progress panel over the library surface. The Tauri shell now exposes a thin `import_folder` command that delegates to `silica_core::import_folder` and returns scanned/supported/unsupported counts while preserving source files. Native folder picking, pause/cancel/minimized background jobs, recursive import, real progress events, persisted import history, and populated grid rendering remain later scoped work.

### Task 5.5.5: Implement Library Grid MVP

- **Location:** `apps/desktop/static/`, `MockupUI/M003_Library_Grid_populated.png`
- **Description:** Show imported catalog photos in a grid with selection and culling controls.
- **Dependencies:** Task 5.5.4
- **Acceptance Criteria:**
  - Grid can show imported photos using catalog data.
  - Rating, pick, and reject actions call the command/API path.
  - Empty, loading, missing, and unsupported states are represented.
- **Validation:** Grid smoke test and culling persistence check.

**Status:** Completed on 2026-06-09. The Library workbench now renders an M003-style catalog grid with selected-card state, file type badges, rating rows, pick/reject/missing/unsupported state badges, empty/loading states, bottom photo count, and a thumbnail-size control. A thin `list_library_photos` command now reads catalog rows through storage/core and returns JSON for the desktop grid, while rating, pick, and reject inspector actions reuse the existing `set_photo_flags` command path. Real thumbnail generation, virtualized scrolling, native preview opening, advanced filters, and populated metadata/histogram values remain later scoped work.

### Task 5.5.6: Implement Preview/Loupe MVP

- **Location:** `apps/desktop/static/`, `MockupUI/M004_Library_Loupe.png`
- **Description:** Add one-photo preview/loupe state backed by the existing preview readiness command.
- **Dependencies:** Task 5.5.5
- **Acceptance Criteria:**
  - JPEG raster candidates show a preview surface.
  - RAW candidates show the blocked decode state without implying RAW decode is implemented.
  - Unsupported files show a clear unsupported state.
- **Validation:** Preview status smoke test with JPEG, RAW placeholder, and unsupported file.

**Status:** Completed on 2026-06-09. The Library workbench now has an M004-style Loupe view that opens from the selected grid photo, displays file name, rating, fit controls, preview readiness status, and a bottom filmstrip. The Loupe uses the existing `open_photo_preview` command path when running inside Tauri and falls back to the same local alpha readiness rules for static UI smoke checks. JPEG/raster candidates show the ready preview surface, RAW candidates show the blocked decode state, and unsupported entries show an unsupported state. This does not implement RAW decoding, real pixel rendering, Metal viewer output, full metadata, or Develop edits.

### Task 5.5.7: Implement Develop Panel MVP

- **Location:** `apps/desktop/static/`, `MockupUI/M005_Develop_default.png`
- **Description:** Add Develop mode controls for exposure and contrast using the current preview/commit commands.
- **Dependencies:** Task 5.5.6
- **Acceptance Criteria:**
  - Exposure and contrast controls use `SrAdjustmentSlider` rules.
  - Draft updates do not write per tick.
  - Commit persists final values.
- **Validation:** Edit preview/commit smoke test and no-draft-write check.

**Status:** Completed on 2026-06-09. Develop mode now has an M005-style MVP surface with selected photo context, preview readiness state, bottom filmstrip, Basic exposure and contrast `SrAdjustmentSlider` controls, manual numeric inputs, reset actions, draft dirty state, revert, and explicit commit. Slider and numeric input changes call the existing `preview_exposure_contrast_edit` path only for draft feedback, while `Commit Edit` is the only UI action wired to `commit_exposure_contrast_edit`. Static smoke mode mirrors the same local-alpha boundaries without claiming catalog persistence. This does not implement RAW decoding, real pixel rendering, Metal viewer output, masks, full tone/color/detail controls, or sidecar writing.

### Task 5.5.8: Implement Export Dialog MVP

- **Location:** `apps/desktop/static/`, `MockupUI/M007_Export_Dialog.png`
- **Description:** Add export dialog UI connected to the JPEG sRGB export command.
- **Dependencies:** Task 5.5.7
- **Acceptance Criteria:**
  - User can enter/select an output path and export JPEG sRGB.
  - UI states that originals are not modified.
  - Export success/error state is visible.
- **Validation:** Export smoke test, exported JPEG inspection, and catalog export record check.

**Status:** Completed on 2026-06-09. Export mode now opens an M007-style Export Photos dialog for the selected catalog photo. The dialog accepts a local output path, locks the local-alpha settings to JPEG, sRGB, and quality 90, shows a non-destructive original-file safety note, validates that the output path differs from the referenced original, and surfaces ready, blocked, success, and error states. Runtime export uses the existing `export_photo_jpeg_srgb` command, while static smoke mode shows that the desktop runtime is required to write the JPEG and catalog export record. This does not add native folder picking, multi-photo export, presets, alternate formats, Display P3, resizing, metadata policy editing, RAW decoding, Metal rendering, or sidecar writing.

### Task 5.5.9: Add UI Workflow Smoke Harness

- **Location:** `scripts/harness/`, `apps/desktop/static/`, test docs
- **Description:** Add a lightweight repeatable UI smoke path for the connected local alpha workflow.
- **Dependencies:** Tasks 5.5.3 through 5.5.8
- **Acceptance Criteria:**
  - Harness documents or automates the core UI workflow.
  - It does not require MLX, MCP, plugin runtime, cloud, or telemetry.
- **Validation:** Harness passes locally and in CI where feasible.

**Status:** Completed on 2026-06-09. The main harness now runs `scripts/harness/check-ui-workflow-smoke.py`, a Python stdlib static workflow contract check for the connected local alpha path: open/create library, import by reference, browse/cull the grid, open loupe preview, apply Develop exposure/contrast, and export JPEG sRGB. The harness verifies required UI element IDs, command wiring, non-destructive original-file copy/reference messaging, Develop edit bounds, locked JPEG sRGB export settings, and the export guard that blocks writing over the referenced original path. It does not require a browser automation dependency, MLX, MCP, plugin runtime, cloud services, telemetry, RAW decoding, or Metal rendering.

### Task 5.5.10: Run Visual and Responsive QA Pass

- **Location:** `MockupUI/`, `apps/desktop/static/`, QA notes
- **Description:** Compare implemented M003/M005/M007 surfaces against compact and large mockup variants.
- **Dependencies:** Task 5.5.9
- **Acceptance Criteria:**
  - No text overlaps or clipped controls at 1280px, 1440px, and 1728px.
  - Layout preserves the photo-first hierarchy.
  - Export and Develop dialogs/panels remain usable at compact width.
- **Validation:** Screenshot review and recorded QA notes.

**Status:** Completed on 2026-06-09. Browser QA covered Library grid, Develop, and Export dialog at `1280x800`, `1440x900`, and `1728x965` against the M003/M005/M007 compact and large mockup families. The pass found and fixed a 1280px toolbar density issue where the mode switcher could visually collide with the search/actions region. After the CSS fix, horizontal overflow is false, visible clipping candidates are zero, toolbar mode/action overlap is zero, and Export/Develop remain usable at compact width. Recorded notes are in [UI Visual and Responsive QA](../topics/ui-visual-responsive-qa.md).

## Phase 5.6: Product Alpha Runtime Completion

**Goal:** Turn the Phase 5.5 screen/command-wired UI into a usable local alpha app before install QA.

Phase 5.5 proved screen structure and command paths. It did not prove a real installed photo-editing loop because important surfaces still rely on typed paths, string command parsing, static demo state, placeholder thumbnails/previews, and QA-simulated cache clearing. Phase 5.6 is the product runtime completion pass.

**Installed-alpha capability contract:**

- JPEG/JPG originals are the first fully supported visible photo path for grid thumbnails, loupe preview, Develop preview, persisted exposure/contrast, and JPEG sRGB export.
- RAW files may be imported only as clearly decode-blocked catalog entries until RAW decoding is explicitly implemented.
- Unsupported files must never look editable or exportable.
- PNG, TIFF, HEIC, and other raster formats are not guaranteed installed-alpha edit/export inputs until codec support, UI behavior, and tests are explicitly added.
- Original source files must remain unmodified.

**Demo/Validation:**

- A tester can use the installed app to create/open a library, import JPEG/JPG originals by reference, see real thumbnails, open a real loupe preview, visibly adjust exposure/contrast, commit the edit, export JPEG sRGB, clear disposable caches, restart, reopen, and see persisted state.
- The same workflow is covered by a connected runtime smoke path before clean-Mac DMG QA begins.
- Phase 6 clean-Mac install QA does not start until this phase is complete.

See [Product Alpha Runtime Completion](../topics/product-alpha-runtime-completion.md) for the runtime gap audit.

### Task 5.6.1: Runtime Gap Audit and Alpha Capability Contract

- **Location:** `docs/wiki/topics/product-alpha-runtime-completion.md`, `docs/wiki/roadmaps/local-dmg-distribution-plan.md`
- **Description:** Record the product/runtime gaps found after Phase 5.5 and define the exact installed-alpha capability contract before more runtime work begins.
- **Dependencies:** Task 5.5.10
- **Acceptance Criteria:**
  - Phase 5.6 is atomized before implementation continues.
  - The installed-alpha visible photo path is scoped to JPEG/JPG until more codecs are proven.
  - Missing product runtime behavior is separated from Phase 6 packaging QA.
  - Phase 6 dependencies are updated so clean-Mac QA waits for Phase 5.6.
- **Validation:** Markdown link check and harness pass.

**Status:** Completed on 2026-06-09. Added the Phase 5.6 runtime gap audit and alpha capability contract in [Product Alpha Runtime Completion](../topics/product-alpha-runtime-completion.md), inserted Tasks 5.6.1 through 5.6.12 into this roadmap, and clarified that Phase 6 clean-Mac QA resumes only after the product runtime loop is complete.

### Task 5.6.2: Structured Desktop Command Responses

- **Location:** `apps/desktop/src-tauri/src/main.rs`, `apps/desktop/static/index.html`, `crates/silica-core`
- **Description:** Replace command status strings and frontend regex parsing with structured success/error response envelopes.
- **Dependencies:** Task 5.6.1
- **Acceptance Criteria:**
  - Library, import, grid, culling, preview, edit, export, and future cache commands return typed/JSON-shaped data instead of parse-only strings.
  - Frontend uses fields, not regex, to update UI state.
  - Error responses expose stable kind, message, and relevant path/photo context.
  - Existing Rust command tests cover the response shape.
- **Validation:** Desktop command tests, static UI contract, and harness pass.

**Status:** Completed on 2026-06-09. Desktop Tauri commands now return structured `ok` / `data` / `error` response envelopes instead of parse-only status strings. The frontend reads stable response fields for library, import, grid, culling, preview, Develop preview/commit, and JPEG export flows, and the UI workflow smoke harness rejects legacy command string parsing helpers. `serde` is now a direct desktop dependency for response serialization and is recorded in [Dependencies](../../DEPENDENCIES.md).

### Task 5.6.3: Native Path Picker UX

- **Location:** `apps/desktop/src-tauri`, `apps/desktop/static/`, `docs/DEPENDENCIES.md` if a Tauri plugin is added
- **Description:** Add native or product-grade selectable path affordances for library create/open, import folder, and export output path.
- **Dependencies:** Task 5.6.2
- **Acceptance Criteria:**
  - Users do not need to type absolute paths for the main installed-alpha workflow.
  - Cancel is distinct from error.
  - Selected paths flow into the existing create/open/import/export commands.
  - Any new dependency is documented in `docs/DEPENDENCIES.md`.
- **Validation:** Desktop command/UI smoke and harness pass.

### Task 5.6.4: Real JPEG Thumbnail Cache MVP

- **Location:** `crates/silica-storage`, `crates/silica-core`, `apps/desktop/static/`
- **Description:** Generate and render real JPEG/JPG thumbnails for supported imported originals.
- **Dependencies:** Task 5.6.2
- **Acceptance Criteria:**
  - JPEG/JPG catalog rows expose a safe thumbnail asset or encoded preview field.
  - Thumbnail files live under the library `thumbnails/` directory.
  - `cache_records` or equivalent catalog state records disposable thumbnail cache metadata.
  - RAW/unsupported entries keep clear blocked/unsupported placeholders.
  - Original files are not modified.
- **Validation:** Thumbnail cache test, UI smoke, original hash protection, and harness pass.

### Task 5.6.5: Real JPEG Loupe Preview MVP

- **Location:** `crates/silica-core`, `apps/desktop/static/`, `crates/silica-storage`
- **Description:** Show real JPEG/JPG preview pixels in the Loupe view.
- **Dependencies:** Task 5.6.4
- **Acceptance Criteria:**
  - Opening Loupe for JPEG/JPG displays the selected original or generated preview image.
  - RAW candidates show decode-blocked state without implying RAW decoding.
  - Missing/unsupported candidates show clear blocked states.
  - Preview cache data is disposable.
- **Validation:** Preview UI smoke, cache/original safety test, and harness pass.

### Task 5.6.6: Real JPEG Develop Preview MVP

- **Location:** `crates/silica-core`, `crates/silica-render`, `apps/desktop/static/`
- **Description:** Make exposure/contrast changes visibly affect JPEG/JPG Develop preview pixels before commit.
- **Dependencies:** Task 5.6.5
- **Acceptance Criteria:**
  - Exposure/contrast slider changes update the visible Develop preview for JPEG/JPG.
  - Draft preview updates remain non-persistent until commit.
  - Commit persists the active edit graph.
  - RAW candidates may keep valid edit graph state while preview pixels remain decode-blocked.
- **Validation:** Draft/no-write test, visual/runtime smoke, edit persistence test, and harness pass.

### Task 5.6.7: Persisted Edit-State Readback in UI

- **Location:** `crates/silica-core`, `apps/desktop/src-tauri/src/main.rs`, `apps/desktop/static/index.html`
- **Description:** Expose committed active edit state to the frontend and restore Develop controls after open/reopen.
- **Dependencies:** Task 5.6.6
- **Acceptance Criteria:**
  - The frontend can read committed exposure/contrast for the selected photo.
  - Develop sliders and edited/clean state match catalog state after library reopen.
  - Restart/reopen UI validation no longer relies on in-memory JavaScript state.
- **Validation:** Core/desktop command tests, UI restart smoke, and harness pass.

### Task 5.6.8: Product Cache Clear Command and Maintenance UI

- **Location:** `crates/silica-storage`, `crates/silica-core`, `apps/desktop/src-tauri/src/main.rs`, `apps/desktop/static/`, `MockupUI/M008_Preferences_Appearance.png`
- **Description:** Add a safe cache-clear product path and minimal maintenance UI.
- **Dependencies:** Tasks 5.6.4, 5.6.5, 5.6.6
- **Acceptance Criteria:**
  - Cache clear deletes only disposable cache directories such as `thumbnails`, `previews`, `render-cache`, and `ai-cache`.
  - Catalog, edit state, export records, sidecars, backups, logs, and original source files are preserved.
  - Cache directories are recreated after clear.
  - UI labels make the destructive scope precise.
- **Validation:** Automated cache-clear/original hash test, UI smoke, and harness pass.

### Task 5.6.9: Remove Fake Demo State and Harden Culling UX

- **Location:** `apps/desktop/static/`, `crates/silica-core`
- **Description:** Remove hardcoded demo recents/grid assumptions and make minimal culling controls usable.
- **Dependencies:** Task 5.6.7
- **Acceptance Criteria:**
  - Clean launch shows a real empty/recent state, not fictional hardcoded project rows.
  - Rating can be set from 0 through 5.
  - Pick/reject can be toggled and remain mutually coherent.
  - Dead controls are hidden, disabled with clear reason, or wired to real behavior.
  - Alpha copy accurately states blocked RAW/Metal/AI capabilities.
- **Validation:** Static UI contract, culling persistence test, visual QA spot check, and harness pass.

### Task 5.6.10: Legal QA Fixture Generator and Installed-App Preflight

- **Location:** `scripts/harness/`, `checklists/`, `docs/wiki/topics/product-alpha-runtime-completion.md`
- **Description:** Add repeatable local fixtures and a developer installed-app preflight before clean-Mac testing.
- **Dependencies:** Task 5.6.9
- **Acceptance Criteria:**
  - Fixture generator creates legal JPEG/JPG supported samples, unsupported files, and optional RAW-blocked placeholders without committing user photos.
  - Fixture metadata includes expected source hashes.
  - Developer preflight records app artifact, macOS version, fixture path, hash results, and known limitations.
  - Temporary artifacts are ignored by git.
- **Validation:** Fixture generator check, docs link check, and harness pass.

### Task 5.6.11: Connected Runtime UI Smoke

- **Location:** `scripts/harness/`, `apps/desktop/`, `checklists/`
- **Description:** Add a repeatable smoke path against the actual desktop runtime rather than static HTML inspection only.
- **Dependencies:** Task 5.6.10
- **Acceptance Criteria:**
  - Smoke covers create/open library, import, grid, rating/pick/reject, loupe, Develop edit, export, cache clear, reopen, and original hash check.
  - The smoke path clearly separates local developer runtime from clean-Mac DMG execution.
  - It does not require MLX, MCP, plugin runtime, cloud, telemetry, RAW decoding, or Metal rendering.
- **Validation:** Runtime smoke and harness pass.

### Task 5.6.12: Final Visual and Responsive QA Refresh

- **Location:** `MockupUI/`, `apps/desktop/static/`, `docs/wiki/topics/ui-visual-responsive-qa.md`
- **Description:** Re-run visual/responsive QA after real pixels, native/selectable path UX, cache UI, and demo-state cleanup are present.
- **Dependencies:** Task 5.6.11
- **Acceptance Criteria:**
  - M001, M002, M003, M004, M005, M007, M008-minimal, and M009 surfaces remain visually coherent.
  - Real thumbnails/previews do not create overflow, clipping, or text overlap.
  - The UI remains usable at compact, standard, and large desktop widths.
  - Final QA notes explicitly supersede the static-only Phase 5.5 visual pass as the Phase 6 readiness gate.
- **Validation:** Screenshot review, DOM overflow/clipping checks, runtime smoke, and harness pass.

## Phase 6: Local Install QA

**Goal:** Verify the completed product alpha runtime behaves correctly when installed from DMG, not only when run from the build tree.

Phase 6 is not a substitute for missing app behavior. Tasks 6.1 and 6.2 created useful preparatory checklists and original-safety automation, but clean-Mac execution waits for Phase 5.6.

**Demo/Validation:**

- DMG install test passes on a clean Apple Silicon Mac after Phase 5.6 is complete.

### Task 6.1: Add Install Smoke Test Checklist

- **Location:** `checklists/LOCAL_DMG_INSTALL_CHECKLIST.md`
- **Description:** Add a checklist for installing from DMG and running the local alpha workflow.
- **Dependencies:** Phase 5.6 for execution; checklist authoring was completed earlier.
- **Acceptance Criteria:**
  - Includes mount DMG, drag app, launch from `/Applications`, import folder, edit, export, restart.
  - Includes offline launch check.
- **Validation:** Checklist is executed manually.

**Status:** Completed on 2026-06-09. Added [Local DMG Install Smoke Checklist](../../../checklists/LOCAL_DMG_INSTALL_CHECKLIST.md) for developer unsigned DMGs and later signed/notarized release candidates. The checklist records artifact metadata, mount and drag-to-`/Applications` install steps, launch from `/Applications`, offline launch, local library create/open, import by reference, grid culling, preview/loupe, exposure/contrast edit commit, JPEG sRGB export, restart persistence, and a quick original-file hash spot check. Manual execution remains a later QA step.

### Task 6.2: Run Original Safety QA

- **Location:** `checklists/QA_CHECKLIST.md`, test fixtures
- **Description:** Verify local alpha workflow does not mutate original source files.
- **Dependencies:** Phase 5.6 for manual installed-app execution; automated core safety was completed earlier.
- **Acceptance Criteria:**
  - Original file hashes are unchanged after import, edit, export, cache clear, and restart.
- **Validation:** Automated hash test and manual QA record.

**Status:** Completed on 2026-06-09. Added `silica-core` automated original-safety hash QA for the connected local alpha workflow. The generated fixture test records an original JPEG hash, imports by reference, updates culling flags, opens preview, runs draft exposure/contrast preview, commits the edit, exports JPEG sRGB to a separate output path, deletes current disposable cache directories, reopens the library, and verifies the original hash remains unchanged after each stage. The current alpha has no product cache-clear command, so cache clear is represented by deleting `thumbnails`, `previews`, `render-cache`, and `ai-cache` under the disposable test library. Manual QA record fields were added to [QA Checklist](../../../checklists/QA_CHECKLIST.md).

### Task 6.3: Run Clean-Mac DMG Test

- **Location:** release candidate artifact
- **Description:** Install the DMG on a Mac that did not build the app.
- **Dependencies:** Phase 5.6, Task 6.1, Task 6.2
- **Acceptance Criteria:**
  - App launches from `/Applications`.
  - No missing bundled resources.
  - Local alpha workflow completes.
- **Validation:** Manual test record with macOS version and machine model.

## Phase 7: Signing and Notarization

**Goal:** Make the DMG acceptable to Gatekeeper for normal downloaded-app behavior.

**Demo/Validation:**

- Signed and notarized DMG installs and launches without command-line quarantine removal.

### Task 7.1: Prepare Apple Developer Credentials

- **Location:** maintainer Apple Developer account, GitHub repository secrets
- **Description:** Create and store required signing and notarization credentials.
- **Dependencies:** Phase 2
- **Acceptance Criteria:**
  - Developer ID Application certificate exists.
  - Certificate is exported as password-protected `.p12`.
  - GitHub Secrets include certificate, certificate password, keychain password, and notarization credentials.
- **Validation:** CI can import certificate into a temporary keychain.

### Task 7.2: Configure Hardened Runtime and Entitlements

- **Location:** Tauri/macOS bundle config
- **Description:** Configure entitlements required for the app and avoid unnecessary permissions.
- **Dependencies:** Task 7.1
- **Acceptance Criteria:**
  - Hardened runtime is enabled for Developer ID distribution.
  - Entitlements are minimal and reviewed.
- **Validation:** `codesign --display --entitlements :-` output is reviewed.

### Task 7.3: Sign and Notarize Local Release Candidate

- **Location:** local or GitHub Actions macOS runner
- **Description:** Build, sign, notarize, and staple the `.app` and/or `.dmg` according to the chosen workflow.
- **Dependencies:** Tasks 7.1, 7.2
- **Acceptance Criteria:**
  - Notarization succeeds.
  - Stapling succeeds.
  - Gatekeeper assessment passes.
- **Validation:**
  - `spctl --assess --type execute --verbose SilicaRAW.app`
  - `spctl --assess --type open --verbose SilicaRAW.dmg`

## Phase 8: GitHub Release Pipeline

**Goal:** Publish a downloadable DMG from GitHub.

**Demo/Validation:**

- A GitHub Release contains the DMG, checksums, and notes.

### Task 8.1: Add Release Build Workflow

- **Location:** `.github/workflows/release-macos.yml`
- **Description:** Build app on macOS runner, sign, notarize, staple, generate checksum, and upload artifacts.
- **Dependencies:** Phase 7
- **Acceptance Criteria:**
  - Workflow triggers on version tag.
  - Workflow uploads DMG and checksum as artifacts.
  - Secrets are not printed in logs.
- **Validation:** Workflow succeeds on a prerelease tag.

### Task 8.2: Publish GitHub Release Asset

- **Location:** GitHub Releases
- **Description:** Attach the DMG and SHA256 checksum to a draft GitHub Release.
- **Dependencies:** Task 8.1
- **Acceptance Criteria:**
  - Release contains DMG.
  - Release contains checksum.
  - Release notes include supported macOS version, known issues, and local-only privacy statement.
- **Validation:** Download asset from release page.

### Task 8.3: Verify Downloaded Release Artifact

- **Location:** clean Apple Silicon Mac
- **Description:** Download the GitHub Release DMG and run the install workflow.
- **Dependencies:** Task 8.2
- **Acceptance Criteria:**
  - Checksum matches.
  - DMG opens.
  - App copies to `/Applications`.
  - App launches without command-line workaround.
  - Local alpha workflow completes.
- **Validation:** Completed install checklist.

## Phase 9: Local Distribution Hardening

**Goal:** Make repeated local releases predictable.

**Demo/Validation:**

- A maintainer can cut a new local alpha release using documented steps.

### Task 9.1: Add Release Runbook

- **Location:** `docs/wiki/roadmaps/local-dmg-release-runbook.md`
- **Description:** Document the exact manual and CI steps for cutting a local DMG release.
- **Dependencies:** Phase 8
- **Acceptance Criteria:**
  - Includes prerelease checklist.
  - Includes tag naming.
  - Includes rollback steps.
  - Includes notarization troubleshooting links.
- **Validation:** Maintainer dry-runs the runbook.

### Task 9.2: Add Release Notes Template

- **Location:** `.github/release-template.md`
- **Description:** Add release notes structure for local alpha DMG releases.
- **Dependencies:** Phase 8
- **Acceptance Criteria:**
  - Includes install steps.
  - Includes known issues.
  - Includes privacy statement.
  - Includes checksum verification command.
- **Validation:** Template used in draft release.

### Task 9.3: Decide Homebrew and Auto-Update Deferral

- **Location:** `docs/wiki/decisions/adr-0005-homebrew-and-auto-update-deferral.md`
- **Description:** Explicitly defer Homebrew Cask and auto-update until after local DMG alpha.
- **Dependencies:** Phase 8
- **Acceptance Criteria:**
  - Decision explains why DMG is first.
  - Decision lists prerequisites before revisiting auto-update.
- **Validation:** ADR linked from release docs.

## Parallelization Map

- Phase 1 can run while Phase 0 docs are finalized, after git is initialized.
- Phase 3 spikes can run partly in parallel if each produces isolated reports.
- Phase 4 storage work can begin after SQLite spike while rendering spikes continue.
- Phase 7 signing prep can begin early if Apple Developer credentials exist, but notarization cannot fully validate until there is an app bundle.
- Phase 8 must wait for signing/notarization and local install QA.

## Required Secrets for Signed GitHub DMG

Exact names may change during implementation, but the release workflow should need:

- `APPLE_CERTIFICATE`
- `APPLE_CERTIFICATE_PASSWORD`
- `KEYCHAIN_PASSWORD`
- `APPLE_SIGNING_IDENTITY`
- `APPLE_ID` and app-specific password, or App Store Connect API credentials
- `APPLE_TEAM_ID`

Do not store signing certificates or private keys directly in the repository.

## Rollback Plan

- If a release artifact is broken, mark the GitHub Release as draft or delete the release asset.
- If notarization fails, keep the unsigned DMG as an internal artifact only and do not publish it as user-installable.
- If Tauri + Metal fails, stop Tauri-dependent UI work and record the fallback decision before continuing.
- If data safety tests fail, block release until original-file protection is fixed.
- If Gatekeeper rejects the downloaded app, block release until signing, notarization, and stapling are fixed.

## Notes for LLM Agents

- Do not skip from packaging directly to product release. A DMG that installs a broken editor is not local distribution.
- Do not treat unsigned developer DMGs as user-ready releases.
- Do not add dependencies without updating `docs/DEPENDENCIES.md`.
- Do not add MLX, MCP, plugins, telemetry, cloud sync, or auto-update for local DMG alpha unless a maintainer explicitly changes the scope.
