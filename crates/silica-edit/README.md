# silica-edit

Edit graph boundary for SilicaRAW.

This crate contains the Phase 5.2 typed edit graph structures for
`schemas/edit_graph.schema.json` and a schema-aware validator.

Implemented:

- Round-trip serialization for `schemas/edit_graph.example.json`.
- Closed Rust structs for schema sections that disallow unknown fields.
- `extensions` as the explicit place for experimental top-level data.
- Validation for the schema marker, version, enum deserialization, and numeric ranges.

Not implemented:

- Edit application or render integration.
- Undo/redo history.
- Sidecar storage.
- UI controls.
- RAW decoding, Metal viewer, MLX, MCP, or plugin behavior.
