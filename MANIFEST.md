# SilicaRAW Final Codex Docs v1.2

Generated: 2026-06-08T02:45:46.417727Z

This v1.1 bundle incorporates external review fixes and is the recommended bundle to give to Codex / Claude Code.

## Critical v1.1 Changes

```txt
1. Added Tauri + Metal fallback strategy.
2. Added RAW decoder decision gate.
3. Added authoritative Edit Graph JSON Schema v0.1.
4. Added Sidecar / Plugin / Model / MCP schema files.
5. Consolidated doc 06 into a single source of truth.
6. Archived old doc 06 variants.
7. Added SQLite index requirements.
8. Added benchmark fixture specification.
9. Added license/dependency gates.
10. Added docs/DEPENDENCIES.md.
```

## Recommended Codex Reading Order

```txt
1. MANIFEST.md
2. codex/CODEX_HANDOFF.md
3. codex/AGENT_RULES.md
4. docs/20_v1_1_Architecture_Patch.md
5. docs/19_Schema_Reference.md
6. schemas/edit_graph.schema.json
7. docs/18_Final_Master_Plan.md
8. docs/13_Development_Roadmap.md
9. docs/14_Codex_Claude_Task_Breakdown.md
10. github/ISSUE_LIST.md
```

## Single Source Notes

```txt
docs/06_Screen_Inventory_and_Wireframe_Specification.md is authoritative.
docs/archive/ contains previous 06 variants and must not be used for implementation.
schemas/ contains authoritative JSON schemas.
```

## First Codex Task

```txt
Task 001:
Create monorepo structure.

Do not implement RAW decoding.
Do not implement Metal viewer.
Do not implement MLX/MCP.
Do not implement UI screens.
Do not add dependencies without updating docs/DEPENDENCIES.md.
```


---

## v1.2 Patch

Generated: 2026-06-08T02:52:10.931333Z

Changes:
- Confirmed `schemas/` directory is included in the ZIP.
- Added docs/19 and docs/20 to CODEX_HANDOFF Read First list.
- Expanded docs/DEPENDENCIES.md with currently verified license notes and explicit verification sources.
- Kept version-specific dependency verification mandatory.

---

## v1.3 Patch

Generated: 2026-06-08T03:01:31.823921Z

Changes:
- Added sidecar.flags as required typed schema field.
- Clarified catalog photo_flags vs edit_graph.metadata vs sidecar.flags.
- Added schema versioning policy for future v2+ schemas.
- Marked intentionally loose schema fields to prevent Codex from inventing hidden formats.

---

## v1.4 Patch

Generated: 2026-06-08T03:07:22.759251Z

Changes:
- Clarified that `edited` and `exported` are catalog-only flags.
- Confirmed `sidecar.flags` contains only rating/picked/rejected/color_label.
- Added `schemas/sidecar.example.json`.
- Added rebuild rules for `edited` and `exported`.
