---
title: LUT and Video Service Master Plan
status: draft
audience: agents
updated: 2026-07-11
source_of_truth: docs/wiki/roadmaps/lut-video-service-master-plan.md
---

# LUT and Video Service Master Plan

## Summary

This page is the draft execution router for the service-capable pre-v1 track: Phase 29 through Phase 36. The product remains a local-first macOS desktop editor; "service-capable" does not mean SaaS, a hosted service, or a network dependency.

The required baseline is Phases 29-32. RAW and video claims require Phases 33 and 35 respectively; deterministic local AI assistance in Phase 34 is optional.

```txt
service foundation hardening
-> float color chain (single source of color truth)
-> manual LUT creation (.cube export)
-> LUT import and apply (looks)
-> RAW product enablement (parallel-capable)
-> AI-assisted LUT creation (local-only)
-> video foundation (LUT-first scope, not a timeline editor)
-> service release gate
```

This plan continues the numbering of the [Post-Alpha Master Execution Plan](post-alpha-master-execution-plan.md), which ends at Phase 28. Task 27.2 is blocked on Developer ID funding, signing/notarization credentials and artifacts, checksums, and clean-Mac downloaded-artifact QA. Phase 28 has not started because it waits for public-beta feedback and a v1.0 scope freeze; it is not blocked on funding alone. Neither gate is duplicated or bypassed here.

## Operating Rules

- Use this page before choosing any Phase 29 or later task.
- Keep frontmatter `status: draft` until the Task 29.0 ADR accepts the charter; only then may a docs-only update mark this plan active.
- One task is one atomic, committable unit: one PR, smallest reviewable scope.
- When a phase starts, create its task cards under `docs/wiki/tasks/` from this page in a docs-only task first (`29.0`, `29.1`, ... follow the existing card template: Goal, Read Before Work, Files, Scope, Acceptance Criteria, Validation, Stop Gates, Completion State) and link them from `docs/wiki/tasks/index.md`.
- Creating cards does not activate the track or change `docs/wiki/llm/current-route.md`. Update the current route only when Task 29.0 accepts the charter; keep Q6.3/Q6.4 and release gates visible after that update.
- Every `X.0` task is a design gate. Do not start `X.1+` before the `X.0` decision is recorded (ADR under `docs/wiki/decisions/` when it changes architecture, schema, or dependencies).
- All existing hard rules stay in force: never modify original files, no network/telemetry/cloud by default, document every new dependency in `docs/DEPENDENCIES.md`, do not invent edit graph structure outside `schemas/edit_graph.schema.json`, use design tokens for UI.
- Do not claim visual color correctness beyond recorded evidence. LUT claims follow the same evidence discipline as export color claims.

## Current Position (facts this plan builds on)

As of 2026-07-11:

- The working product is a raster (JPEG/PNG/TIFF) develop-and-export alpha. RAW decode is not in the default build; supported import extensions are raster-only (`crates/silica-catalog/src/lib.rs:97`).
- The only real pixel math is the CPU chain in `crates/silica-export/src/lib.rs:1337-1596`. It handles encoded RGB8 bytes and re-quantizes after stages, but it has no explicit input profile/transfer transform proving that those bytes are sRGB. This chain is legacy behavior and parity evidence, not the working-space authority.
- Detail sharpening and noise reduction are persisted as edit state but never applied to pixels (no `apply_detail`/`apply_noise` exists in `silica-export`); the Develop Detail controls are already disabled and labeled unsupported, and export is blocked for non-neutral stored Detail state.
- Develop preview does a full open-decode-adjust-encode-write-read disk round-trip per slider event with no debounce.
- `silica-export` profile lookup and `silica-render` color-probe tests hardcode macOS ColorSync profile paths (`/System/Library/ColorSync/Profiles/...`); portable ICC fixtures and Linux coverage are missing.
- `silica-render` is a routing contract (no rendering); `native_metal_viewer` is a shell (no Metal calls); `silica-mlx` has no ML runtime.
- God files: `crates/silica-core/src/lib.rs` (~13.3k lines), `crates/silica-storage/src/lib.rs` (~10.6k), `apps/desktop/src-tauri/src/main.rs` (~9.3k), `apps/desktop/static/index.html` (~8k with one inline script).
- There is no LUT, `.cube`, or video code anywhere in the repository.

## Track Map

| Wave | Scope | Entry Gate | Exit Gate |
| --- | --- | --- | --- |
| A | Phase 29: Service Foundation Hardening | Maintainer direction | Honest UI claims, portable tests, modular crates/frontend, in-memory preview, CSP enabled |
| B | Phase 30: Float Color Chain | All Phase 29 tasks complete | Linear Display P3 f32 CPU/export semantics and default Metal product preview agree within recorded tolerance |
| C | Phase 31: Manual LUT Export | Phase 30 complete | User exports a `.cube` from any edit state; round-trip tolerance evidence recorded |
| D | Phase 32: LUT Import and Apply | Phase 31 complete | Imported LUTs preview/commit/undo like any edit; schema extension recorded |
| E | Phase 33: RAW Product Enablement | Phase 30 complete (then parallel to C/D) | RAW files import, develop, and export on macOS product builds |
| F | Phase 34: AI-Assisted LUT | Phase 31 complete; Phase 32 recommended | Local-only reference match produces approvable suggestions and one-click LUT bake |
| G | Phase 35: Video Foundation | Phases 31 and 32 complete | Video imports, previews, and exports with a LUT applied; originals untouched |
| H | Phase 36: Service Release Gate | Phases 29-32 complete; Phase 33/35 complete for RAW/video claims; Phase 34 optional | Claims audit, capability audit, current-identity review, evidence index, inherited release gates reconciled |

Dependency notes:

- Phase 33 (RAW) starts only after Phase 30 is complete and can then run in parallel with Phases 31-32 by a second agent; RAW develop/export must consume the completed f32 chain as the only color path.
- Phase 35 (video) consumes the `.cube` parser/applier from Phase 31 and the LUT edit-state extension from Phase 32; do not start it earlier.
- Phase 34 is optional and must not add network access. Cloud inference is out of scope unless a maintainer changes the no-network rule explicitly.

---

## Phase 29: Service Foundation Hardening

Goal: remove debt that blocks every later phase — UI truth regressions, macOS-hardcoded tests, god files, and the disk-round-trip preview.

Task dependency DAG (no partial Phase 29 exit):

| Task | Depends on |
| --- | --- |
| 29.0 | Maintainer direction |
| 29.1, 29.2, 29.3, 29.4, 29.7 | 29.0 |
| 29.5 | 29.3, 29.4 |
| 29.6 | 29.5 |
| 29.8 | 29.7 |
| 29.9 | 29.7 |
| 29.10 | 29.3, 29.5 |
| 29.11 | 29.10 |
| 29.12 | 29.8, 29.10 |

Tasks on separate DAG branches may run in parallel, but every Task 29.1-29.12 is blocked until 29.0 is accepted and Phase 30 is blocked until every Phase 29 task is complete.

### Task 29.0: Service Direction Design Gate

- Goal: record the track charter before implementation. SilicaRAW remains a local-first macOS desktop application, not SaaS or a network service. This service-capable pre-v1 track may extend feature scope, but it never bypasses Q6.3, Task 27.2, or any Phase 28 release gate. Required baseline scope is Phases 29-32; RAW and video claims require completion of Phases 33 and 35 respectively; Phase 34 deterministic local AI is optional. Product identity remains `SilicaRAW` with bundle identifier `dev.silicaraw.desktop`; no fork, rename, or rebrand is assumed. CSP hardening is required.
- Files: `docs/wiki/decisions/`, this plan.
- Acceptance: ADR accepted; all charter requirements above are explicit; Task 29.1+ remains blocked until acceptance; plan status changes from `draft` to `active` only after acceptance; no code change.

### Task 29.1: Detail UI Regression and Truth Audit

- Goal: verify the UI continues to state truthfully that sharpening/noise-reduction do not affect pixels while `silica-export` never applies them.
- Files: `apps/desktop/static/index.html`, `apps/desktop/src-tauri/src/main.rs`, `docs/wiki/topics/`.
- Scope: preserve the disabled Detail controls and unsupported messaging; audit frontend events and IPC wiring; remove any remaining callable UI path only if the audit finds one; keep stored edit state readable; document the gap.
- Acceptance: Detail controls remain disabled and clearly marked unsupported; no frontend interaction can preview or commit Detail edits that the pixel pipeline ignores; existing detail edit-state tests still pass.
- Stop gate: do not implement sharpening/NR here; that is future work after Phase 30.

### Task 29.2: ICC Profile Portability

- Goal: remove `/System/Library/ColorSync/Profiles/*.icc` hard dependencies from `silica-export` profile lookup and `silica-render` color-probe tests.
- Files: `crates/silica-export`, `crates/silica-render`, `docs/DEPENDENCIES.md` if a profile-generation dependency is added (prefer none).
- Scope: bundle minimal known-good sRGB and Display P3 ICC byte tables as repo assets (license-checked) or generate them; make profile-byte tests platform-neutral while keeping macOS system profiles as an optional override; record profile SHA-256 in export results unchanged.
- Acceptance: `cargo test -p silica-export` and `cargo test -p silica-render --features color-probe` pass on Linux; macOS behavior and embedded-ICC evidence unchanged.

### Task 29.3: Modularize silica-export

- Goal: split `crates/silica-export/src/lib.rs` into modules (e.g. `ops/`, `encode/`, `metadata/`, `requests/`) with zero behavior change.
- Acceptance: public API unchanged (re-exports allowed); `cargo test -p silica-export` passes; no logic edits in the same PR.

### Task 29.4: Modularize silica-storage

- Goal: same mechanical split for `crates/silica-storage/src/lib.rs` (e.g. `schema/`, `migrations/`, `photos/`, `edits/`, `exports/`, `sessions/`).
- Acceptance: public API unchanged; all storage tests pass.

### Task 29.5: Modularize silica-core

- Goal: same mechanical split for `crates/silica-core/src/lib.rs` (e.g. `library/`, `develop/`, `export/`, `session/`, `permissions/`).
- Acceptance: public API unchanged; all core tests pass.

### Task 29.6: Modularize Desktop main.rs

- Goal: split `apps/desktop/src-tauri/src/main.rs` into `commands/` and `dto/` modules; keep the single `generate_handler!` registration list in `main.rs`.
- Acceptance: command names and payload shapes unchanged; desktop tests pass.

### Task 29.7: Extract Frontend Modules

- Goal: move the ~6.6k-line inline script out of `apps/desktop/static/index.html` into ES modules under `apps/desktop/static/js/` (per-screen modules: library, develop, export, session, ipc).
- Scope: mechanical extraction, `<script type="module">`, no framework, no behavior change, no bundler.
- Acceptance: all screens function as before (manual QA per `checklists/QA_CHECKLIST.md` smoke subset); `index.html` contains markup only.

### Task 29.8: Frontend State Isolation

- Goal: replace module-level mutable globals with one explicit state object per screen module plus a small shared store; document the state shape.
- Acceptance: no implicit cross-module globals; stale-preview sequence guard behavior preserved.

### Task 29.9: Enable CSP and Capability Audit

- Goal: replace `"csp": null` in `apps/desktop/src-tauri/tauri.conf.json` with a strict policy compatible with blob: preview URLs and module scripts; re-verify `capabilities/default.json` stays minimal.
- Acceptance: app functions with CSP enforced; capability file diff reviewed and documented.

### Task 29.10: In-Memory Develop Preview

- Goal: return develop preview JPEG bytes without the write-to-disk-then-read-back round-trip in `silica-core`/`silica-export`.
- Files: `crates/silica-export` (encode-to-buffer entry point), `crates/silica-core` (preview path).
- Acceptance: preview bytes identical (hash) to the previous disk path for fixture inputs; disk cache write becomes optional/explicit; original-safety tests pass.

### Task 29.11: Decoded Source Preview Cache

- Goal: cache the decoded, downscaled working raster per photo session so slider changes re-run only the color chain, not file open + decode.
- Files: `crates/silica-core`.
- Scope: bounded in-memory cache keyed by photo id + source fingerprint; invalidate on file change; respect existing cache-clear command.
- Acceptance: repeated previews on the same photo do not re-open the source file (assert via test seam); memory bound documented.

### Task 29.12: Develop Slider Debounce

- Goal: measure preview latency and coalesce rapid slider IPC at the smallest bounded interval that preserves the existing <50 ms perceived-response target; do not hardcode a speculative trailing delay. Keep the monotonic sequence guard for stale-response drops.
- Files: `apps/desktop/static/js/`.
- Acceptance: timing evidence justifies the selected coalescing policy; rapid drag produces bounded IPC calls; the final value always renders; the sequence guard remains active; manual QA records perceived response below the existing target.

Phase exit gate: all tasks complete; full workspace tests green on Linux and macOS evidence recorded.

---

## Phase 30: Float Color Chain

Goal: make the documented linear Display P3 pipeline authoritative: a dependency-free CPU reference/export implementation in `silica-color` and a parity-tested Metal implementation for the product preview. This is the foundation for LUT baking (Phase 31), LUT application (Phase 32), RAW (Phase 33), optional AI matching (Phase 34), and video (Phase 35).

Required flow:

```txt
resolved source profile/transfer contract
-> linear Display P3 f32 working raster
-> global color operations
-> spatial geometry outside the global/LUT chain
-> f32 manual-mask local compositor in post-geometry coordinates
-> explicit export or display transform
-> quantization at the declared output boundary only
```

Task dependencies: `30.0 -> 30.1/30.2`; `30.1 + 30.2 -> 30.3`; Tasks 30.4-30.9 branch after 30.1; `30.3 + 30.4-30.9 -> 30.10 -> 30.11 -> 30.12 -> 30.13 -> 30.14 -> 30.15`. Task 30.16 switches CPU consumers; `30.12 + 30.15 -> 30.17 -> 30.18`; Tasks 30.16 and 30.18 both block 30.19.

### Task 30.0: Color Chain Design Gate

- Goal: ADR fixing the flow above, supported input profile/transfer pairs and untagged policy, linear Display P3 numeric/clamping semantics, global/local/spatial composition order, output/display transforms, histogram domain, CPU/Metal tolerance policy, and fail-closed behavior for unknown color contracts. The current encoded RGB8 path is a legacy reference, not proof of sRGB correctness. `silica-color` owns CPU reference and export semantics; the product preview remains Metal-first. The single-quantization claim applies only to the final output boundary, never to source decoding or intermediate stages.
- Acceptance: ADR accepted; crate and shader ownership recorded; global LUT-bakeable operations are separated from manual masks and photo geometry; no release path may defer the Phase 30 Metal preview work.

### Task 30.1: Create silica-color Crate

- Goal: workspace member `crates/silica-color` with finite f32 RGB/working-raster types, typed input/output color contracts, global/local parameter blocks, and identity defaults. No ops, serialization, I/O, GPU code, or edit-graph mapping yet; serialization and mapping stay in `silica-core`.
- Acceptance: builds in workspace; `silica-color` has no dependencies; README states CPU reference/export responsibility; `docs/DEPENDENCIES.md` untouched.

### Task 30.2: Resolve and Backfill Source Color Contracts

- Goal: inspect embedded ICC/profile tags at import/decode boundaries, apply the explicit Task 30.0 untagged policy, and persist the resolved input profile, transfer identity, decoder backend, and `linear_display_p3` working-space evidence through the schema-owned edit-graph `profile` path. Add an idempotent, resumable existing-library backfill keyed by source fingerprint; normal grid/filter/metadata queries read persisted state and never reopen originals. Unknown, malformed, or unsupported source color contracts remain typed blocked states.
- Files: `crates/silica-core`, `crates/silica-storage`, `crates/silica-edit`, import/decode boundaries.
- Acceptance: tagged sRGB/Display P3, untagged-policy, malformed ICC, unknown profile, existing-library upgrade, resume, and source-change invalidation tests; repeated queries perform no original-file reads; Develop/export cannot proceed while the persisted contract is unknown.

### Task 30.3: Input Profile and Transfer Transform

- Goal: transform decoded encoded RGB plus the persisted Task 30.2 contract into a linear Display P3 f32 working raster. No transform may infer sRGB from RGB8 storage or reread source metadata independently.
- Acceptance: reference vectors and fixtures cover primaries, transfer decoding, neutral/extended values, finite output, and all accepted source contracts; unknown or stale contracts block.

### Tasks 30.4-30.9: f32 Global Operations (one task per op)

- 30.4 exposure/contrast, 30.5 white balance, 30.6 tone recovery, 30.7 tone curve (+ curve evaluation), 30.8 color presence (vibrance/saturation), 30.9 HSL mixer (+ RGB/HSL conversion defined by the 30.0 ADR).
- Goal (each): pure finite-f32 operation over linear Display P3 data, preserving the accepted operation order and avoiding intermediate integer conversion.
- Acceptance (each): identity, known-vector, bounds/extended-range, determinism, and non-finite rejection tests; the corresponding legacy 8-bit function remains unchanged until Task 30.15 evidence.

### Task 30.10: f32 Spatial Geometry

- Goal: apply the supported crop/rotation/flip/transform subset to the linear Display P3 f32 raster after global color operations and before masks. Transform durable normalized mask geometry/brush coverage into the resulting image coordinates; geometry never enters `apply_global_color_chain` or a baked LUT.
- Acceptance: identity, crop, rotation, flip, supported transform, bounds, dimension, and geometry-plus-mask coordinate fixtures pass without RGB8 conversion; unsupported non-neutral geometry blocks rather than disappearing.

### Task 30.11: f32 Manual-Mask Local Compositor

- Goal: port the currently supported manual-mask exposure/contrast compositor, including transformed gradient/brush coverage, invert, opacity, feather, and overlap semantics, to the post-geometry f32 working raster. Unsupported local adjustment keys remain typed blocked states.
- Acceptance: identity and synthetic-mask tests cover post-crop/rotate coordinates, edge weights, brush alpha, overlapping masks, operation order, and finite output without RGB8 round trips.

### Task 30.12: Export and Display Output Transforms

- Goal: transform the post-mask linear Display P3 f32 raster to explicit sRGB/Display P3 export encodings or an explicit active-display contract without quantizing inside the transform.
- Acceptance: reference vectors and ICC-backed fixtures cover export and display contracts; only the encoder/drawable output boundary quantizes; unknown or unsupported destinations block.

### Task 30.13: Compose the CPU Develop Pipeline

- Goal: compose source contract → input transform → global chain → spatial geometry → manual masks → output transform. Keep `apply_global_color_chain` independently typed for LUT baking and exclude geometry/masks by construction.
- Acceptance: identity, exact stage-order, dimension/coordinate, determinism, non-finite, and fail-closed tests; no alternate CPU stage order exists in production.

### Task 30.14: Edit Graph Field Mapping and Exclusion Audit

- Goal: maintain an exhaustive path-level table generated against every v1 schema field and classify each as input-contract mapped, global mapped, spatial mapped, local-mask mapped, unsupported, or non-pixel excluded. It must cover `source`, every `profile` field, every `basic` field, all tone curves, every HSL/color-grading field, all detail/lens/geometry fields, every mask/source/geometry/brush/local field, metadata, timestamps, and extensions. Non-neutral texture, clarity, dehaze, color grading, detail, unsupported lens values, unsupported geometry, and local-adjustment keys other than the accepted set return typed blockers; neutral unsupported values and non-pixel metadata/extensions appear in an explicit exclusion report.
- Acceptance: a schema-inventory test fails when any current or newly added v1 path lacks a classification; fixtures exercise every mapped family and every blocker above; no non-neutral value is silently dropped; `silica-color` gains no serde/edit-graph dependency.

### Task 30.15: Legacy Parity and Intentional-Difference Evidence

- Goal: compare the explicit-transform f32 CPU pipeline with the legacy RGB8 pipeline on tagged fixtures and every enabled global/spatial/local family. Freeze tolerances in Task 30.0 before switching; separate regressions from intentional differences caused by correct linear processing and removal of intermediate quantization.
- Files: `crates/silica-export` tests, `checklists/GOLDEN_IMAGE_TOLERANCE_POLICY.md`.
- Acceptance: per-stage and composed evidence recorded; every out-of-tolerance difference blocks or has an accepted color-semantics explanation; no parity result is presented as broad visual color-correctness proof.

### Task 30.16: Switch Export and Histogram to the CPU Reference

- Goal: make all raster export entry points and Develop histogram generation consume the accepted f32 composition and explicit output contract. Encoding performs the only claimed quantization; legacy RGB8 operations become test-only references or are removed after evidence.
- Acceptance: export/histogram tests and updated goldens pass; profile evidence and mapping/exclusion reports remain explicit; original-safety tests pass; no production export path applies encoded-byte operations.

### Task 30.17: Active Display ICC and Metal Surface Contract

- Goal: obtain the current window's `NSScreen` and ColorSync ICC identity, resolve the active display contract, configure the product `MTKView`/`CAMetalLayer` drawable color space, and refresh it whenever the window changes screens or the display profile changes. Never assume the built-in display or a static Display P3 profile.
- Acceptance: macOS tests/evidence cover initial screen resolution, movement between two differing display profiles, profile-change notification, drawable/layer reconfiguration, stale-request invalidation, and unknown/malformed display profile fail-closed behavior.

### Task 30.18: Implement Metal Shaders and CPU Preview Parity

- Goal: implement the Phase 14 product viewer's real Metal texture/shader path in the authoritative order: input transform → linear Display P3 global operations → spatial geometry → f32 masks → Task 30.17 active-display transform. Keep latest-request-wins and disposable texture ownership.
- Acceptance: macOS GPU readback/visual fixtures match the `silica-color` CPU reference within Task 30.0 tolerances for every enabled stage and composition; multi-display, resize, Retina, stale-request, blocked-field, and original-safety evidence pass; the spike module is not reused as product code.

### Task 30.19: Switch the Product Preview to Metal

- Goal: make the parity-proven native Metal viewer with active-display updates the default macOS Develop/Loupe preview path and retire the encoded JPEG/WebView round-trip as the authoritative product preview.
- Acceptance: default product builds exercise Metal preview, export remains CPU-reference/full-resolution and independent of viewer textures, unknown source/display contracts fail closed visibly, installed-app QA passes, and release remains blocked if active-display, Metal parity, or product-switch evidence is missing.

Phase exit gate: explicit input/output color contracts, one linear Display P3 f32 semantic pipeline, CPU export/histogram, and default Metal product preview all pass parity and installed-app evidence. GPU/Metal work cannot be deferred past this gate.

---

## Phase 31: Manual LUT Export (.cube)

Goal: a user can export the global, spatially invariant part of the current edit as an industry-standard 3D `.cube` LUT with an explicit linear Display P3 contract.

### Task 31.0: LUT Format Design Gate

- Goal: ADR for `.cube` specifics — supported `LUT_3D_SIZE` values 17/33/65 (default 33), normalized `DOMAIN_MIN 0 0 0` / `DOMAIN_MAX 1 1 1`, red-fastest then green then blue lattice serialization, linear Display P3 input/output working-domain contract, `TITLE` from photo/preset name, comments recording that otherwise non-standardized color contract plus app version and edit-state hash, finite 6-decimal values, LF line endings, and checked node-count/file-size bounds. Manual masks, photo geometry, detail, and every other spatial operation are never baked and must be reported.
- Acceptance: ADR accepted with a small hand-authored formatting/order golden; supported-size tests use node-count and sentinel-order assertions rather than checked-in full 33^3 output.

### Task 31.1: Cube Writer

- Goal: `write_cube(&Lut3d) -> String` in `silica-color` (pure formatting, no I/O, no new deps).
- Acceptance: a small synthetic lattice golden locks header/float/order semantics; generated 17/33/65 tests assert exact `size^3` row counts and sentinel lattice positions without full-file golden snapshots; non-finite values and unsupported sizes are rejected.

### Task 31.2: LUT Bake

- Goal: `bake_lut(&GlobalColorChainParams, size) -> (Lut3d, BakeExclusionReport)` iterating the normalized linear Display P3 lattice through `apply_global_color_chain`; the report lists every edit-graph family excluded (masks, geometry, detail) for UI display.
- Acceptance: identity params bake to an identity LUT (exact); non-identity spot nodes match direct chain evaluation; exclusion report tested.

### Task 31.3: Cube Parser and Validator

- Goal: `parse_cube(&str) -> Result<Lut3d, CubeError>` accepting writer output plus bounded common variants (comments, CRLF, optional DOMAIN lines), enforcing red-fastest/green/blue lattice interpretation, finite values, supported sizes 17/33/65, checked `size^3` arithmetic, exact row count, and a v1 maximum of 65^3 nodes before allocation.
- Acceptance: writer round trips preserve sentinel node order; small fixtures lock grammar and errors; malformed, non-finite, truncated, extra-row, unsupported-size, and oversized inputs fail with typed errors; no full 33^3 golden fixture is required.

### Task 31.4: Tetrahedral LUT Applier

- Goal: `apply_lut(rgb, &Lut3d) -> Rgb32` with tetrahedral interpolation and the Task 31.0 domain policy, in `silica-color`; input/output are linear Display P3 working values.
- Acceptance: identity LUT is passthrough within 1e-6; known-node exactness; fixtures cover each tetrahedron and shared boundaries without discontinuities.

### Task 31.5: Round-Trip Tolerance Proof

- Goal: bake → write → parse → apply on linear Display P3 fixture rasters versus direct global-chain evaluation; record max/mean error per LUT size.
- Acceptance: 33³ max error within the tolerance recorded in the 31.0 ADR (propose ≤ 1.5/255 for typical edits); evidence in the task card; failures block, not warn.

### Task 31.6: Core LUT Export API and Record

- Goal: add a safe `.cube` file writer in `silica-export` and orchestration in `silica-core`. Core loads and maps the active edit graph, bakes the LUT, rejects original-file destinations, calls the export writer, then records the completed export and logged-only action. The writer owns destination validation, sibling temporary-file write/flush, final rename, and partial-file cleanup. Use the existing `exports` table unchanged: its `export_settings_json` records a versioned evidence object with schema/version, `format: "cube"`, LUT size/domain, linear Display P3 contract, edit-state hash, exclusion report, and output hash.
- Acceptance: no catalog migration; successful file, `exports` row, and action-log row are asserted; failure leaves no completed record or partial output; originals remain unchanged; file content matches direct bake.

### Task 31.7: Desktop Command Wiring

- Goal: `export_photo_lut_cube` Tauri command + DTO, registered alongside existing export commands; save-dialog flow only (no new capabilities).
- Acceptance: command test coverage matching existing export command tests.

### Task 31.8: Export LUT UI

- Goal: Export panel section "Export LUT (.cube)" with size selector (17/33/65) and a non-dismissable notice listing excluded spatial edits from the bake report; design tokens only.
- Acceptance: manual QA per checklist; disabled state when photo has no committed edit state is defined and tested.

### Task 31.9: External Tool QA Checklist

- Goal: `checklists/LUT_EXPORT_MANUAL_QA.md` — load an exported LUT in at least two external tools configured for the declared linear Display P3 contract, compare against in-app render, record screenshots/hashes and setup limitations; add a harness check script for the checklist format.
- Acceptance: checklist merged with one recorded evidence run; harness check wired into `scripts/harness/check.sh`.

Phase exit gate: a real `.cube` exported from a real edit, verified in an external tool, evidence recorded.

---

## Phase 32: LUT Import and Apply

Goal: imported `.cube` LUTs become durable library assets and first-class, undoable edit state ("looks"), previewable in Develop.

### Task 32.0: Edit Graph LUT Extension Design Gate

- Goal: ADR fixing the v1 path as a versioned, namespaced entry under the existing `extensions` object, for example `extensions["silica.lut"] = { schema, version, library_lut_id, sha256, intensity }`, without a root edit-graph version bump. Define a managed, content-addressed library asset domain outside every disposable cache directory; catalog/sidecar/backup/restore and missing-asset behavior; chain position after HSL as the final global color op; and linear Display P3 intensity blending. If review rejects the extension path, stop Phase 32 and replace Tasks 32.1+ with an explicit v2 schema, deterministic v1-to-v2 migration, history/sidecar compatibility, and rollback plan before implementation.
- Acceptance: ADR accepted with the namespaced v1 extension path, or Phase 32 remains blocked on the replacement v2 migration plan; no task may invent an unversioned `extensions.lut` payload.

### Task 32.1: Schema and Edit Model Extension

- Goal: implement the accepted namespaced v1 extension contract in `silica-edit` with typed accessors and validation for schema/version, intensity 0..1, SHA-256, and library LUT identity, plus `apply_lut_reference`/`clear_lut_reference` mutators. Update the authoritative schema/example/reference docs only as required to describe the accepted extension contract; do not bump the root edit-graph version.
- Acceptance: example and reference docs updated; serde round-trip and validator tests reject unknown extension fields/versions and malformed identities while preserving unrelated namespaced extensions.

### Task 32.2: Library LUT Import and Catalog

- Goal: import a `.cube` file into the library as an atomic managed copy, SHA-256 verified via the 31.3 parser, with a new `luts` catalog table (migration): id, title, sha256, size, managed relative path, source filename, created_at.
- Files: `crates/silica-catalog`, `crates/silica-storage`, `crates/silica-core`, `docs/10_Data_Model_and_Storage_Specification.md`, `docs/19_Schema_Reference.md`.
- Acceptance: catalog schema version and required-table contract updated; fresh and every supported upgrade migration pass; duplicate import by hash is idempotent; catalog rows cannot point outside the managed LUT root; malformed files or failed copies leave no row/partial asset; action log records the import.

### Task 32.3: Managed LUT Durability

- Goal: integrate managed LUT assets with cache clear, backup/restore, sidecar recovery, and catalog validation. LUT assets are durable library data, not `cache_records` and not members of `DISPOSABLE_CACHE_DIRECTORIES`; backup and restore copy and hash-verify the managed assets together with their catalog rows.
- Acceptance: cache clear preserves LUT bytes and catalog rows; backup manifest includes assets and restore reproduces the catalog/asset pair; a missing asset, hash mismatch, or sidecar-only LUT reference without the managed asset yields a typed blocked state and never silently drops the look or reconstructs bytes from cache.

### Task 32.4: Chain Integration

- Goal: extend the global chain parameters with an optional resolved `Lut3d` + intensity; `apply_global_color_chain` applies it as the final global color op via the 31.4 applier in linear Display P3, before manual-mask local composition.
- Acceptance: identity LUT and intensity 0 are exact passthrough; chain property tests extended.

### Task 32.5: Preview, Commit, and History Wiring

- Goal: `preview_lut_edit` / `commit_lut_edit` / clear paths in `silica-core` + desktop commands, resolving `library_lut_id` to the hash-verified managed asset and optionally caching only its parsed runtime representation; history/undo/redo entries behave like any other edit family.
- Acceptance: undo/redo tests; missing/deleted LUT asset yields a typed blocked state (matching missing-file conventions), never a crash.

### Task 32.6: Develop LUT Panel

- Goal: Develop panel "Look / LUT" section — pick from imported LUTs, intensity slider, clear button; import entry point via file dialog; design tokens only.
- Acceptance: manual QA; blocked state for missing assets rendered per existing blocked-state patterns.

### Task 32.7: LUT Apply Evidence

- Goal: record parity evidence — exporting a photo with LUT applied equals chain+LUT reference within tolerance; extend `checklists/LUT_EXPORT_MANUAL_QA.md` with an import/apply section.
- Acceptance: evidence recorded; harness check updated.

Phase exit gate: import → preview → commit → undo → export all work with LUTs; cache clear and backup/restore preserve managed LUT assets; missing-asset behavior and parity evidence are recorded.

---

## Phase 33: RAW Product Enablement (parallel-capable)

Goal: the product finally decodes RAW on macOS builds using the already-selected Core Image path (Spike 002 decision stands; LibRaw remains deferred).

### Task 33.0: RAW Enablement Design Gate

- Goal: ADR for explicit macOS product feature activation and a high-depth, non-lossy, linear Display P3 working artifact matching `DecodedImagePixelFormat::Rgba16Float`, or an evidence-backed equivalent with no lossy intermediate. Define decode/render/export ownership, artifact format and metadata, disposable cache invalidation, and candidate extensions from Task 12.3 evidence. Extension and metadata-probe success are candidate states only; a file becomes supported only after validated high-depth artifact creation succeeds.
- Acceptance: ADR accepted; JPEG and the existing `JpegSrgb8` full-resolution proof artifact are excluded from the product working contract; profile/transfer/pixel-format evidence, feature activation, and every support-state transition are explicit; fixture policy is reaffirmed.

### Task 33.1: Enable Core Image Decode in macOS Product Builds

- Goal: explicitly enable `core-image-raw-probe` in macOS target product dependencies and macOS app/CI build commands. Do not claim or attempt a target-specific Cargo default feature; keep the underlying feature non-default for non-macOS consumers, which continue returning typed `Unavailable` states.
- Acceptance: default macOS product and developer-preview builds include the feature without ad hoc flags; Linux workspace builds/tests remain unaffected and contain no Apple framework linkage.

### Task 33.2: RAW Import Support Mapping

- Goal: classify matrix-listed RAW extensions as decode candidates, not supported photos. Import records a typed probe-pending state; metadata probe success advances only to artifact-pending, while failure records its typed blocked reason. Extension or probe success alone never produces a support claim.
- Files: `crates/silica-catalog`, `crates/silica-storage`, `crates/silica-core`.
- Acceptance: migration/import tests cover candidate, probe-pending, artifact-pending, and blocked states; grid behavior is explicit; no row is marked supported by extension or metadata probe alone.

### Task 33.3: High-Depth RAW Decode, Render, and Export Contract

- Goal: replace the proof-only `JpegSrgb8`/JPEG-sRGB handoff and render request with typed high-depth contracts across `silica-decode`, `silica-render`, `silica-export`, and `silica-core`. The full-resolution contract carries source identity, dimensions/orientation, input profile/transfer, linear Display P3 working space, `Rgba16Float`-compatible pixel format, and artifact hash; preview and JPEG/PNG/TIFF export plans consume it without an 8-bit lossy intermediate.
- Acceptance: contract/unit tests reject mismatched profile, pixel format, dimensions, source identity, and hash; no production RAW path references the proof-only JPEG artifact; non-macOS typed-unavailable behavior remains intact.

### Task 33.4: RAW Working Artifact Cache and Support Transition

- Goal: after metadata probe success, create and validate the Task 33.3 high-depth artifact through Core Image, track it in `cache_records`, and invalidate by source fingerprint plus decode contract. Promote the catalog row to supported only after artifact validation succeeds; otherwise persist a typed blocked reason.
- Acceptance: artifact reuse across sessions is asserted; cache clear removes it; support promotion is atomic with validated artifact evidence; failed/partial artifacts never promote support; original SHA-256 remains unchanged.

### Task 33.5: Develop on RAW Artifacts

- Goal: route Develop preview/commit and histogram for supported RAW photos through the cached Task 33.3 artifact into the Phase 30 linear Display P3 CPU/Metal contracts.
- Acceptance: end-to-end Develop on manifest RAW fixtures is recorded as macOS evidence; blocked/pending states remain visible; non-RAW paths are unchanged.

### Task 33.6: RAW Export Wiring

- Goal: export supported RAW photos to JPEG/PNG/TIFF from the Task 33.3 full-resolution high-depth contract through the Phase 30 CPU reference and explicit output transform, never from viewer textures or the proof-only JPEG path.
- Acceptance: all formats use full-resolution artifact evidence; export records retain RAW source/artifact/profile hashes; output alias protection and original-safety tests pass.

### Task 33.7: RAW Manual Color QA

- Goal: manual comparison of in-app RAW rendering versus Preview.app/Photos on the fixture manifest set; record deviations honestly (no correctness claims beyond evidence).
- Acceptance: checklist run recorded; README "Known Limitations" updated to reflect actual RAW status.

Phase exit gate: candidate import → metadata probe → validated high-depth artifact/support promotion → Metal Develop → full-resolution export works on macOS with evidence; RAW claims name only fixture-backed formats and failure states.

---

## Phase 34: AI-Assisted LUT Creation (local-only)

Goal: optionally create an "auto" LUT from a reference image under explicit suggestion/approval discipline. v1 is deterministic (no ML runtime, no network); a learned model is a stop-gated follow-up.

### Task 34.0: AI Assist Design Gate

- Goal: ADR reaffirming local-only operation (no network inference); v1 algorithm = deterministic reference matching (channel statistics + histogram matching → white balance + tone curve + saturation suggestion); output is normalized mutator input, never an edit graph or silent edit. Record that the current Phase 24 review path displays only `blur_score` and approves only `basic_exposure_contrast`, so color matching requires Tasks 34.2 and 34.4 before routing. MLX remains deferred behind 34.8.
- Acceptance: ADR accepted; optional scope, privacy boundary, contract ownership, approval transaction, and quality evidence gate are explicit.

### Task 34.1: Image Statistics Module

- Goal: `silica-color` functions for channel histograms, percentiles, and mean/std over a downsampled linear Display P3 f32 working raster.
- Acceptance: unit tests on synthetic images (uniform, gradient, two-tone); deterministic outputs.

### Task 34.2: Versioned Color-Match Suggestion Contract

- Goal: define and validate a nested approval payload with `schema: "silica.color_match_suggestion"`, `version: 1`, `kind: "reference_color_match"`, bounded temperature/tint, monotonic RGB tone-curve points, bounded saturation, and human-readable provenance summary. It is normalized input to an approved mutator inside the existing `silica.ai_result` envelope, not raw edit-graph state.
- Files: `crates/silica-core`, `crates/silica-storage`, `docs/19_Schema_Reference.md`.
- Acceptance: typed parse/validation tests reject unknown versions/kinds/fields, non-finite or out-of-range values, non-monotonic curves, and direct mutation payloads; the existing `basic_exposure_contrast` contract remains compatible.

### Task 34.3: Reference Match Estimator

- Goal: pure `(source_stats, reference_stats) -> ColorMatchSuggestionV1` implementation producing Task 34.2 payload values through bounded percentile/histogram matching.
- Acceptance: identical stats produce identity; curve monotonicity and all contract ranges hold under property tests; fixture-pair goldens are deterministic.

### Task 34.4: Color-Match Approval Mutator

- Goal: add one typed mutator that applies the approved v1 temperature/tint (custom white balance), RGB tone curve, and saturation atomically to a validated edit graph. Core approval commits exactly one undoable checkpoint, marks the result approved only after commit success, and records versioned provenance/action-log evidence; rejection remains log-only.
- Acceptance: approval, rejection, stale/already-approved result, validation failure, and transaction failure tests prove no partial edit/result state; originals and photo flags remain untouched.

### Task 34.5: Suggestion Routing and AI Review Extension

- Goal: `suggest_reference_match(photo_id, reference_path)` computes both statistics, stores a non-mutating v1 result in `ai_results`, and exposes `reference_color_match` through an explicitly extended AI Review read/approve/reject path. Do not route the new kind through the blur/basic-only parser.
- Acceptance: stored suggestion changes no edit/history state; explicit approval uses Task 34.4; rejection leaves edit state untouched; unreadable references and unsupported contract versions return typed blocked states.

### Task 34.6: Reference Match UI

- Goal: Develop action "Match reference..." (file dialog) surfaces the versioned suggestion in the AI Review panel with standard approve/reject controls and field-level proposed values.
- Acceptance: manual QA covers approval, rejection, unreadable reference, stale result, and no-network operation.

### Task 34.7: One-Click LUT from Suggestion

- Goal: after approval, offer "Export this look as LUT" by reusing the Phase 31 global-chain bake path on the approved edit state.
- Acceptance: end-to-end test: reference match → approve → bake → parse → apply parity within Phase 31 tolerances.

### Task 34.8: Learned Model Gate (stop-gated, optional)

- Goal: only if Task 34.7 quality evidence is recorded as insufficient — ADR for an MLX-based color-suggestion model behind a non-default feature gate, using the existing model-manifest validation (license, version, SHA-256) from `silica-mlx`; no model weights committed to the repo.
- Stop gate: do not start without a maintainer decision and a quality-gap record from 34.7 evidence.

Phase exit gate: reference-match → approval → LUT export demonstrated with recorded evidence; no network access added.

---

## Phase 35: Video Foundation (LUT-first scope)

Goal: videos import into the library, show thumbnails, play back, and export with a LUT applied. This is explicitly NOT a timeline editor: no cuts, no keyframes, no per-clip grading beyond one LUT + intensity in v1.

### Task 35.0: Video Scope and Look-State Design Gate

- Goal: ADR defining the catalog asset domain before import, including photo/video identity, metadata ownership, migration compatibility, and photo-only versus asset-aware query boundaries. Define a separate versioned `silica.video_look_state` v1 contract containing only video identity plus managed LUT id/hash/intensity; it is catalog-owned, mutated only through Core, undoable through typed video history, restored on reopen/backup, and has no v1 sidecar. Use AVFoundation/Core Image/VideoToolbox on macOS; v1 candidates are `.mov`/`.mp4` with H.264/HEVC. The color contract must distinguish Rec.709 full range from video range, decode into linear values, transform into Phase 30 linear Display P3, apply the LUT, then perform explicit display/export transforms. HDR, PQ, HLG, and log sources are typed blocked states. Scope remains one LUT + intensity: no timeline, cuts, transitions, keyframes, or per-frame edit automation.
- Acceptance: ADR accepted; asset schema/query migration, versioned look-state/history ownership, reopen/backup behavior, color metadata/probe rules, blocked HDR/log behavior, output tagging, and no-timeline boundary are explicit; `docs/DEPENDENCIES.md` plan for AVFoundation-related crates is recorded.

### Task 35.1: silica-video Crate Boundary

- Goal: new workspace crate `crates/silica-video` with typed request/result contracts and a macOS feature-gated AVFoundation probe: duration, fps, dimensions, codec, audio presence, color primaries, transfer function, full/video range, and HDR/log classification. Non-macOS returns typed `Unavailable`.
- Acceptance: probe evidence on a small self-generated fixture clip (generated in-test via AVFoundation or checked-in tiny clip with license note); Linux builds pass with stubs; DEPENDENCIES.md updated.

### Task 35.2: Catalog Asset-Domain Migration

- Goal: implement the Task 35.0 asset-domain decision, `video_metadata`, `video_look_states`, and typed video history. Rebuild or replace the current `file_type` constraint so accepted video values coexist with photo/raw/unsupported values. Update every count/select/filter/search/metadata query: photo APIs explicitly exclude video; only asset-aware APIs include/filter video. Preserve existing photo IDs, foreign keys, flags, edits, exports, and unsupported-state semantics through upgrade.
- Acceptance: fresh and every supported upgrade path assert expanded constraints, photo-only exclusion, asset-aware filtering, look-state/history foreign keys and indexes, reopen persistence, undo/redo, backup/restore, and unchanged existing photo rows; no video sidecar is created.

### Task 35.3: Video Import

- Goal: after Task 35.2, classify `.mov`/`.mp4` as candidates, authorize an AVFoundation probe, and store supported SDR metadata or a typed codec/color blocked reason. Other video extensions remain unsupported; HDR/log candidates remain catalog-visible but blocked.
- Acceptance: macOS tests cover supported Rec.709 full/video-range files and blocked codec/HDR/log results; Linux tests cover typed unavailable states; import is by reference and original hashes remain unchanged.

### Task 35.4: Poster Frame Thumbnails

- Goal: extract a poster frame (t≈0 or first non-black) via `AVAssetImageGenerator`, run the explicit Rec.709 range/transfer → linear Display P3 → thumbnail output transform, and write only to the disposable thumbnail cache.
- Acceptance: grid shows color-contract-valid video thumbnails; full/video-range fixtures are covered; cache clear/invalidation and blocked HDR/log behavior are tested; macOS evidence is recorded.

### Task 35.5: Grid Video Presentation

- Goal: library grid badge (duration, video glyph) and asset-aware photo/video filtering; existing photo-only screens and commands continue receiving photos only; design tokens only.
- Acceptance: UI QA note; query tests prove the Task 35.2 domain boundary.

### Task 35.6: Core-Authorized Opaque Media Protocol

- Goal: add a session-scoped opaque media handle issued only by `silica-core` for a video asset in the currently opened library. The protocol resolves the handle to the exact canonical path stored for that catalog/library identity, rechecks kind/support/missing state, and serves bounded byte-range reads. IPC and WebView receive no filesystem path or `file://` URL; arbitrary paths, stale handles, cross-library handles, directories, symlinks to another target, and non-video rows fail closed.
- Acceptance: authorization and range-read tests cover allowed catalog identity plus every rejection above; capability diff is minimal and documented; no frontend command can mint a handle from a path.

### Task 35.7: Playback Preview

- Goal: loupe playback for supported SDR videos through the Task 35.6 opaque protocol; expose play/pause/scrub without arbitrary media-path access.
- Acceptance: play/pause/range-seek works in manual QA; stale/cross-library handles and blocked HDR/log assets render typed states; capability and original-safety evidence are recorded.

### Task 35.8: LUT Look Preview on Video

- Goal: transform a poster frame through explicit Rec.709 range/transfer → linear Display P3, apply the selected managed LUT + intensity, then apply the display output transform; label this still result as a preview approximation and persist one `silica.video_look_state` v1 mutation through Core and typed video history.
- Acceptance: CPU reference parity covers full/video range, output transform, and LUT intensity; commit/reopen/undo/redo/backup tests preserve the exact versioned state; missing managed LUT and blocked HDR/log states remain typed.

### Task 35.9: Video Export with LUT, Progress, and Cancellation

- Goal: `export_video_with_lut` in `silica-core` + `silica-video` first rejects direct, canonical, symlink, and hard-link identity matches between source and destination before creating any output. Then read supported frames, normalize Rec.709 range/transfer, transform to linear Display P3, apply the managed LUT, transform/tag Rec.709 output, and write video plus audio passthrough to a sibling temporary output before final rename. Surface bounded progress and cancellation. Cancellation or failure closes the writer, removes partial/temp output, records no completed export row, and may record only a canceled/failed action attempt.
- Acceptance: alias-preflight regressions run before output creation; macOS evidence covers playable output, duration within ±1 frame, A/V sync, audio, range/transfer tags, progress monotonicity, cancellation at multiple stages, zero partial output after cancel, no completed export record on cancel, and unchanged original SHA-256; unsupported codec/HDR/log inputs fail before output creation.

### Task 35.10: Video QA Checklist

- Goal: `checklists/VIDEO_LUT_MANUAL_QA.md` — A/V sync, Rec.709 full/video-range input and output tags, rotation metadata, opaque-media authorization, blocked HDR/PQ/HLG/log behavior, long-clip progress, cancellation, and partial-output cleanup; harness check for checklist structure.
- Acceptance: one recorded evidence run proves blocked HDR/log states, successful cancel with no partial output, and unchanged originals; harness wired.

Phase exit gate: asset-aware import → transformed thumbnail → Core-authorized playback → LUT export/cancel is demonstrated on macOS; photo-only queries remain isolated, HDR/log stays blocked, output color evidence is explicit, and originals remain unchanged.

---

## Phase 36: Service Release Gate

Goal: decide whether the completed local desktop track can produce a releasable artifact without weakening any inherited trust or distribution gate.

### Task 36.0: Claims and Docs Truth Pass

- Goal: update `README.md`, wiki overview/topics, and Known Limitations to match actual behavior after Phases 29-35 (RAW status, LUT features, video scope, remaining gaps like detail ops).
- Acceptance: every claim maps to recorded evidence; no capability named that returns `Unavailable` in default builds.

### Task 36.1: Security and Capability Final Audit

- Goal: re-audit CSP, Tauri capabilities (including the Phase 35 media scope), action-log coverage for new sensitive actions (LUT import, video export), and `SECURITY.md` accuracy.
- Acceptance: audit recorded; any gap becomes a blocking task before release.

### Task 36.2: Current Identity Review Gate

- Goal: review current product/bundle/repository identity for release consistency. The default and expected result remains product name `SilicaRAW` and bundle identifier `dev.silicaraw.desktop`; this track assumes neither a fork nor a rebrand. Any rename or identity change requires a separate explicit maintainer ADR and later mechanical task.
- Acceptance: current identity, icon, license-header retention, repository metadata, and bundle evidence agree; no rename/rebrand work occurs without the separate ADR.

### Task 36.3: Signed Release Dependency Check

- Goal: reconcile with Q6.3/Q6.4, Task 27.2, and Phase 28 — offline installed-app evidence, release notes, Developer ID prerequisites, signing, notarization, checksums, Gatekeeper, and clean-Mac downloaded-artifact QA. An unsigned artifact may remain an explicitly labeled developer preview only; it never satisfies this release gate, public beta, or v1.0 requirements and leaves release blocked.
- Acceptance: every inherited gate has current evidence; the signed path is releasable only when all pass; otherwise the evidence index records a blocking state and any unsigned artifact remains developer-preview-only.

### Task 36.4: Track Evidence Index

- Goal: evidence index linking every required-baseline Phase 29-32 exit record plus Phase 33 and/or Phase 35 when RAW/video is selected or claimed. Record optional Phase 34 and every unselected capability track as explicit `N/A` with rationale rather than a missing blocker.
- Acceptance: index merged; missing required or claimed evidence is a blocker; optional/unselected tracks are visibly `N/A`, never silently omitted.

Phase exit gate: releasable only when all inherited Q6.3/Q6.4, Task 27.2, and applicable Phase 28 gates pass. Otherwise Phase 36 exits only to an explicit blocked state; an unsigned developer preview is not a release outcome.

---

## Cross-Phase Stop Gates

Stop and report instead of proceeding when:

- Any task would modify an original photo or video file.
- Any task needs network access, telemetry, or cloud inference (Phase 34 especially).
- A new dependency is needed that is not already gated in a design-gate ADR (record license + reason in `docs/DEPENDENCIES.md` first).
- Edit graph structure needs a change not covered by the Task 32.0 ADR.
- The Phase 30 linear Display P3 CPU reference, Metal preview, export/histogram path, or explicit input/output transforms cannot meet the accepted parity/evidence policy. Legacy RGB8 behavior cannot replace the CPU/Metal contract.
- Task 31.5 LUT round-trip tolerance fails, or `.cube` parsing/writing cannot preserve finite bounded 17/33/65 lattice count and serialization order. Do not loosen limits or tolerances without a recorded decision.
- A managed LUT can be deleted by cache clear, omitted from backup/restore, resolved outside its managed root, or silently ignored when missing or hash-mismatched.
- RAW support is inferred from extension alone, or the working artifact is lossy, low-depth, not explicit linear Display P3, or inconsistent with the `Rgba16Float` handoff without accepted equivalent evidence.
- A video can enter a photo-only query, receive an arbitrary file URL/path, bypass Core catalog/library authorization, skip explicit Rec.709 range/transfer transforms, treat HDR/log as supported, leave partial output after cancel/failure, or mutate its original.
- An unsigned artifact is described as satisfying a release, public beta, or v1.0 gate.

## Known Deferred Items (recorded, not scheduled)

- Sharpening/noise reduction pixel implementation (unblocked after Phase 30; needs a spatial-op design gate).
- Broader camera-profile refinement beyond the Phase 30/33 explicit input-profile contract.
- Advanced real-time GPU video grading beyond the v1 one-LUT preview/export scope; the Phase 30 Metal still-image product preview is not deferred.
- Log footage color management and 1D shaper LUTs.
- Timeline editing, cuts, transitions, and keyframes.
- Windows/Linux product builds (the stack is deliberately Apple-native).
- Learned-model auto grading (Task 34.8 gate).
