---
title: Edit Graph
status: active
audience: all
updated: 2026-06-16
source_of_truth: schemas/edit_graph.schema.json
---

# Edit Graph

## Summary

The edit graph is the authoritative portable structure for non-destructive edit state. Its shape is defined by JSON Schema, not by ad hoc implementation choices.

## Current Stance

- Use `schemas/edit_graph.schema.json`.
- Use `schemas/edit_graph.example.json` for example shape.
- `crates/silica-edit` implements the Phase 5.2 typed Rust structures and validation boundary.
- Phase 5.3 adds default graph construction and exposure/contrast graph updates.
- Serialization must continue to round-trip `schemas/edit_graph.example.json`.
- JSON validation must reject wrong schema/version values, closed-object unknown fields, invalid enum values, and out-of-range numeric values.
- Unknown experimental data belongs under `extensions`.
- Color profile state belongs in schema-owned `profile` fields, not hidden top-level or extension fields.
- Phase 16 action trust treats validated edit graph commits as undoable catalog checkpoints. Draft preview updates remain render-only and must not create edit history rows.
- Task 16.1 defines one edit checkpoint as one committed Develop release containing explicit before/after schema-valid edit graph state.
- Task 16.2 persists one edit history checkpoint per committed exposure/contrast edit and validates the before/after graphs on write and reopen tests.
- Task 17.1.1 adds a typed white-balance-family mutator for `white_balance`, `temperature`, and `tint`; it remains graph-only and does not add preview, export, UI, storage, or sidecar behavior.
- Task 17.1.2 adds a graph-only tone recovery mutator for `highlights`, `shadows`, `whites`, and `blacks`.
- Task 17.1.3 adds a graph-only color presence mutator for `vibrance` and `saturation`.
- Task 17.2.1 wires committed white-balance-family state through JPEG/JPG Develop preview, undoable commit history, and JPEG export settings.
- Task 17.2.2 wires committed tone recovery state through JPEG/JPG Develop preview, undoable commit history, and JPEG export settings.
- Task 17.2.3 wires committed color presence state through JPEG/JPG Develop preview, undoable commit history, and JPEG export settings.
- Task 17.4 adds validated P0 Basic reset and built-in preset graph helpers. Presets and reset preserve source/profile/metadata/extensions and only change P0 Basic controls.
- Task 18.1.1 adds a graph-only tone curve mutator for `tone.curve_mode`, `rgb_curve`, `red_curve`, `green_curve`, and `blue_curve`. Non-empty curves require endpoints and strictly increasing `x` values. `Parametric` mode remains rejected by the mutator until schema-owned parameters exist.
- Task 18.1.2 wires point tone curve state through JPEG/JPG Develop preview, undoable commit history, desktop commands, and JPEG export settings. Parametric tone curve runtime behavior remains explicitly blocked instead of approximated or ignored.
- Task 18.2.1 adds graph-only HSL color mixer mutation for red, orange, yellow, green, aqua, blue, purple, and magenta channels. Each channel validates schema-owned hue, saturation, and luminance ranges without preview/export/color-correctness claims.
- Task 18.2.2 wires HSL color mixer state through supported JPEG/JPG Develop preview, undoable commit history, desktop commands, histogram/export parity, and JPEG export settings. This is deterministic local pixel adjustment, not a broad color-correctness claim.
- Task 18.3.1 adds graph-only Detail mutation for sharpening amount/radius/detail/masking and non-MLX noise reduction luminance/detail/contrast/color/color_detail values. `detail.mlx_denoise` remains unchanged and inert.
- Task 18.3.2 defines the Detail runtime boundary: non-neutral Detail preview is unsupported with no preview bytes, Detail commit is blocked with `UnsupportedEdit`, and active committed non-neutral Detail export/histogram is blocked rather than ignored. Neutral Detail state still flows through request and desktop response contracts.
- Task 18.4.1 adds graph-only lens and geometry mutators for lens toggles, distortion, vignetting, transform, rotation, flips, crop, and crop clearing. Crop rectangles must stay within the normalized frame. No lens profile database, preview/export behavior, UI, storage, sidecar, or original metadata mutation is added.
- Task 18.5.1 adds a typed edit clipboard contract for schema-owned subsets: `basic`, `tone`, `color`, `detail`, `lens`, and `geometry`. Clipboard payloads never carry `source`, `profile`, `metadata`, `masks`, or `extensions`. Detail clipboard data excludes `detail.mlx_denoise`; lens clipboard data excludes source-specific `lens.profile_id`. Applying a payload preserves the target graph identity and remains graph-only with no catalog, sidecar, UI, plugin, or MCP mutation path.
- Task 18.5.2 applies typed edit clipboard payloads through Core batch planning and storage all-or-none commit. Unsupported detail, lens, geometry, and Basic runtime-only fields are blocked before writes; successful sync preserves each target graph identity and records one undoable catalog checkpoint per changed photo.
- Task 18.5.3 exposes edit clipboard copy, paste-to-primary, and batch sync in the Develop UI with explicit selected-page scope, JPEG/JPG Develop target gating, and disabled unsupported clipboard subsets.
- Task 19.1 completes the mask schema audit before mask behavior is added. Manual gradient masks use schema-owned `masks[].geometry`, not hidden `mask.source` properties. `mask.source.kind = "manual"` is provenance-only; AI/model/cache fields stay reserved for future non-manual provenance. Brush durable data remains deferred to Task 19.3 and raster/cache bytes must not become the source of truth.

## Required Sections

- `source`
- `profile`
- `basic`
- `tone`
- `color`
- `detail`
- `lens`
- `geometry`
- `masks`
- `metadata`
- `extensions`

## Color Profile Contract

The profile object carries color metadata:

```txt
profile.input_profile -> explicit profile name, or "unknown" when unavailable
profile.working_space -> "linear_display_p3" for the current pipeline
profile.decoder_backend -> raster/core_image_raw/libraw/embedded_preview/null
```

Default edit graphs use `input_profile = "unknown"` and `decoder_backend = null`. Evidence-backed updates use `crates/silica-edit` profile helpers to write these schema-owned fields directly.

## Mask Contract

`masks[]` is the durable edit graph location for mask metadata and graph-owned manual gradient geometry.

```txt
masks[].type -> brush | linear_gradient | radial_gradient | future AI/procedural selection types
masks[].source.kind -> manual | mlx | procedural
masks[].geometry -> schema-owned linear/radial normalized geometry when type is linear_gradient or radial_gradient
masks[].local_adjustments -> numeric local adjustment map
```

Task 19.1 decision:

- `linear_gradient` requires `geometry.kind = "linear_gradient"` with normalized `start_x`, `start_y`, `end_x`, and `end_y`; start and end must differ in `silica-edit` validation.
- `radial_gradient` requires `geometry.kind = "radial_gradient"` with normalized center/radii and bounded rotation.
- Non-gradient mask types must not carry `geometry` until their durable shape is explicitly defined.
- Manual sources must serialize as provenance-only `{ "kind": "manual" }`; durable geometry, brush data, cache paths, model identifiers, and AI result identifiers do not belong in manual source.
- `brush` remains an allowed enum value but has no committed manual brush durable payload until Task 19.3 defines it.

Task 19.2 runtime boundary:

- `linear_gradient` and `radial_gradient` masks can be created through graph-owned helpers, previewed on supported JPEG/JPG develop previews, and committed through undoable catalog history.
- Phase 19.2 supports only local `exposure` and `contrast` mask adjustments; unsupported adjustment keys or ranges must block before preview/commit.
- JPEG export blocks active masks until Task 19.4 adds export compositing, so export never silently ignores committed mask state.

## Links

- [Edit Graph Schema](../../../schemas/edit_graph.schema.json)
- [Edit Graph Example](../../../schemas/edit_graph.example.json)
- [Schema Reference](../../19_Schema_Reference.md)
- [Data Model and Storage Specification](../../10_Data_Model_and_Storage_Specification.md)
- [Action Trust](action-trust.md)

## Notes for LLM Agents

Do not invent an alternate edit graph. Do not place experimental top-level fields beside schema-owned fields; use `extensions`.

Phase 5.3 adds the first exposure/contrast graph update path and commit boundary. Phase 13.7 adds the color metadata contract for schema-owned `profile` fields. Task 15.4 validates exposure/contrast Metal draft requests through the edit graph validator while keeping slider drafts out of catalog history; commit remains the only path that writes the validated edit graph. Task 16.0 keeps undo/redo scoped to validated catalog checkpoints, not external files or draft previews. Task 16.1 defines one edit checkpoint as one committed Develop action with explicit before/after schema-valid edit graph state. Task 16.2 persists the first durable checkpoint rows for committed exposure/contrast edits while keeping drafts non-persistent. Tasks 17.1.1 through 17.1.3 add graph-only mutation for the Phase 17 basic control families. Tasks 17.2.1 through 17.2.3 add white-balance-family, tone-recovery, and color-presence preview/commit/export parity for supported JPEG/JPG paths and export evidence. Task 17.4 adds reset and built-in preset graph helpers while keeping before/after UI state outside catalog history. Tasks 18.1.1 and 18.1.2 add tone curve mutation plus supported JPEG/JPG runtime parity without claiming parametric, broad RAW, MLX, MCP, or plugin behavior. Tasks 18.2.1 and 18.2.2 add HSL color mixer graph mutation plus supported JPEG/JPG runtime parity without broad color-correctness claims. Tasks 18.3.1 and 18.3.2 add Detail graph mutation plus explicit unsupported runtime boundaries; no Detail pixel effect, commit, MLX denoise, model, or UI enablement is claimed yet. Tasks 18.4.1 through 18.4.3 add lens and geometry graph/runtime/UI behavior only for the supported subset. Task 18.5.1 defines graph-only edit clipboard payloads; Task 18.5.2 adds all-or-none catalog sync for the supported clipboard subset; Task 18.5.3 exposes that clipboard contract in the Develop UI with explicit selected-page scope, JPEG/JPG Develop target gating, and disabled unsupported subsets. Task 19.1 defines the manual gradient mask schema contract. Task 19.2 adds manual gradient preview/commit behavior while keeping brush, AI, MLX, full export compositing, and UI behavior out of scope until later Phase 19 tasks.
