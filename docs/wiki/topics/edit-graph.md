---
title: Edit Graph
status: active
audience: all
updated: 2026-06-09
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

## Links

- [Edit Graph Schema](../../../schemas/edit_graph.schema.json)
- [Edit Graph Example](../../../schemas/edit_graph.example.json)
- [Schema Reference](../../19_Schema_Reference.md)
- [Data Model and Storage Specification](../../10_Data_Model_and_Storage_Specification.md)

## Notes for LLM Agents

Do not invent an alternate edit graph. Do not place experimental top-level fields beside schema-owned fields; use `extensions`.

Phase 5.3 adds the first exposure/contrast graph update path and commit boundary. Pixel edit application, full render integration, sidecar persistence, product UI controls, RAW decoding, Metal viewer work, MLX, MCP, and plugin behavior remain separate explicit tasks.
