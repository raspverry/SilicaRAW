---
title: LUT and Video Service Master Plan
status: active
audience: agents
updated: 2026-07-10
source_of_truth: docs/wiki/roadmaps/lut-video-service-master-plan.md
---

# LUT and Video Service Master Plan

## Summary

This page is the execution router for the service-direction track: Phase 29 through Phase 36.

The track goal is to grow the local alpha into a shippable service product with three pillars, in this order:

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

This plan continues the numbering of the [Post-Alpha Master Execution Plan](post-alpha-master-execution-plan.md), which ends at Phase 28. Phases 27 and 28 (signing/notarization and v1.0 gates) remain externally blocked on Developer ID funding and are not duplicated here; Phase 36 depends on them.

## Operating Rules

- Use this page before choosing any Phase 29 or later task.
- One task is one atomic, committable unit: one PR, smallest reviewable scope.
- When a phase starts, create its task cards under `docs/wiki/tasks/` from this page in a docs-only task first (`29.0`, `29.1`, ... follow the existing card template: Goal, Read Before Work, Files, Scope, Acceptance Criteria, Validation, Stop Gates, Completion State).
- Every `X.0` task is a design gate. Do not start `X.1+` before the `X.0` decision is recorded (ADR under `docs/wiki/decisions/` when it changes architecture, schema, or dependencies).
- All existing hard rules stay in force: never modify original files, no network/telemetry/cloud by default, document every new dependency in `docs/DEPENDENCIES.md`, do not invent edit graph structure outside `schemas/edit_graph.schema.json`, use design tokens for UI.
- Do not claim visual color correctness beyond recorded evidence. LUT claims follow the same evidence discipline as export color claims.

## Current Position (facts this plan builds on)

As of 2026-07-10:

- The working product is a raster (JPEG/PNG/TIFF) develop-and-export alpha. RAW decode is not in the default build; supported import extensions are raster-only (`crates/silica-catalog/src/lib.rs:97`).
- The only real pixel math is the CPU chain in `crates/silica-export/src/lib.rs:1337-1596`. Every stage re-quantizes to 8-bit sRGB. This chain is the seed of the float color core.
- Detail sharpening and noise reduction are persisted as edit state but never applied to pixels (no `apply_detail`/`apply_noise` exists in `silica-export`).
- Develop preview does a full open-decode-adjust-encode-write-read disk round-trip per slider event with no debounce.
- `silica-export` tests hard-depend on macOS ColorSync profile paths (`/System/Library/ColorSync/Profiles/...`) and fail off-macOS.
- `silica-render` is a routing contract (no rendering); `native_metal_viewer` is a shell (no Metal calls); `silica-mlx` has no ML runtime.
- God files: `crates/silica-core/src/lib.rs` (~13.3k lines), `crates/silica-storage/src/lib.rs` (~10.6k), `apps/desktop/src-tauri/src/main.rs` (~9.3k), `apps/desktop/static/index.html` (~8k with one inline script).
- There is no LUT, `.cube`, or video code anywhere in the repository.

## Track Map

| Wave | Scope | Entry Gate | Exit Gate |
| --- | --- | --- | --- |
| A | Phase 29: Service Foundation Hardening | none | Honest UI claims, portable tests, modular crates/frontend, in-memory preview |
| B | Phase 30: Float Color Chain | Phase 29 complete through 29.5 | f32 chain is the single color path for preview and export, parity evidence recorded |
| C | Phase 31: Manual LUT Export | Phase 30 complete | User exports a `.cube` from any edit state; round-trip tolerance evidence recorded |
| D | Phase 32: LUT Import and Apply | Phase 31 complete | Imported LUTs preview/commit/undo like any edit; schema extension recorded |
| E | Phase 33: RAW Product Enablement | Phase 29 complete (parallel to C/D) | RAW files import, develop, and export on macOS product builds |
| F | Phase 34: AI-Assisted LUT | Phase 31 complete; Phase 32 recommended | Local-only reference match produces approvable suggestions and one-click LUT bake |
| G | Phase 35: Video Foundation | Phases 31 and 32 complete | Video imports, previews, and exports with a LUT applied; originals untouched |
| H | Phase 36: Service Release Gate | Phases 29-32 complete; 33-35 as scoped | Claims audit, capability audit, identity decision, evidence index |

Dependency notes:

- Phase 33 (RAW) only needs Phase 29 and can run in parallel with Phases 31-32 by a second agent, but merges must keep the f32 chain (Phase 30) as the only color path.
- Phase 35 (video) consumes the `.cube` parser/applier from Phase 31 and the LUT edit-state extension from Phase 32; do not start it earlier.
- Phase 34 must not add network access. Cloud inference is out of scope unless a maintainer changes the no-network rule explicitly.

---

## Phase 29: Service Foundation Hardening

Goal: remove debt that blocks every later phase — dishonest UI claims, macOS-hardcoded tests, god files, and the disk-round-trip preview.

### Task 29.0: Service Direction Design Gate

- Goal: record the track charter — macOS-only v1, LUT-first video scope, fork/branding deferred to Phase 36, CSP hardening committed, phases 27/28 dependency acknowledged.
- Files: `docs/wiki/decisions/`, this plan.
- Acceptance: ADR merged; no code change.

### Task 29.1: Honest Detail Controls

- Goal: stop the UI from claiming sharpening/noise-reduction affect pixels while `silica-export` never applies them.
- Files: `apps/desktop/static/index.html`, `apps/desktop/src-tauri/src/main.rs`, `docs/wiki/topics/`.
- Scope: disable the detail sliders with a "deferred" tooltip (matching existing deferred-control convention) OR remove their preview/commit commands from the UI path; keep stored edit state readable; document the gap.
- Acceptance: no enabled control changes edit state that the pixel pipeline ignores; existing detail edit-state tests still pass.
- Stop gate: do not implement sharpening/NR here; that is future work after Phase 30.

### Task 29.2: ICC Profile Portability

- Goal: remove the hard dependency on `/System/Library/ColorSync/Profiles/*.icc` so `silica-export` tests pass on Linux CI.
- Files: `crates/silica-export`, `docs/DEPENDENCIES.md` if a profile-generation dependency is added (prefer none).
- Scope: bundle minimal known-good sRGB and Display P3 ICC byte tables as repo assets (license-checked) or generate them; keep macOS system profiles as an optional override; record profile SHA-256 in export results unchanged.
- Acceptance: `cargo test -p silica-export` passes on Linux; macOS behavior and embedded-ICC evidence unchanged.

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

- Goal: debounce/throttle develop slider IPC (~100 ms trailing) in the frontend while keeping the existing monotonic sequence guard for stale-response drops.
- Files: `apps/desktop/static/js/`.
- Acceptance: rapid drag produces bounded IPC calls; final value always rendered; manual QA note recorded.

Phase exit gate: all tasks complete; full workspace tests green on Linux and macOS evidence recorded.

---

## Phase 30: Float Color Chain

Goal: one pure `f32` color chain, fed by the edit graph, used by both preview and export. This is the foundation for LUT baking (Phase 31), LUT application (Phase 32), AI matching (Phase 34), and video (Phase 35).

### Task 30.0: Color Chain Design Gate

- Goal: ADR for a new dependency-free crate `silica-color`: `f32` RGB in `[0,1]`, sRGB-encoded domain for v1 (linear working space is a recorded future migration, not v1), fixed operation order matching `write_jpeg_develop_preview` (exposure/contrast → white balance → tone recovery → tone curve → color presence → HSL mixer), single quantization at encode time only.
- Acceptance: ADR merged; crate boundaries recorded (silica-color depends on nothing; silica-export and silica-core depend on it; silica-edit does not).

### Task 30.1: Create silica-color Crate

- Goal: workspace member `crates/silica-color` with `Rgb32` type, `ColorChainParams` struct (all op parameter blocks, serde optional), and identity-default constructors. No ops yet.
- Acceptance: builds in workspace; README states crate responsibility; `docs/DEPENDENCIES.md` untouched (no new external deps).

### Tasks 30.2-30.7: Port Ops to f32 (one task per op)

- 30.2 exposure/contrast, 30.3 white balance, 30.4 tone recovery, 30.5 tone curve (+ curve evaluation), 30.6 color presence (vibrance/saturation), 30.7 HSL mixer (+ f32 rgb↔hsl).
- Goal (each): pure `fn(Rgb32, &Params) -> Rgb32` ported from the corresponding `silica-export` function (`lib.rs:1337-1594` pre-split locations), without the per-stage 8-bit quantization.
- Acceptance (each): unit tests for identity params (bit-exact passthrough), known input/output vectors, and clamping; no NaN/inf outputs for valid params (property test).
- Stop gate: do not change the legacy 8-bit functions in the same PR.

### Task 30.8: Compose Full Chain

- Goal: `apply_color_chain(rgb, &ColorChainParams) -> Rgb32` applying ops in the gated order; identity-params chain is exact passthrough.
- Acceptance: property tests (identity, clamp bounds, determinism); order matches the ADR.

### Task 30.9: Edit Graph Mapping

- Goal: `edit_graph_to_color_chain_params(&EditGraph) -> ColorChainParams` in `silica-core`, covering every color op the chain supports and explicitly listing unmapped (spatial) families.
- Acceptance: round-trip tests from fixture edit graphs; unmapped families enumerated in a typed report, not silently dropped.

### Task 30.10: Golden Parity Evidence

- Goal: prove the f32 chain matches the legacy 8-bit chain within tolerance on fixture rasters (per-pixel max diff ≤ 1/255 per stage, ≤ 2/255 full chain, or record justified deviations).
- Files: `crates/silica-export` tests, `checklists/GOLDEN_IMAGE_TOLERANCE_POLICY.md`.
- Acceptance: tolerance evidence recorded in the task card; deviations explained (expected: fewer quantization steps means small, directionally-better diffs).

### Task 30.11: Switch Preview and Export to f32 Chain

- Goal: `write_jpeg_develop_preview`, histogram source, and all export entry points consume `apply_color_chain` with one final quantization; legacy per-stage 8-bit functions become test-only references or are deleted.
- Acceptance: all export/preview tests pass with updated goldens per the 30.10 tolerance policy; original-safety tests pass; manual visual QA note recorded.

Phase exit gate: one color code path; parity evidence merged.

---

## Phase 31: Manual LUT Export (.cube)

Goal: a user can export their current edit's color transform as an industry-standard 3D `.cube` LUT.

### Task 31.0: LUT Format Design Gate

- Goal: ADR for `.cube` specifics — default `LUT_3D_SIZE 33` (17/65 selectable), `DOMAIN_MIN 0 0 0` / `DOMAIN_MAX 1 1 1`, sRGB-in/sRGB-out v1, `TITLE` from photo/preset name, comment lines recording app version and edit-state hash, 6-decimal float formatting, LF line endings; exclusion policy: spatial ops (masks, geometry, detail) are never baked and must be reported.
- Acceptance: ADR merged with a sample golden file.

### Task 31.1: Cube Writer

- Goal: `write_cube(&Lut3d) -> String` in `silica-color` (pure formatting, no I/O, no new deps).
- Acceptance: golden-file tests for 17/33 sizes; float formatting and header order locked by tests.

### Task 31.2: LUT Bake

- Goal: `bake_lut(&ColorChainParams, size) -> (Lut3d, BakeExclusionReport)` iterating the lattice through `apply_color_chain`; the report lists every edit-graph family excluded (masks, geometry, detail) for UI display.
- Acceptance: identity params bake to an identity LUT (exact); non-identity spot nodes match direct chain evaluation; exclusion report tested.

### Task 31.3: Cube Parser and Validator

- Goal: `parse_cube(&str) -> Result<Lut3d, CubeError>` accepting the writer's output plus common third-party variants (comments, CRLF, optional DOMAIN lines); typed errors for malformed input.
- Acceptance: parses own golden files bit-identically; rejects malformed fixtures with specific errors; fuzz-ish edge tests (empty, truncated, wrong count).

### Task 31.4: Trilinear LUT Applier

- Goal: `apply_lut(rgb, &Lut3d) -> Rgb32` with trilinear interpolation and input clamping, in `silica-color`.
- Acceptance: identity LUT is passthrough within 1e-6; known-node exactness; interpolation midpoint tests.

### Task 31.5: Round-Trip Tolerance Proof

- Goal: bake → write → parse → apply on fixture images versus direct chain evaluation; record max/mean error per LUT size.
- Acceptance: 33³ max error within the tolerance recorded in the 31.0 ADR (propose ≤ 1.5/255 for typical edits); evidence in the task card; failures block, not warn.

### Task 31.6: Core LUT Export API and Record

- Goal: `export_photo_lut_cube` in `silica-core`: load active edit graph → map params → bake → write file via dialog-provided path; write an `exports` record (new format value `cube`, migration if the schema constrains formats) and an action-log entry.
- Acceptance: storage migration tested; export record and action log rows asserted; originals untouched; file content matches direct bake.

### Task 31.7: Desktop Command Wiring

- Goal: `export_photo_lut_cube` Tauri command + DTO, registered alongside existing export commands; save-dialog flow only (no new capabilities).
- Acceptance: command test coverage matching existing export command tests.

### Task 31.8: Export LUT UI

- Goal: Export panel section "Export LUT (.cube)" with size selector (17/33/65) and a non-dismissable notice listing excluded spatial edits from the bake report; design tokens only.
- Acceptance: manual QA per checklist; disabled state when photo has no committed edit state is defined and tested.

### Task 31.9: External Tool QA Checklist

- Goal: `checklists/LUT_EXPORT_MANUAL_QA.md` — load an exported LUT in at least two external tools (e.g. DaVinci Resolve, Final Cut/Photos, or an open-source LUT previewer), compare against in-app render, record screenshots/hashes; add a harness check script for the checklist format.
- Acceptance: checklist merged with one recorded evidence run; harness check wired into `scripts/harness/check.sh`.

Phase exit gate: a real `.cube` exported from a real edit, verified in an external tool, evidence recorded.

---

## Phase 32: LUT Import and Apply

Goal: imported `.cube` LUTs become first-class, undoable edit state ("looks"), previewable in Develop.

### Task 32.0: Edit Graph LUT Extension Design Gate

- Goal: ADR for storing a LUT reference in the edit graph — use the existing `extensions` object (`extensions.lut = { library_lut_id, sha256, intensity }`) versus a schema version bump; decide LUT asset storage location inside the library (managed copies, content-addressed by SHA-256); decide chain position (after HSL mixer, final color op) and intensity blend semantics (linear mix of input/output).
- Acceptance: ADR merged; `schemas/edit_graph.schema.json` change reviewed against the "do not invent edit graph structure" rule.

### Task 32.1: Schema and Edit Model Extension

- Goal: implement the gated schema change in `schemas/edit_graph.schema.json` + `silica-edit` (typed accessors, validation: intensity 0..1, sha256 format), plus `apply_lut_reference`/`clear_lut_reference` mutators.
- Acceptance: schema example updated; serde round-trip and validator tests; unknown-field rejection preserved.

### Task 32.2: Library LUT Import and Catalog

- Goal: import a `.cube` file into the library (managed copy, SHA-256 verified via 31.3 parser) with a new `luts` catalog table (migration): id, title, sha256, size, source filename, created_at.
- Files: `crates/silica-storage`, `crates/silica-core`.
- Acceptance: migration tested (fresh + upgrade); duplicate import by hash is idempotent; malformed files rejected with typed errors; action log records the import.

### Task 32.3: Chain Integration

- Goal: extend `ColorChainParams` with an optional resolved `Lut3d` + intensity; `apply_color_chain` applies it as the final color op via the 31.4 applier.
- Acceptance: identity LUT and intensity 0 are exact passthrough; chain property tests extended.

### Task 32.4: Preview, Commit, and History Wiring

- Goal: `preview_lut_edit` / `commit_lut_edit` / clear paths in `silica-core` + desktop commands, resolving `library_lut_id` → parsed LUT with cache; history/undo/redo entries like any other edit family.
- Acceptance: undo/redo tests; missing/deleted LUT asset yields a typed blocked state (matching missing-file conventions), never a crash.

### Task 32.5: Develop LUT Panel

- Goal: Develop panel "Look / LUT" section — pick from imported LUTs, intensity slider, clear button; import entry point via file dialog; design tokens only.
- Acceptance: manual QA; blocked state for missing assets rendered per existing blocked-state patterns.

### Task 32.6: LUT Apply Evidence

- Goal: record parity evidence — exporting a photo with LUT applied equals chain+LUT reference within tolerance; extend `checklists/LUT_EXPORT_MANUAL_QA.md` with an import/apply section.
- Acceptance: evidence recorded; harness check updated.

Phase exit gate: import → preview → commit → undo → export all work with LUTs; evidence recorded.

---

## Phase 33: RAW Product Enablement (parallel-capable)

Goal: the product finally decodes RAW on macOS builds using the already-selected Core Image path (Spike 002 decision stands; LibRaw remains deferred).

### Task 33.0: RAW Enablement Design Gate

- Goal: ADR to promote `core-image-raw-probe` from a non-default feature to the default macOS product build; decide the RAW-derived working artifact format (v1: full-resolution JPEG quality 95 as today's probe writes, with 16-bit TIFF recorded as a future upgrade), artifact cache location under existing cache-records, and the supported-extension list sourced from the Core Image support matrix (Task 12.3 evidence).
- Acceptance: ADR merged; fixture policy reaffirmed (no committed RAW files; local manifest per Task 12.5).

### Task 33.1: Default-Enable Core Image Decode on macOS

- Goal: make the Core Image probe compile into default macOS builds (feature default per-target), keep non-macOS builds returning the existing typed `Unavailable` states.
- Acceptance: macOS CI/dev-preview workflow builds it; Linux workspace tests unaffected.

### Task 33.2: RAW Import Support Mapping

- Goal: extend `ALPHA_SUPPORTED_PHOTO_EXTENSIONS` handling so matrix-supported RAW extensions import as supported-with-decode-pending instead of `unsupported`; unsupported RAW stays blocked with the existing state.
- Files: `crates/silica-catalog`, `crates/silica-storage`, `crates/silica-core`.
- Acceptance: import tests for supported/unsupported RAW paths; grid blocked-state behavior for pending decode defined.

### Task 33.3: RAW Working Artifact Cache

- Goal: on first develop/preview of a RAW photo, produce and cache the working raster artifact via the Core Image path (`write_core_image_jpeg` full-res mode), tracked in `cache_records`, invalidated by source fingerprint.
- Acceptance: artifact reused across sessions (test seam); cache-clear removes it; original untouched (SHA-256 evidence).

### Task 33.4: Develop on RAW Artifacts

- Goal: route Develop preview/commit for RAW photos through the cached artifact into the existing f32 chain; histogram included.
- Acceptance: end-to-end develop on a manifest RAW fixture recorded as macOS evidence; non-RAW paths unchanged.

### Task 33.5: RAW Export Wiring

- Goal: wire the existing `RawFullResolutionArtifact` export contract (`silica-render` plan + `silica-core` request path) so JPEG/PNG/TIFF export of RAW photos renders from the full-res artifact, never the viewer preview.
- Acceptance: export tests using the artifact path; export records show the RAW source; original-safety evidence.

### Task 33.6: RAW Manual Color QA

- Goal: manual comparison of in-app RAW rendering versus Preview.app/Photos on the fixture manifest set; record deviations honestly (no correctness claims beyond evidence).
- Acceptance: checklist run recorded; README "Known Limitations" updated to reflect actual RAW status.

Phase exit gate: RAW import → develop → export works on macOS with evidence; claims updated.

---

## Phase 34: AI-Assisted LUT Creation (local-only)

Goal: "auto" LUT creation from a reference image, with the existing suggestion-approval discipline. v1 is deterministic (no ML runtime, no network); a learned model is a stop-gated follow-up.

### Task 34.0: AI Assist Design Gate

- Goal: ADR reaffirming local-only (no network inference — cloud APIs are prohibited by standing rules unless a maintainer explicitly changes them); v1 algorithm = deterministic reference matching (channel statistics + histogram matching → white balance + tone curve + saturation estimate); output is an edit-graph suggestion routed through the Phase 24 approval flow, never a silent edit; MLX learned model deferred behind its own gate (34.6).
- Acceptance: ADR merged.

### Task 34.1: Image Statistics Module

- Goal: `silica-color` functions for channel histograms, percentiles, mean/std in f32, over a downsampled working raster.
- Acceptance: unit tests on synthetic images (uniform, gradient, two-tone); deterministic outputs.

### Task 34.2: Reference Match Estimator

- Goal: pure function `(source_stats, reference_stats) -> ColorMatchSuggestion` producing white balance gains, a monotonic tone curve (via percentile/histogram matching, clamped control points), and a saturation delta; bounded parameter ranges matching edit-graph validation.
- Acceptance: property tests (identical stats → identity suggestion; monotonic curve; all outputs within edit-graph valid ranges); golden tests on fixture pairs.

### Task 34.3: Suggestion Routing

- Goal: `suggest_reference_match(photo_id, reference_path)` in `silica-core`: compute stats for both images, run the estimator, store the result in `ai_results` as a non-mutating suggestion consumable by the existing AI Review approve/reject flow (Phase 24 checkpoint + action-log provenance).
- Acceptance: approval applies an undoable edit checkpoint; rejection leaves state untouched; tests for both.

### Task 34.4: Reference Match UI

- Goal: Develop panel action "Match reference..." (file dialog for the reference image) surfacing the suggestion in the existing AI Review panel with the standard approve/reject controls.
- Acceptance: manual QA; blocked states for unreadable reference files.

### Task 34.5: One-Click LUT from Suggestion

- Goal: after approval, offer "Export this look as LUT" reusing the Phase 31 bake path on the approved edit state.
- Acceptance: end-to-end test: reference match → approve → bake → parse → apply parity within Phase 31 tolerances.

### Task 34.6: Learned Model Gate (stop-gated, optional)

- Goal: only if 34.2 quality is recorded as insufficient — ADR for an MLX-based color-suggestion model behind a non-default feature gate, using the existing model-manifest validation (license, version, SHA-256) from `silica-mlx`; no model weights committed to the repo.
- Stop gate: do not start without a maintainer decision and a quality-gap record from 34.5 evidence.

Phase exit gate: reference-match → approval → LUT export demonstrated with recorded evidence; no network access added.

---

## Phase 35: Video Foundation (LUT-first scope)

Goal: videos import into the library, show thumbnails, play back, and export with a LUT applied. This is explicitly NOT a timeline editor: no cuts, no keyframes, no per-clip grading beyond one LUT + intensity in v1.

### Task 35.0: Video Scope Design Gate

- Goal: ADR — AVFoundation/Core Image/VideoToolbox on macOS only (matching the Core Image RAW precedent; ffmpeg rejected for v1 to avoid LGPL/size), supported containers `.mov`/`.mp4` (H.264/HEVC), catalog approach (asset `kind` column + `video_metadata` table), color handling v1 (Rec.709/sRGB tagged sources only; log footage imports but LUT preview is marked unmanaged), export = same container, video track re-encoded with LUT via `CIColorCube`, audio passed through untouched.
- Acceptance: ADR merged; `docs/DEPENDENCIES.md` plan for `objc2-av-foundation` (and related) recorded.

### Task 35.1: silica-video Crate Boundary

- Goal: new workspace crate `crates/silica-video` with typed request/result contracts and a macOS feature-gated AVFoundation probe: duration, fps, dimensions, codec, audio presence. Non-macOS returns typed `Unavailable`.
- Acceptance: probe evidence on a small self-generated fixture clip (generated in-test via AVFoundation or checked-in tiny clip with license note); Linux builds pass with stubs; DEPENDENCIES.md updated.

### Task 35.2: Catalog Asset Kind Migration

- Goal: migration adding `kind` (`photo` default, `video`) to `photos` (or the gated alternative) plus `video_metadata` table (duration_ms, fps, width, height, codec, has_audio).
- Acceptance: fresh + upgrade migration tests; all existing photo queries unaffected (kind defaults verified).

### Task 35.3: Video Import

- Goal: import scanner recognizes `.mov`/`.mp4`, stores metadata via the probe, marks other video extensions unsupported with the existing blocked-state convention.
- Acceptance: import tests (macOS evidence for probe values; Linux tests for state handling); originals untouched.

### Task 35.4: Poster Frame Thumbnails

- Goal: extract a poster frame (t≈0 or first non-black) via `AVAssetImageGenerator` into the existing JPEG thumbnail cache path.
- Acceptance: grid shows video thumbnails; cache-clear/invalildation covered; macOS evidence recorded.

### Task 35.5: Grid Video Presentation

- Goal: library grid badge (duration, video glyph) for `kind=video`; filtering by kind in query commands; design tokens only.
- Acceptance: UI QA note; query tests.

### Task 35.6: Playback Preview

- Goal: loupe playback for videos via webview `<video>` served through a scoped Tauri asset/stream protocol; capability diff explicitly reviewed (this is the first filesystem-ish capability expansion — scope it to library video paths only).
- Acceptance: play/pause/scrub works in manual QA; capability change documented in the task card and SECURITY-relevant docs; no arbitrary-path access.

### Task 35.7: LUT Look Preview on Video

- Goal: apply the selected library LUT (Phase 32) to the poster frame for a cheap look preview (still image), clearly labeled as a preview approximation; store the chosen LUT + intensity per video in edit state (color-only subset).
- Acceptance: preview parity with `apply_lut` on the poster raster; edit-state persistence tests.

### Task 35.8: Video Export with LUT

- Goal: `export_video_with_lut` in `silica-core` + `silica-video`: `AVAssetReader` → per-frame `CIColorCube` (filter built from the baked/imported `Lut3d`) → `AVAssetWriter`; audio passthrough; progress callback surfaced to the UI like batch export; output to a new file only.
- Acceptance: macOS evidence — output plays, duration matches ±1 frame, audio intact, original SHA-256 unchanged; identity-LUT export visually matches source (recorded check); typed errors for unsupported codecs.

### Task 35.9: Video QA Checklist

- Goal: `checklists/VIDEO_LUT_MANUAL_QA.md` — A/V sync, color tags (Rec.709 in/out), rotation metadata, HDR/log footage behavior (expected: blocked or labeled unmanaged), long-clip cancel path; harness check for the checklist.
- Acceptance: one recorded evidence run; harness wired.

Phase exit gate: import → thumbnail → playback → LUT export demonstrated on macOS with evidence.

---

## Phase 36: Service Release Gate

Goal: turn the track output into a releasable service artifact honestly.

### Task 36.0: Claims and Docs Truth Pass

- Goal: update `README.md`, wiki overview/topics, and Known Limitations to match actual behavior after Phases 29-35 (RAW status, LUT features, video scope, remaining gaps like detail ops).
- Acceptance: every claim maps to recorded evidence; no capability named that returns `Unavailable` in default builds.

### Task 36.1: Security and Capability Final Audit

- Goal: re-audit CSP, Tauri capabilities (including the Phase 35 media scope), action-log coverage for new sensitive actions (LUT import, video export), and `SECURITY.md` accuracy.
- Acceptance: audit recorded; any gap becomes a blocking task before release.

### Task 36.2: Service Identity Decision Gate

- Goal: maintainer ADR on fork identity — product name, bundle identifier, icon, license header retention (MIT obligations: keep upstream copyright notice), repository visibility.
- Acceptance: ADR merged; rename executed only after the decision (separate mechanical task if renamed).

### Task 36.3: Signed Release Dependency Check

- Goal: reconcile with blocked Phases 27/28 — Developer ID funding, signing, notarization; if still blocked, ship the unsigned developer-preview path with the existing Gatekeeper documentation and record the block.
- Acceptance: release path decision recorded; release runbook updated for the new artifact contents.

### Task 36.4: Track Evidence Index

- Goal: evidence index page linking every phase-exit evidence record for Phases 29-35 (mirroring `public-beta-evidence-index.md`).
- Acceptance: index merged; missing evidence enumerated as blockers, not hidden.

Phase exit gate: releasable artifact with honest claims, or an explicit recorded block.

---

## Cross-Phase Stop Gates

Stop and report instead of proceeding when:

- Any task would modify an original photo or video file.
- Any task needs network access, telemetry, or cloud inference (Phase 34 especially).
- A new dependency is needed that is not already gated in a design-gate ADR (record license + reason in `docs/DEPENDENCIES.md` first).
- Edit graph structure needs a change not covered by the Task 32.0 ADR.
- f32-vs-legacy parity (30.10) or LUT round-trip tolerance (31.5) cannot be met — tolerance failures block; do not loosen tolerances without a recorded decision.
- The Phase 35 capability expansion cannot be scoped to library media paths.

## Known Deferred Items (recorded, not scheduled)

- Sharpening/noise reduction pixel implementation (unblocked after Phase 30; needs a spatial-op design gate).
- Linear working color space migration and camera profiles (post-Phase 33 quality track).
- GPU/Metal render pipeline (required before real-time video grading beyond poster-frame preview).
- Log footage color management and 1D shaper LUTs.
- Windows/Linux product builds (the stack is deliberately Apple-native).
- Learned-model auto grading (Task 34.6 gate).
