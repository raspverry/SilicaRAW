# SilicaRAW Codex Handoff

You are implementing SilicaRAW, an open-source RAW photo editor built for Apple Silicon.

## Read First

1. `MANIFEST.md`
2. `codex/AGENT_RULES.md`
3. `docs/20_v1_1_Architecture_Patch.md`
4. `docs/19_Schema_Reference.md`
5. `schemas/edit_graph.schema.json`
6. `schemas/edit_graph.example.json`
7. `docs/18_Final_Master_Plan.md`
8. `docs/13_Development_Roadmap.md`
9. `docs/14_Codex_Claude_Task_Breakdown.md`
10. `github/ISSUE_LIST.md`

## Product Identity

SilicaRAW is a RAW photo editor first. AI/MLX and MCP are secondary.

## Start Here

Do not build the whole app. Start with these tasks:

1. Create monorepo structure
2. Add Tauri desktop shell
3. Add CI/fmt/lint/test baseline
4. Add architecture guardrails
5. Spike: Tauri + Metal viewer
6. Spike: RAW decode comparison
7. Spike: color-managed preview/export
8. Spike: SQLite catalog persistence
9. Spike: MLX runtime

## Hard Rules

- Never modify original photo files.
- Do not add cloud sync or telemetry.
- Do not add AI/MCP early.
- Do not hard-code UI colors outside tokens.
- Do not bypass Rust Core for storage/render/edit operations.
- Do not introduce major dependencies without documenting license and reason.

## First Codex Prompt

Implement Task 0101: Create the monorepo structure.

Scope:
- Create root Rust workspace.
- Create `apps/desktop` placeholder.
- Create crates listed in the architecture.
- Add README files for each crate.
- Add root README skeleton.

Out of scope:
- No RAW decoding.
- No Metal viewer.
- No UI screens.
- No MLX/MCP/plugin implementation.

Acceptance:
- Workspace builds.
- Crate structure matches docs.
- README files explain crate responsibilities.

---

# v1.1 Critical Handoff Addendum

Before writing product code, Codex must read:

```txt
docs/20_v1_1_Architecture_Patch.md
docs/19_Schema_Reference.md
docs/DEPENDENCIES.md
schemas/edit_graph.schema.json
schemas/edit_graph.example.json
```

## Spike 001 fallback rule

If Tauri + Metal viewer integration fails, do not continue implementing the Tauri shell as if it succeeded.

Record outcome:

```txt
A. Tauri + native Metal viewer works.
B. Tauri shell works but needs stronger native Metal subview bridge.
C. Tauri is unsuitable; switch to SwiftUI/AppKit shell + Rust Core.
```

Metal-first editor identity takes priority over Tauri.

## RAW decoder gate

Do not implement full v1 RAW features until Spike 002 documents which decode path is used:

```txt
Core Image RAW primary
LibRaw primary
Hybrid
```

Tag decoder-dependent work clearly.

## Edit graph rule

Do not invent edit graph structure. Use:

```txt
schemas/edit_graph.schema.json
schemas/edit_graph.example.json
```

## Dependency rule

Any new dependency must be recorded in:

```txt
docs/DEPENDENCIES.md
```

---

# v1.3 Handoff Note

Before implementing storage structs, read the v1.3 clarifications in:

```txt
docs/19_Schema_Reference.md
docs/10_Data_Model_and_Storage_Specification.md
schemas/sidecar.schema.json
```

Important:

```txt
photo_flags in SQLite is authoritative inside the app.
sidecar.flags is the latest portable mirror for recovery.
edit_graph.metadata is a portable snapshot/fallback.
```

Do not invent a separate meaning for these fields.
