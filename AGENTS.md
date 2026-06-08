# SilicaRAW Agent Instructions

## Project Goal

SilicaRAW is an Apple Silicon-first RAW photo editor. The current delivery target is a local macOS alpha that can be installed from a GitHub Release DMG and complete a minimal local editor workflow.

The local alpha workflow is:

```txt
Launch app
Create or open a local library
Import a folder by reference
Show a library grid
Rate, pick, or reject photos
Open a preview
Apply exposure/contrast
Persist edit state
Export JPEG sRGB
Verify original files are unchanged
```

## Read First

Before implementation work, read:

1. `MANIFEST.md`
2. `codex/AGENT_RULES.md`
3. `docs/wiki/index.md`
4. `docs/wiki/roadmaps/local-dmg-distribution-plan.md`
5. `docs/20_v1_1_Architecture_Patch.md`
6. `docs/19_Schema_Reference.md`
7. `schemas/edit_graph.schema.json`

## Scope Rules

- Keep tasks atomic and committable.
- Prefer the documented crate boundaries in `docs/03_System_Architecture.md`.
- Do not implement broad product features ahead of the roadmap.
- Do not add dependencies without updating `docs/DEPENDENCIES.md`.
- Do not modify original photo files.
- Do not use archived screen spec files under `docs/archive/` for implementation.

## Early Alpha Exclusions

Do not add these unless the user explicitly changes scope:

- MLX runtime or model loading
- MCP server or tools
- Plugin runtime
- Cloud sync
- Telemetry or analytics
- Auto-update
- Homebrew distribution
- Mac App Store distribution

## Testing Expectations

Use the smallest useful verification for the change.

Always run relevant checks before claiming completion:

```bash
scripts/harness/check.sh
```

For code changes that cannot run the full harness, state the narrower command used and why.

## Fallback Policy

Do not build large fallback systems preemptively. Record a gate result, choose one path, and keep alternatives as documented decisions until evidence requires a switch.

