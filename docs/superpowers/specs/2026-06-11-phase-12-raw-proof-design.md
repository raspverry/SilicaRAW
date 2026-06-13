# Phase 12 RAW Proof Design

## Goal

Define the full Phase 12 implementation plan before adding a Core Image RAW probe, fixture-backed RAW evidence, support-matrix decisions, or product RAW decode contracts.

Phase 12 must prove which legal RAW fixture classes Core Image can handle before SilicaRAW shows RAW pixels in the product UI.

## Background

Phase 11 is complete. The app now has durable app-session state, real recents, launch restore, paged grid behavior, stored metadata display, reviewable import issues, opt-in recursive import, and connected runtime smoke coverage.

The current RAW boundary is still proof-only:

- `silica-decode` owns RAW decode decisions and currently exposes preview readiness, not decoded RAW pixels.
- `silica-core` calls `silica_decode::plan_preview_decode` and passes the result to `silica-render`.
- RAW candidates remain blocked by missing fixture-backed Core Image probe evidence.
- `docs/DEPENDENCIES.md` records Core Image RAW as the selected first implementation target but no direct `silica-decode` Core Image dependency exists yet.
- `Cargo.lock` already contains `objc2-core-image` transitively through existing Tauri/macOS dependencies, but adding a direct dependency to `silica-decode` still requires an explicit dependency entry or update.

## Accepted Phase Order

Keep this order:

1. Feature-gated Core Image RAW probe dependency gate.
2. Probe result type contract and backend skeleton.
3. Legal fixture probe harness and result recording.
4. Core Image support matrix and LibRaw gate.
5. Product RAW decode API contract.

Do not define product RAW decode behavior before fixture probe results exist.

Do not show RAW pixels in the app UI during Phase 12.

Do not add LibRaw unless Task 12.3 records a concrete fixture-backed Core Image gap and a dependency/distribution impact decision.

## Dependency Gate

Task 12.1 may add a direct macOS-only, optional Core Image binding dependency to `crates/silica-decode` only behind a non-default feature.

Preferred dependency direction:

```txt
silica-decode
  optional macOS Core Image binding
silica-core
  depends on silica-decode API contracts
silica-render
  does not own decoder-specific RAW decisions
silica-storage
  does not own decoder-specific RAW decisions
apps/desktop
  does not call Core Image directly
```

Candidate binding family:

```txt
objc2 0.6.4
objc2-foundation 0.3.2
objc2-core-image 0.3.2
objc2-core-graphics 0.3.2
sha2 0.10.9 for source SHA-256 evidence if the probe computes hashes in Rust
serde_json 1.0.150 for fixture manifest parsing if Task 12.2 parses manifests in Rust
```

These versions match the existing `objc2` family already used by the macOS Metal host spike. The exact feature set must be verified during Task 12.1 with `cargo info` and a feature build.

If direct Core Image binding, SHA-256 hashing, or fixture manifest parsing cannot be kept small and documented, stop and record a dependency decision before implementation.

## Feature Gate

Use one non-default feature:

```txt
core-image-raw-probe
```

Default workspace builds must not compile or run the Core Image probe.

Expected validation:

```bash
cargo test -p silica-decode
cargo test -p silica-decode --features core-image-raw-probe
scripts/harness/check.sh
```

On non-macOS targets, the feature must either compile to a clear unavailable probe result or remain unavailable without breaking default builds. The first implementation target is the user's local macOS environment.

## Probe Result Contract

The probe result must be structured and serializable enough for fixture evidence and later support-matrix docs.

Required fields:

```txt
backend
platform
macos_version
source_path
source_sha256
original_file_size
original_modified_at
status
width
height
orientation
error_category
message
```

Recommended enum values:

```txt
backend: core_image_raw
platform: macos | unsupported_platform
status: success | unsupported | failed | unavailable
error_category:
  unsupported_platform
  missing_file
  source_hash_mismatch
  core_image_unavailable
  core_image_open_failed
  core_image_metadata_missing
  permission_denied
  invalid_fixture
  unknown
```

Dimensions and orientation must be optional. Missing orientation must not become fake orientation.

## Fixture Evidence Contract

Task 12.2 reads a legal fixture manifest through `SILICARAW_RAW_FIXTURE_MANIFEST`.

Each probe run must:

- verify the fixture path is legal and relative through the fixture manifest rules
- verify or record the source SHA-256
- record the original file size and modification time before probing
- run the feature-gated probe
- verify the original file hash remains unchanged after probing
- emit deterministic structured output
- keep unsupported fixture classes explicitly blocked

Probe output may be stored as JSON under a docs or reports path only after the exact location is added to the Task 12.2 plan. Do not commit user photos or unlicensed fixture media.

## Support Matrix Contract

Task 12.3 updates `docs/wiki/topics/raw-decoding.md` with a fixture-backed matrix.

Each row should include:

```txt
fixture class
fixture id
format
backend
probe status
dimensions known
orientation known
evidence path or command
product status
notes
```

Allowed product statuses:

```txt
core_image_supported
blocked_pending_evidence
blocked_core_image_failed
blocked_unsupported_class
deferred
```

Support cannot be inferred from file extension. It must be tied to a legal fixture manifest and probe evidence.

## Product API Contract Boundary

Task 12.4 defines API contracts after Task 12.3.

The product API must return:

- decoded metadata
- dimensions
- orientation
- decoder backend
- explicit blocked states
- a message suitable for UI display later

The product API must not:

- wire RAW pixels into Library, Loupe, Develop, Export, or Metal UI
- mutate originals
- move decoder policy into render or catalog
- claim color correctness
- imply broad camera support

## Atomic Tasks

### Task 12.0.1: Phase 12 Design Gate

- **Location:** `docs/superpowers/specs/`, `docs/superpowers/plans/`, `docs/wiki/phases/phase-12-raw-proof.md`
- **Description:** Add this design gate and an implementation plan before Phase 12 code work.
- **Dependencies:** Phase 11
- **Acceptance Criteria:**
  - Dependency gate is documented.
  - Probe result contract is documented.
  - Fixture evidence contract is documented.
  - Stop gates are documented.
  - Implementation plan exists.
- **Validation:**
  - `python3 scripts/harness/check-md-links.py`
  - `scripts/harness/check.sh`
- **Status:** Completed on 2026-06-11. Added this Phase 12 RAW proof design and implementation plan before starting Task 12.1.

### Task 12.1.1: Core Image Dependency and Feature Gate

- **Location:** `crates/silica-decode/Cargo.toml`, `docs/DEPENDENCIES.md`
- **Description:** Add or confirm the minimal optional macOS Core Image binding dependencies behind `core-image-raw-probe`.
- **Dependencies:** Task 12.0.1
- **Acceptance Criteria:**
  - Feature is non-default.
  - Default `cargo test -p silica-decode` does not require Core Image bindings.
- Direct dependency additions or changes are documented.
- Source hashing dependencies are documented if Rust computes SHA-256.
- **Validation:** `cargo test -p silica-decode --features core-image-raw-probe`
- **Status:** Completed on 2026-06-12. Added the non-default `core-image-raw-probe` feature to `silica-decode`, documented direct Core Image/SHA-256 dependencies, and verified default plus feature builds without adding product RAW pixels.

### Task 12.1.2: Probe Type Contract

- **Location:** `crates/silica-decode/src/lib.rs`
- **Description:** Add typed probe request/result/status/error enums without product decode behavior.
- **Dependencies:** Task 12.1.1
- **Acceptance Criteria:**
  - Result contract includes backend, platform, source hash, dimensions, orientation, status, and error category.
  - Unsupported platform has an explicit result path.
  - No RAW pixels are returned.
- **Validation:** `cargo test -p silica-decode`
- **Status:** Completed on 2026-06-12. Added the proof-only RAW probe request/result/status/error contracts plus an unsupported fallback route, and verified the contract does not change existing preview readiness.

### Task 12.1.3: macOS Core Image Probe Backend

- **Location:** `crates/silica-decode/src/`
- **Description:** Implement the feature-gated macOS probe path.
- **Dependencies:** Task 12.1.2
- **Acceptance Criteria:**
  - Probe reads by path only.
  - Probe records metadata and errors.
  - Probe does not mutate source files.
  - Normal preview readiness behavior remains unchanged.
- **Validation:** `cargo test -p silica-decode --features core-image-raw-probe`
- **Status:** Completed on 2026-06-12. Added the feature-gated macOS Core Image probe path with file metadata capture, SHA-256 verification, Core Image extent probing, and explicit failure categories. Legal fixture evidence and product RAW support remain blocked for Task 12.2 and Task 12.3.

### Task 12.2.1: RAW Probe Result Harness

- **Location:** `crates/silica-decode`, `scripts/harness/`
- **Description:** Add an ignored fixture-manifest probe test and harness command.
- **Dependencies:** Task 12.1.3
- **Acceptance Criteria:**
  - `SILICARAW_RAW_FIXTURE_MANIFEST` is required for fixture probe tests.
  - Missing manifest fails clearly.
  - Original hash preservation is checked.
- **Validation:** `SILICARAW_RAW_FIXTURE_MANIFEST=... cargo test -p silica-decode --features core-image-raw-probe -- --ignored`
- **Status:** In progress on 2026-06-12. Added the manifest probe report types, feature-gated JSON loader, ignored fixture test, and manual harness command. Running the ignored fixture probe is blocked until a legal local RAW fixture manifest is supplied through `SILICARAW_RAW_FIXTURE_MANIFEST`.

### Task 12.2.2: RAW Probe Evidence Documentation

- **Location:** `docs/wiki/topics/raw-decoding.md`, `docs/wiki/log.md`
- **Description:** Record fixture probe result format and current evidence status.
- **Dependencies:** Task 12.2.1
- **Acceptance Criteria:**
  - Docs distinguish probe output from product RAW support.
  - Unsupported classes remain explicit.
  - No real fixture media is committed.
- **Validation:** `python3 scripts/harness/check-md-links.py`

### Task 12.3.1: Core Image Support Matrix

- **Location:** `docs/wiki/topics/raw-decoding.md`
- **Description:** Add the fixture-backed support matrix.
- **Dependencies:** Task 12.2.2
- **Acceptance Criteria:**
  - Each support row points to evidence.
  - Product status is explicit.
  - File extensions alone do not establish support.
- **Validation:** `python3 scripts/harness/check-md-links.py`
- **Status:** Completed on 2026-06-12 as a blocked-pending-evidence matrix. No legal fixture manifest is available, so no fixture class is supported and no Core Image gap exists to justify LibRaw.

### Task 12.3.2: LibRaw Gate Decision

- **Location:** `docs/wiki/decisions/`, `docs/DEPENDENCIES.md`
- **Description:** Keep LibRaw deferred or record a fixture-backed reason to revisit it.
- **Dependencies:** Task 12.3.1
- **Acceptance Criteria:**
  - If Core Image coverage is enough, LibRaw remains deferred.
  - If Core Image has a concrete gap, dependency and distribution impact are documented before any dependency is added.
- **Validation:** `python3 scripts/harness/check-cargo-deps.py`
- **Status:** Completed on 2026-06-12. LibRaw remains deferred because there is no fixture-backed Core Image gap or dependency/distribution decision.

### Task 12.4.1: Product RAW Decode API Types

- **Location:** `crates/silica-decode`, `crates/silica-core`
- **Description:** Define product-facing RAW decode API types for supported fixture classes.
- **Dependencies:** Task 12.3.2
- **Acceptance Criteria:**
  - API returns decoded metadata, dimensions, orientation, backend, and blocked states.
  - Unsupported RAWs remain blocked.
  - Render/catalog layers do not own decoder policy.
- **Validation:** `cargo test -p silica-decode -p silica-core`
- **Status:** Completed on 2026-06-12. Added product RAW decode plan types and a thin core wrapper. Because Task 12.3 recorded no supported fixture classes, product RAW decode returns blocked states only and does not expose pixels.

### Task 12.4.2: API Boundary Smoke

- **Location:** `crates/silica-core`
- **Description:** Add a narrow core test proving supported and blocked decode API responses stay inside core/decode boundaries.
- **Dependencies:** Task 12.4.1
- **Acceptance Criteria:**
  - No UI RAW pixel display is wired.
  - Preview readiness behavior remains stable until later UI work.
  - Original-file safety checks remain present.
- **Validation:** `scripts/harness/check.sh`
- **Status:** Completed on 2026-06-12. Added a core wrapper test for the blocked product RAW decode contract and verified the full harness. UI RAW pixels remain unwired and original-file safety tests remain present.

## Validation Strategy

Use the smallest useful validation for each task. Do not add broad fallback systems or large fixture matrices before evidence requires them.

Always run:

```bash
scripts/harness/check.sh
```

before claiming Phase 12 work complete.

## Stop and Redesign Triggers

Stop and redesign if a task would:

- require committing real RAW fixture media without license/provenance approval
- mutate original photos
- expose RAW pixels in product UI before Task 12.3
- add LibRaw before a fixture-backed gap and dependency decision
- add a broad fallback decoder stack
- move decoder-specific behavior into storage, render, or the desktop shell
- add color correctness claims before Phase 13

## Notes for LLM Agents

Use [LLM Routing Index](../../wiki/llm/index.md), [Phase 12 RAW Proof Brief](../../wiki/phases/phase-12-raw-proof.md), and the matching task card before reading the full roadmap. The wiki route is for token savings; this design remains the Phase 12 implementation contract.
