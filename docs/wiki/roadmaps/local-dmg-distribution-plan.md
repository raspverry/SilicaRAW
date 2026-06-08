---
title: Local DMG Distribution Plan
status: active
audience: all
updated: 2026-06-08
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

### Task 5.4: Implement JPEG sRGB Export

- **Location:** `crates/silica-export`, `crates/silica-render`
- **Description:** Export the edited image as JPEG sRGB.
- **Dependencies:** Tasks 3.3, 5.3
- **Acceptance Criteria:**
  - Export writes only to the chosen export location.
  - Original source file remains unchanged.
  - Export record is stored in catalog.
- **Validation:** Original hash protection test and exported JPEG inspection.

## Phase 6: Local Install QA

**Goal:** Verify the app behaves correctly when installed from DMG, not only when run from the build tree.

**Demo/Validation:**

- DMG install test passes on a clean Apple Silicon Mac.

### Task 6.1: Add Install Smoke Test Checklist

- **Location:** `checklists/LOCAL_DMG_INSTALL_CHECKLIST.md`
- **Description:** Add a checklist for installing from DMG and running the local alpha workflow.
- **Dependencies:** Phase 5
- **Acceptance Criteria:**
  - Includes mount DMG, drag app, launch from `/Applications`, import folder, edit, export, restart.
  - Includes offline launch check.
- **Validation:** Checklist is executed manually.

### Task 6.2: Run Original Safety QA

- **Location:** `checklists/QA_CHECKLIST.md`, test fixtures
- **Description:** Verify local alpha workflow does not mutate original source files.
- **Dependencies:** Phase 5
- **Acceptance Criteria:**
  - Original file hashes are unchanged after import, edit, export, cache clear, and restart.
- **Validation:** Automated hash test and manual QA record.

### Task 6.3: Run Clean-Mac DMG Test

- **Location:** release candidate artifact
- **Description:** Install the DMG on a Mac that did not build the app.
- **Dependencies:** Task 6.1
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
