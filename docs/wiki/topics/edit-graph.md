---
title: Edit Graph
status: active
audience: all
updated: 2026-06-13
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

## Links

- [Edit Graph Schema](../../../schemas/edit_graph.schema.json)
- [Edit Graph Example](../../../schemas/edit_graph.example.json)
- [Schema Reference](../../19_Schema_Reference.md)
- [Data Model and Storage Specification](../../10_Data_Model_and_Storage_Specification.md)
- [Action Trust](action-trust.md)

## Notes for LLM Agents

Do not invent an alternate edit graph. Do not place experimental top-level fields beside schema-owned fields; use `extensions`.

Phase 5.3 adds the first exposure/contrast graph update path and commit boundary. Phase 13.7 adds the color metadata contract for schema-owned `profile` fields. Task 15.4 validates exposure/contrast Metal draft requests through the edit graph validator while keeping slider drafts out of catalog history; commit remains the only path that writes the validated edit graph. Task 16.0 keeps undo/redo scoped to validated catalog checkpoints, not external files or draft previews. Task 16.1 defines one edit checkpoint as one committed Develop action with explicit before/after graph state. Task 16.2 persists the first durable checkpoint rows for committed exposure/contrast edits while keeping drafts non-persistent. Tasks 17.1.1 through 17.1.3 add graph-only mutation for the Phase 17 basic control families. Tasks 17.2.1 through 17.2.3 add white-balance-family, tone-recovery, and color-presence preview/commit/export parity for supported JPEG/JPG paths and export evidence. Full render integration, sidecar persistence, broad RAW decoding, MLX, MCP, and plugin behavior remain separate explicit tasks.
