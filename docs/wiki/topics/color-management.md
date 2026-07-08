---
title: Color Management
status: active
audience: all
updated: 2026-06-13
source_of_truth: docs/09_Color_Management_Specification.md
---

# Color Management

## Summary

Color management is a release-trust issue. Spike 003 selected the first implementation recommendation, but color correctness remains unproven until tagged fixtures exist.

## Current Stance

- Spike 003 selected Core Image/ColorSync-compatible color management first.
- The first working-space recommendation is linear Display P3-compatible RGB.
- Preview should be display-profile aware.
- Export should default to sRGB with ICC embedding and support Display P3 when explicitly selected.
- Decoder-specific color assumptions must be documented.
- Phase 5.1 records display-profile-aware preview readiness but does not prove color correctness.
- Committed tagged color fixtures are still missing.
- Local ignored Class F fixtures now have profile-probe evidence for sRGB, Display P3, and untagged JPEG handling.
- Phase 13 now has an execution plan, brief, and task cards for fixture-backed color proof.
- The local export UI/API now keeps sRGB as the default and exposes Display P3 only as an explicit ICC-backed choice.
- Task 17.2.1 adds deterministic local white-balance adjustment for supported JPEG/JPG Develop preview and JPEG export parity. This is product edit behavior, not a fixture-backed color-correctness claim.
- Task 18.2.2 adds deterministic local HSL color mixer adjustment for supported JPEG/JPG Develop preview and JPEG export parity. This is product edit behavior, not a fixture-backed color-correctness claim.

## Blocked Work

- Fixture-backed color correctness claims.
- Manual visual review records.
- Camera profile behavior.
- Fixture-backed golden image baseline.
- Broad user-facing color claims.

## Fixture Status

Class F tagged raster fixture source review is complete for local-only synthetic fixtures:

```txt
srgb_jpeg -> synthetic pixels + local macOS sRGB ICC, local-only
display_p3_jpeg -> synthetic pixels + local macOS Display P3 ICC, local-only
untagged_jpeg -> synthetic pixels + removed color-management properties, local-only
```

No committed color-management fixture corpus exists yet. Task 13.2 generated an ignored local corpus and manifest:

```txt
manifest: .tmp/legal-color-fixtures/color-fixtures.json
srgb_jpeg sha256: ba7fed85d6fdd5d2dbf8376fdb4030e2206a541e58efa2c69ec5ba494152b6fe
display_p3_jpeg sha256: 84855ac721fbc8062ef543f5fe95df843e6e4a211be2eac2869e3456c55e1ebb
untagged_jpeg sha256: aff808a1c3625a5de3e249c80c3cb9d7e9ae53d92f6bd95158aa4a0f384a23e9
```

This local corpus and the Task 13.4 harness now prove local profile-probe handling for the three Class F subclasses. They do not prove ColorSync transform output, export ICC embedding, or color correctness.

## Task 10.1 Color Class F Contract

Color Class F covers tagged sRGB, tagged Display P3, and untagged raster fixture expectations. Hashes, profile declarations, and manifest entries do not prove color correctness. Color correctness claims remain blocked until fixture-backed proof and tolerance policy exist.

Task 10.1 records the future fixture contract only:

```txt
tagged sRGB raster -> embedded ICC expected, sRGB input expectation
tagged Display P3 raster -> embedded ICC expected, Display P3 input expectation
untagged raster -> no embedded ICC expected, assume_srgb policy
```

These entries do not prove color correctness because no fixture-backed transform output, tolerance policy, or visual review evidence exists yet.

## Task 10.2 Golden Image and Tolerance Policy

Task 10.2 records the first [Golden Image and Tolerance Policy](../../../checklists/GOLDEN_IMAGE_TOLERANCE_POLICY.md).

The policy separates byte equality, file/profile inspection, pixel or perceptual tolerance, and manual visual review. It does not add golden images, ICC parsing, pixel comparison, or color correctness proof.

Color correctness claims remain blocked until fixture-backed proof, approved tolerance values, automated comparison results, and manual visual review records exist.

## Phase 13 Plan

[Phase 13 Color Pipeline Proof Plan](../roadmaps/phase-13-color-pipeline-proof-plan.md) is the active route for turning the Spike 003 recommendation into fixture-backed evidence.

Current planned order:

```txt
13.0 design gate
13.1 source review [complete]
13.2 ignored local fixtures and manifest [complete locally]
13.3 feature-gated color probe [complete]
13.4 probe harness [complete]
13.5 support matrix [complete]
13.6 ICC export proof [complete]
13.7 schema-safe color metadata [complete]
13.8 explicit export color options [complete]
```

Phase 13 now has profile-probe, ICC embedding, metadata, and explicit option wiring evidence. Color correctness claims remain blocked until approved tolerance results and executed manual visual review exist.

Task 13.3 added a non-default `color-probe` feature in `silica-render`. It records JPEG profile metadata and source hashes for fixture proof only. It does not apply a ColorSync transform, write ICC profiles, render pixels, or prove color correctness.

Task 13.4 added `scripts/harness/check-color-probe-fixtures.py` and a feature-gated `silica-render` `color_probe_report` example. With `SILICARAW_COLOR_FIXTURE_MANIFEST` pointed at the ignored local manifest, the harness records profile probe status, input profile, working space, output profile, transform path, source hash, file size, modified time, and original hash preservation.

The local Task 13.4 run passed for sRGB, Display P3, and untagged Class F fixtures. This is profile-probe evidence only; color correctness, export ICC embedding, and transform output remain blocked.

## Phase 13.5 Color Probe Support Matrix

The matrix records local ignored fixture evidence from Task 13.4. It is not a broad product color claim.

| Fixture subclass | Fixture id | Input profile | Embedded ICC | Working space | Output profile | Transform path | Probe status | Original hash | Product state | Evidence |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `srgb_jpeg` | `silicaraw_synthetic_srgb_jpeg` | `srgb` | true | `linear_display_p3` | `srgb` | `embedded_icc_to_linear_display_p3_to_srgb` | success | unchanged | `profile_probe_supported` | Task 13.4 local ignored manifest probe on macOS |
| `display_p3_jpeg` | `silicaraw_synthetic_display_p3_jpeg` | `display_p3` | true | `linear_display_p3` | `srgb` | `embedded_icc_to_linear_display_p3_to_srgb` | success | unchanged | `profile_probe_supported` | Task 13.4 local ignored manifest probe on macOS |
| `untagged_jpeg` | `silicaraw_synthetic_untagged_jpeg` | `none` | false | `linear_display_p3` | `srgb` | `assume_srgb_to_linear_display_p3_to_srgb` | success | unchanged | `assume_srgb_profile_probe_supported` | Task 13.4 local ignored manifest probe on macOS |

Blocked states remain explicit:

| Area | State | Reason |
| --- | --- | --- |
| Color correctness | `blocked_pending_tolerance_and_visual_review` | No approved pixel or perceptual comparison and no manual visual QA record. |
| Transform output appearance | `blocked_pending_tolerance_and_visual_review` | ICC/profile inspection does not prove rendered pixel correctness. |
| Display P3 user-facing option | `explicit_export_option_available` | Task 13.8 wires Display P3 as an explicit ICC-backed JPEG export option only. |
| Committed fixture corpus | `blocked_pending_redistribution_review` | Local macOS profile-derived fixture files remain ignored by git. |

## Phase 13.6 ICC Export Proof

Task 13.6 proves JPEG ICC embedding by file/profile inspection. It does not prove visual color correctness.

| Export target | Entry point | Embedded ICC | ICC SHA-256 | Output hash | Original hash | Product state | Evidence |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `srgb` | `export_jpeg_srgb` | true | `2b3aa1645779a9e634744faf9b01e9102b0c9b88fd6deced7934df86b949af7e` | recorded in `JpegExportResult.output_sha256` | unchanged in tests | `default_export_icc_supported` | `cargo test -p silica-export -p silica-render` |
| `display_p3` | `export_jpeg_with_color_profile(... DisplayP3)` | true | recorded from active macOS system profile | recorded in `JpegExportResult.output_sha256` | unchanged in tests | `explicit_api_proof_only` | `cargo test -p silica-export -p silica-render` |

The local Display P3 baseline profile hash was `0ff6958f98684c61f6bbdce1368ddeaf3873baf84545baba482e920d92a914c0`, but macOS runner images may ship byte-different Display P3 ICC profiles. SilicaRAW classifies sRGB and Display P3 profiles by known hashes and RGB XYZ tag semantics, then records the actual embedded ICC SHA-256 in export evidence.

Manual visual review is ready but not executed:

- [Color Export Manual QA Checklist](../../../checklists/COLOR_EXPORT_MANUAL_QA.md)

User-facing Display P3 export stays limited to an explicit ICC-backed output option; visual color correctness remains blocked.

## Phase 13.7 Color Metadata Contract

Task 13.7 keeps color metadata in existing schema-owned fields:

```txt
edit_graph.profile.input_profile
edit_graph.profile.working_space
edit_graph.profile.decoder_backend
exports.export_settings_json.color_profile
exports.export_settings_json.source_sha256
exports.export_settings_json.source_sha256_after_export
exports.export_settings_json.source_original_hash_unchanged
exports.export_settings_json.output_sha256
exports.export_settings_json.icc_profile_embedded
exports.export_settings_json.icc_profile_sha256
exports.export_settings_json.profile_metadata_source
exports.export_settings_json.decoder_backend
exports.export_settings_json.input_profile
exports.export_settings_json.working_space
```

Default edit graphs record `input_profile = "unknown"` and `decoder_backend = null`. Evidence-backed raster profile updates use `decoder_backend = "raster"`.

No new edit graph schema fields were added.

## Phase 13.8 Explicit Export Color Options

Task 13.8 wires the proven export profiles into the local desktop boundary:

```txt
sRGB -> default JPEG export path
Display P3 -> explicit JPEG export choice using embedded ICC evidence
unknown profile string -> exportBlocked
```

The UI/API distinction is deliberate. Omitted color profile means default sRGB. `display_p3` must be selected or passed explicitly. No other color-space labels silently run.

This task does not prove visual color correctness; it only connects the already-proven ICC embedding path to product controls.

## Phase 15.5 RAW-Derived JPEG sRGB Export

Task 15.5 keeps RAW-derived export claims evidence-limited:

```txt
Core Image RAW fixture evidence -> full-resolution RAW export source artifact
full-resolution source artifact -> JPEG sRGB export path
JPEG sRGB export path -> embedded sRGB ICC and recorded ICC/profile/hash evidence
```

The export record stores source SHA-256, output SHA-256, ICC embedded state, ICC profile SHA-256, decoder backend, input profile, working space, and `profile_metadata_source`. This proves ICC/profile evidence for the fixture-backed RAW export path. It still does not prove broad visual color correctness.

## Phase 15.6 RAW Export Manual QA

[Phase 15 RAW Export Manual Color QA](../../../checklists/PHASE_15_RAW_EXPORT_MANUAL_QA.md) records one Preview.app review of a fixture-backed RAW-derived JPEG sRGB export.

The review passed the evidence-limited gate for Preview.app opening, embedded sRGB ICC evidence, source/output hash recording, and original source hash preservation. Broad visual color correctness remains blocked.

## Phase 15 Entry Gate

Task 15.0 rechecked local ignored Color Class F profile-probe evidence and requires all three Class F subclasses for Phase 15 ICC/profile regression:

```txt
srgb_jpeg -> profile probe success, embedded ICC, original hash unchanged
display_p3_jpeg -> profile probe success, embedded ICC, original hash unchanged
untagged_jpeg -> profile probe success, assume_srgb path, original hash unchanged
```

This gate does not prove visual color correctness, transform-output appearance, camera profile behavior, or parity with Preview.app, Photos, Lightroom, Capture One, or camera vendor renderers.

## Color-Dependent Tags

```txt
Color Baseline: color-blocking
Preview Transform: color-blocking
Export ICC: color-blocking
Golden Images: color-dependent
RAW Camera Profiles: decoder-dependent color-dependent
```

## Links

- [Spike 003 Report](../../spikes/003-color-managed-preview-export.md)
- [Phase 13 Color Pipeline Proof Plan](../roadmaps/phase-13-color-pipeline-proof-plan.md)
- [Synthetic Local Color Fixture Source Review](../sources/color-fixtures-synthetic-local.md)
- [Phase 15 Vertical Slice Evidence Gate](../../../checklists/PHASE_15_VERTICAL_SLICE_EVIDENCE.md)
- [Color Management Specification](../../09_Color_Management_Specification.md)
- [Testing and QA Plan](../../15_Testing_QA_Plan.md)
- [Architecture Patch](../../20_v1_1_Architecture_Patch.md)
- [RAW Decoding](raw-decoding.md)

## Notes for LLM Agents

Do not claim color correctness from Spike 003 or compile success. Spike 003 records an implementation direction, not fixture-backed color proof.
