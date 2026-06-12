# Phase 12 RAW Proof Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement Phase 12 by proving Core Image RAW behavior on legal fixtures before product RAW pixels are exposed.

**Architecture:** `silica-decode` owns the feature-gated Core Image RAW probe and product decode contracts. `silica-core` may wrap the final product API after fixture evidence exists. Storage, render, and desktop UI do not own decoder-specific decisions in this phase.

**Tech Stack:** Rust 2021, optional macOS `objc2` Core Image bindings, existing fixture manifest schema, Python 3 harness checks, Markdown wiki/docs.

---

## File Structure

- Modify: `crates/silica-decode/Cargo.toml`
  - Add non-default `core-image-raw-probe` feature and optional macOS Core Image dependencies if Task 12.1 confirms them.
- Modify: `crates/silica-decode/src/lib.rs`
  - Add probe/result/API contracts and route to feature-gated backend modules.
- Create: `crates/silica-decode/src/core_image_raw_probe.rs`
  - macOS feature-gated Core Image probe implementation.
- Create: `crates/silica-decode/src/raw_probe_fixture.rs`
  - fixture manifest probe helpers and ignored tests if the implementation stays small enough for one module.
- Modify: `crates/silica-core/src/lib.rs`
  - Add product RAW decode API wrapper only after Task 12.3 support matrix exists.
- Modify: `docs/DEPENDENCIES.md`
  - Update if `silica-decode` adds direct external dependencies.
- Modify: `docs/wiki/topics/raw-decoding.md`
  - Add probe result status, support matrix, and LibRaw gate outcome.
- Modify: `docs/wiki/phases/phase-12-raw-proof.md`
  - Update task status links as tasks complete.
- Modify: `docs/wiki/tasks/12.1-core-image-raw-probe.md`
  - Mark Task 12.1 complete after validation.
- Modify: `docs/wiki/tasks/12.2-raw-fixture-probe-harness.md`
  - Mark Task 12.2 complete after validation.
- Modify: `docs/wiki/tasks/12.3-core-image-support-matrix.md`
  - Mark Task 12.3 complete after validation.
- Modify: `docs/wiki/tasks/12.4-product-raw-decode-api-contract.md`
  - Mark Task 12.4 complete after validation.
- Modify: `docs/wiki/log.md`
  - Append one entry per completed task.

Do not modify `apps/desktop/static/` or `apps/desktop/src-tauri/` in Phase 12 unless a task is explicitly expanded. No UI RAW display belongs here.

## Task 0: Verify Phase 12 Design Gate

**Files:**
- Read: `docs/superpowers/specs/2026-06-11-phase-12-raw-proof-design.md`
- Read: `docs/wiki/phases/phase-12-raw-proof.md`
- Read: `docs/wiki/tasks/12.1-core-image-raw-probe.md`

- [ ] **Step 1: Confirm routing docs are readable**

Run:

```bash
python3 scripts/harness/check-md-links.py
```

Expected: `local links ok`.

- [ ] **Step 2: Confirm full harness is green before code work**

Run:

```bash
scripts/harness/check.sh
```

Expected: `Harness checks passed`.

- [ ] **Step 3: Commit if only planning docs changed**

Run:

```bash
git status --short
git add docs/superpowers/specs/2026-06-11-phase-12-raw-proof-design.md docs/superpowers/plans/2026-06-11-phase-12-raw-proof.md docs/wiki/phases/phase-12-raw-proof.md docs/wiki/log.md
git commit -m "docs(raw): add phase 12 proof plan"
```

Expected: one docs-only commit.

## Task 1: Implement Task 12.1 Dependency and Feature Gate

**Files:**
- Modify: `crates/silica-decode/Cargo.toml`
- Modify: `docs/DEPENDENCIES.md`

- [x] **Step 1: Verify candidate dependency metadata**

Run:

```bash
cargo info objc2-core-image@0.3.2 --verbose
cargo info objc2-foundation@0.3.2 --verbose
cargo info sha2@0.10.9
```

Expected: versions and licenses match the entries planned for `docs/DEPENDENCIES.md`.

- [x] **Step 2: Add non-default feature and optional macOS dependencies**

Update `crates/silica-decode/Cargo.toml` with this shape after confirming `cargo info` still reports the same versions:

```toml
[features]
default = []
core-image-raw-probe = [
  "dep:objc2",
  "dep:objc2-core-graphics",
  "dep:objc2-core-image",
  "dep:objc2-foundation",
  "dep:sha2",
]

[dependencies]
sha2 = { version = "0.10.9", optional = true }

[target.'cfg(target_os = "macos")'.dependencies]
objc2 = { version = "0.6.4", optional = true }
objc2-core-graphics = { version = "0.3.2", default-features = false, optional = true }
objc2-core-image = { version = "0.3.2", default-features = false, features = ["CIContext", "CIImage", "CIRAWFilter", "objc2-core-foundation", "objc2-core-graphics", "objc2-image-io", "std"], optional = true }
objc2-foundation = { version = "0.3.2", default-features = false, features = ["NSDictionary", "NSError", "NSObject", "NSString", "NSURL", "NSGeometry", "std"], optional = true }
```

If this snippet does not compile, keep the same dependency family but reduce features to the smallest set that compiles and update `docs/DEPENDENCIES.md` with the verified set.

- [x] **Step 3: Document dependencies**

Add entries to `docs/DEPENDENCIES.md` for direct `silica-decode` usage of:

```txt
Name: objc2-core-image
Version: 0.3.2
Purpose: Feature-gated Core Image RAW probe bindings for Task 12.1.
License: Zlib OR Apache-2.0 OR MIT
Repository/Homepage: https://github.com/madsmtm/objc2
Used by: crates/silica-decode behind core-image-raw-probe
Why needed: Access Core Image RAW probe APIs without adding LibRaw.
Alternatives considered: raw Objective-C FFI, Swift shim, LibRaw, no probe.
Risk notes: Non-default macOS proof only; no product RAW pixels.
Binary size impact: No default build impact while feature is disabled.
Security notes: Reads local fixture paths only; must not mutate originals.
Verification source: cargo info objc2-core-image@0.3.2 --verbose
```

Add this entry if the feature uses Rust-side SHA-256:

```txt
Name: sha2
Version: 0.10.9
Purpose: Compute SHA-256 source hashes for feature-gated RAW fixture probe evidence.
License: MIT OR Apache-2.0
Repository/Homepage: https://github.com/RustCrypto/hashes
Used by: crates/silica-decode behind core-image-raw-probe
Why needed: Task 12.2 evidence must verify source hashes and original-file preservation.
Alternatives considered: Python-only hash verification, existing partial FNV-style test hash, platform-specific hashing APIs.
Risk notes: Non-default probe feature only; do not use partial hashes for fixture evidence.
Binary size impact: No default build impact while feature is disabled.
Security notes: Reads local fixture files only and does not mutate originals.
Verification source: cargo info sha2@0.10.9
```

Also add or update entries for `objc2`, `objc2-foundation`, `objc2-core-graphics`, and any other direct `silica-decode` dependency added in this task.

- [x] **Step 4: Run dependency checks**

Run:

```bash
python3 scripts/harness/check-cargo-deps.py
cargo test -p silica-decode --features core-image-raw-probe
```

Expected: dependency docs pass and the feature build still passes because no code path uses the optional dependencies yet.

- [x] **Step 5: Commit Task 12.1.1**

Run:

```bash
git add crates/silica-decode/Cargo.toml docs/DEPENDENCIES.md
git commit -m "chore(decode): gate core image raw probe"
```

Expected: commit only dependency/feature gate files if implementation types are deferred to Task 2.

## Task 2: Implement Task 12.1 Probe Type Contract

**Files:**
- Modify: `crates/silica-decode/src/lib.rs`
- Create: `crates/silica-decode/src/core_image_raw_probe.rs`

- [ ] **Step 1: Write the failing probe contract test**

Add this test to `crates/silica-decode/src/lib.rs`:

```rust
#[cfg(test)]
mod raw_probe_contract_tests {
    #[test]
    fn core_image_raw_probe_contract_does_not_change_preview_readiness() {
        let unavailable =
            crate::probe_core_image_raw(crate::RawProbeRequest::new("/tmp/missing.dng"));
        assert_eq!(unavailable.backend, crate::RawProbeBackend::CoreImageRaw);
        assert!(matches!(
            unavailable.status,
            crate::RawProbeStatus::Unavailable | crate::RawProbeStatus::Failed
        ));

        let raw_plan = crate::plan_preview_decode("/tmp/sample.dng", false);
        assert_eq!(
            raw_plan.status,
            crate::PreviewDecodeStatus::BlockedByMissingRawFixtureProbe
        );
    }
}
```

Run:

```bash
cargo test -p silica-decode core_image_raw_probe_contract_does_not_change_preview_readiness
```

Expected: compile failure for missing `RawProbeRequest`, `probe_core_image_raw`, `RawProbeBackend`, and `RawProbeStatus`.

- [ ] **Step 2: Add probe contract types**

Add these public types to `crates/silica-decode/src/lib.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawProbeRequest {
    pub source_path: String,
    pub expected_sha256: Option<String>,
}

impl RawProbeRequest {
    pub fn new(source_path: impl AsRef<str>) -> Self {
        Self {
            source_path: source_path.as_ref().to_string(),
            expected_sha256: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawProbeBackend {
    CoreImageRaw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawProbePlatform {
    Macos,
    UnsupportedPlatform,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawProbeStatus {
    Success,
    Unsupported,
    Failed,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawProbeErrorCategory {
    UnsupportedPlatform,
    MissingFile,
    SourceHashMismatch,
    CoreImageUnavailable,
    CoreImageOpenFailed,
    CoreImageMetadataMissing,
    PermissionDenied,
    InvalidFixture,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawProbeResult {
    pub backend: RawProbeBackend,
    pub platform: RawProbePlatform,
    pub macos_version: Option<String>,
    pub source_path: String,
    pub source_sha256: Option<String>,
    pub original_file_size: Option<u64>,
    pub original_modified_at: Option<String>,
    pub status: RawProbeStatus,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub orientation: Option<i32>,
    pub error_category: Option<RawProbeErrorCategory>,
    pub message: String,
}
```

- [ ] **Step 3: Add stable fallback probe function**

Add this public function:

```rust
pub fn probe_core_image_raw(request: RawProbeRequest) -> RawProbeResult {
    core_image_raw_probe::probe_core_image_raw(request)
}
```

Create `crates/silica-decode/src/core_image_raw_probe.rs` with an unsupported fallback:

```rust
use crate::{
    RawProbeBackend, RawProbeErrorCategory, RawProbePlatform, RawProbeRequest, RawProbeResult,
    RawProbeStatus,
};

pub fn probe_core_image_raw(request: RawProbeRequest) -> RawProbeResult {
    RawProbeResult {
        backend: RawProbeBackend::CoreImageRaw,
        platform: RawProbePlatform::UnsupportedPlatform,
        macos_version: None,
        source_path: request.source_path,
        source_sha256: None,
        original_file_size: None,
        original_modified_at: None,
        status: RawProbeStatus::Unavailable,
        width: None,
        height: None,
        orientation: None,
        error_category: Some(RawProbeErrorCategory::UnsupportedPlatform),
        message: "Core Image RAW probe is unavailable on this platform or feature build.".to_string(),
    }
}
```

Add `mod core_image_raw_probe;` in `lib.rs`.

- [ ] **Step 4: Run tests**

Run:

```bash
cargo test -p silica-decode
cargo test -p silica-decode --features core-image-raw-probe
```

Expected: tests pass, even before true Core Image metadata extraction is implemented.

- [ ] **Step 5: Commit Task 12.1.2**

Run:

```bash
git add crates/silica-decode/src/lib.rs crates/silica-decode/src/core_image_raw_probe.rs
git commit -m "feat(decode): add raw probe contract"
```

## Task 3: Implement Task 12.1 macOS Core Image Backend

**Files:**
- Modify: `crates/silica-decode/src/core_image_raw_probe.rs`

- [ ] **Step 1: Add macOS feature cfg split**

Refactor `core_image_raw_probe.rs` into:

```rust
#[cfg(all(target_os = "macos", feature = "core-image-raw-probe"))]
mod platform;

#[cfg(all(target_os = "macos", feature = "core-image-raw-probe"))]
pub use platform::probe_core_image_raw;

#[cfg(not(all(target_os = "macos", feature = "core-image-raw-probe")))]
pub fn probe_core_image_raw(request: crate::RawProbeRequest) -> crate::RawProbeResult {
    crate::RawProbeResult {
        backend: crate::RawProbeBackend::CoreImageRaw,
        platform: crate::RawProbePlatform::UnsupportedPlatform,
        macos_version: None,
        source_path: request.source_path,
        source_sha256: None,
        original_file_size: None,
        original_modified_at: None,
        status: crate::RawProbeStatus::Unavailable,
        width: None,
        height: None,
        orientation: None,
        error_category: Some(crate::RawProbeErrorCategory::UnsupportedPlatform),
        message: "Core Image RAW probe is unavailable on this platform or feature build.".to_string(),
    }
}
```

Create `crates/silica-decode/src/core_image_raw_probe/platform.rs` for the macOS implementation.

- [ ] **Step 2: Implement file metadata and hash preservation first**

Before Core Image calls, implement:

```rust
let before = std::fs::metadata(&path)?;
let source_sha256 = sha256_file(&path)?;
```

Implement `sha256_file` with the documented optional `sha2` dependency:

```rust
fn sha256_file(path: &std::path::Path) -> Result<String, std::io::Error> {
    use sha2::{Digest, Sha256};

    let mut file = std::fs::File::open(path)?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = std::io::Read::read(&mut file, &mut buffer)?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(format!("{:x}", digest.finalize()))
}
```

Do not reuse the existing FNV-style test hash helpers for fixture evidence; they are not SHA-256.

- [ ] **Step 3: Implement Core Image metadata probe**

Use `CIImage` or `CIRAWFilter` through `objc2-core-image` to open the file URL and read extent/dimensions where available. Map failures to:

```rust
RawProbeErrorCategory::CoreImageOpenFailed
RawProbeErrorCategory::CoreImageMetadataMissing
RawProbeErrorCategory::PermissionDenied
RawProbeErrorCategory::Unknown
```

No decoded pixel buffer should be returned.

- [ ] **Step 4: Run macOS feature tests**

Run:

```bash
cargo test -p silica-decode --features core-image-raw-probe
```

Expected: probe contract tests pass. If no real fixture is present, tests should cover missing/unavailable paths only.

- [ ] **Step 5: Commit Task 12.1.3**

Run:

```bash
git add crates/silica-decode/src/core_image_raw_probe.rs crates/silica-decode/src/core_image_raw_probe/platform.rs crates/silica-decode/src/lib.rs docs/DEPENDENCIES.md
git commit -m "feat(decode): add core image raw probe"
```

## Task 4: Implement Task 12.2 Fixture Probe Harness

**Files:**
- Create or modify: `crates/silica-decode/src/raw_probe_fixture.rs`
- Modify: `crates/silica-decode/src/lib.rs`
- Create: `scripts/harness/check-raw-probe-fixtures.py` if a Python wrapper is useful
- Modify: `docs/wiki/tasks/12.2-raw-fixture-probe-harness.md`

- [ ] **Step 1: Add ignored fixture test**

Add an ignored test in `silica-decode`:

```rust
#[test]
#[ignore]
fn probes_raw_fixture_manifest_without_mutating_originals() {
    let manifest = std::env::var("SILICARAW_RAW_FIXTURE_MANIFEST")
        .expect("SILICARAW_RAW_FIXTURE_MANIFEST must point to a legal RAW fixture manifest");
    let report = crate::probe_raw_fixture_manifest(manifest)
        .expect("probe legal RAW fixture manifest");
    assert!(!report.results.is_empty());
    assert!(report.results.iter().all(|result| result.original_hash_unchanged));
}
```

Expected initial failure: `probe_raw_fixture_manifest` and report types do not exist.

- [ ] **Step 2: Add manifest probe report types**

Add types:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawFixtureProbeReport {
    pub manifest_path: String,
    pub results: Vec<RawFixtureProbeResult>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawFixtureProbeResult {
    pub fixture_id: String,
    pub fixture_class: String,
    pub relative_path: String,
    pub probe: RawProbeResult,
    pub original_hash_unchanged: bool,
}
```

- [ ] **Step 3: Implement minimal manifest loading**

Add direct `serde_json = "1.0.150"` to `crates/silica-decode/Cargo.toml` if the Rust fixture loader parses manifests, and update `docs/DEPENDENCIES.md` for direct `silica-decode` use. Keep it behind normal dependencies only if the public fixture report API needs it outside the feature; otherwise make it optional behind `core-image-raw-probe`.

The loader must reject:

```txt
absolute paths
dot segments
missing expected_source_hashes
fixtures without raw kind
```

- [ ] **Step 4: Run ignored fixture test with a real local manifest**

Run:

```bash
SILICARAW_RAW_FIXTURE_MANIFEST=/absolute/path/to/local/raw-fixtures.json cargo test -p silica-decode --features core-image-raw-probe -- --ignored
```

Expected: pass only with a legal local fixture manifest. If no legal fixtures exist, stop and report the blocked condition instead of fabricating samples.

- [ ] **Step 5: Commit Task 12.2.1**

Run:

```bash
git add crates/silica-decode/src/lib.rs crates/silica-decode/src/raw_probe_fixture.rs scripts/harness/check-raw-probe-fixtures.py docs/DEPENDENCIES.md
git commit -m "test(decode): add raw fixture probe harness"
```

## Task 5: Implement Task 12.3 Support Matrix and LibRaw Gate

**Files:**
- Modify: `docs/wiki/topics/raw-decoding.md`
- Create: `docs/wiki/decisions/adr-0009-raw-decoder-support-matrix.md` if a durable decision is needed
- Modify: `docs/wiki/tasks/12.3-core-image-support-matrix.md`
- Modify: `docs/wiki/log.md`

- [ ] **Step 1: Add support matrix section**

Add this table shape to `docs/wiki/topics/raw-decoding.md`:

```markdown
## Phase 12 Core Image Support Matrix

| Fixture class | Fixture id | Format | Backend | Probe status | Dimensions | Orientation | Product status | Evidence |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| A | example | DNG | core_image_raw | success | known | unknown | core_image_supported | local probe report |
```

Replace `example` rows with actual Task 12.2 evidence. If no legal fixture evidence exists, use a blocked row that says `blocked_pending_evidence`.

- [ ] **Step 2: Record LibRaw gate**

If Core Image covers the legal fixture set, add:

```markdown
LibRaw remains deferred. No fixture-backed Core Image gap has been recorded.
```

If Core Image fails a required fixture class, create ADR 0009 before adding any dependency.

- [ ] **Step 3: Validate docs and dependency guard**

Run:

```bash
python3 scripts/harness/check-md-links.py
python3 scripts/harness/check-cargo-deps.py
```

Expected: both pass.

- [ ] **Step 4: Commit Task 12.3**

Run:

```bash
git add docs/wiki/topics/raw-decoding.md docs/wiki/decisions docs/wiki/tasks/12.3-core-image-support-matrix.md docs/wiki/log.md
git commit -m "docs(raw): record core image support matrix"
```

## Task 6: Implement Task 12.4 Product RAW Decode API Contract

**Files:**
- Modify: `crates/silica-decode/src/lib.rs`
- Modify: `crates/silica-core/src/lib.rs`
- Modify: `docs/wiki/tasks/12.4-product-raw-decode-api-contract.md`
- Modify: `docs/wiki/log.md`

- [ ] **Step 1: Add decode API contract types in silica-decode**

Add:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductRawDecodeStatus {
    Supported,
    BlockedPendingEvidence,
    BlockedCoreImageFailed,
    BlockedUnsupportedClass,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductRawDecodePlan {
    pub source_path: String,
    pub backend: RawProbeBackend,
    pub status: ProductRawDecodeStatus,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub orientation: Option<i32>,
    pub message: String,
}
```

- [ ] **Step 2: Add core wrapper**

Add a thin `silica-core` function:

```rust
pub fn plan_product_raw_decode(
    source_path: impl AsRef<str>,
) -> silica_decode::ProductRawDecodePlan {
    silica_decode::plan_product_raw_decode(source_path)
}
```

The wrapper must not read storage, create cache, or render pixels in this task.

- [ ] **Step 3: Add tests**

Add tests proving:

```rust
let plan = silica_decode::plan_product_raw_decode("/tmp/sample.dng");
assert_ne!(plan.status, silica_decode::ProductRawDecodeStatus::Supported);
```

until Task 12.3 evidence maps a fixture class to supported.

- [ ] **Step 4: Run validation**

Run:

```bash
cargo test -p silica-decode -p silica-core
scripts/harness/check.sh
```

Expected: pass.

- [ ] **Step 5: Commit Task 12.4**

Run:

```bash
git add crates/silica-decode/src/lib.rs crates/silica-core/src/lib.rs docs/wiki/tasks/12.4-product-raw-decode-api-contract.md docs/wiki/log.md
git commit -m "feat(raw): add product decode contract"
```

## Review Checklist

Before declaring Phase 12 complete:

- [ ] `core-image-raw-probe` is non-default.
- [ ] Default `scripts/harness/check.sh` passes.
- [ ] Direct dependencies added to `silica-decode` are documented.
- [ ] Legal fixture evidence exists for any support claim.
- [ ] Original hash preservation is checked during fixture probes.
- [ ] LibRaw is still deferred or backed by a concrete ADR.
- [ ] No UI RAW pixels are wired.
- [ ] No color correctness claim is made.

## Execution Notes

Use code-review graph after each implementation task:

```txt
detect changes against HEAD
review changed files and affected flows
run the task validation
commit atomic result
```

Do not run fixture probes with user photos or unlicensed samples. Use only legal local fixture manifests.
