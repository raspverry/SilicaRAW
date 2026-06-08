# 14 — SilicaRAW Codex / Claude Code Task Breakdown

Status: GO WITH CONDITIONS

## Principle

One task, one small deliverable, one review gate.

## Global Agent Rules

- Do not modify original photo files.
- Do not add AI/MCP unless task explicitly requests it.
- Do not hard-code UI colors outside tokens.
- Do not add telemetry/network/cloud sync.
- Do not change architecture silently.
- Add tests where feasible.
- Respect out-of-scope sections.

## Initial Task Order

001. Create monorepo structure
002. Add Tauri desktop shell
003. Add CI/fmt/lint/test baseline
004. Add architecture guardrails doc
005. Spike: Tauri + Metal viewer
006. Spike: RAW decode comparison
007. Spike: color-managed preview/export
008. Spike: SQLite catalog persistence
009. Spike: MLX runtime
010. Implement design tokens
011. Implement component gallery
012. Implement app frame
013. Implement SQLite migration foundation
014. Implement initial catalog schema
015. Implement folder scanner
016. Implement Library grid with mock thumbnails
017. Implement rating/reject/pick persistence
018. Implement thumbnail cache v0
019. Integrate Metal viewer into app frame
020. Implement render request model
021. Implement TextureManager v0
022. Implement exposure/contrast shader pass
023. Implement edit graph types
024. Implement active edit state storage
025. Implement Basic panel v0
026. Implement undo/redo
027. Implement export dialog
028. Implement JPEG export v0
029. Implement sRGB/ICC export v0
030. Implement batch export queue

## PR Review Checklist

- Scope matches task
- No unrelated files changed
- No architecture changes without approval
- Tests added or rationale provided
- No hardcoded design values
- No original file mutation
- No network/telemetry added
- Docs updated if behavior changed
- Build passes

## Final Verdict

GO WITH CONDITIONS.

Need dependency approval policy, QA plan, release plan, OSS docs.
