# SilicaRAW Agent Rules

## Always Follow

1. One task at a time.
2. Respect scope and out-of-scope.
3. Do not change architecture silently.
4. Do not modify original photo files.
5. Do not add network, telemetry, or cloud sync.
6. Do not add AI/MCP/plugin features unless explicitly requested.
7. Use design tokens for UI.
8. Add tests where feasible.
9. Document assumptions.
10. Keep PRs small.

## Forbidden Without Explicit Approval

- Original file deletion/modification
- Raw SQL from plugin/MCP layer
- Arbitrary executable plugins
- MCP dangerous tools
- New dependency without license/reason
- Hardcoded UI color/radius/spacing
- Cloud upload of photos/metadata

## Review Questions

- Did this task do only what it was asked to do?
- Are originals safe?
- Are tests added?
- Are docs updated?
- Does this preserve the architecture?

---

# v1.1 Added Rules

## Dependency Documentation

If you add any dependency, update:

```txt
docs/DEPENDENCIES.md
```

No exceptions.

## Edit Graph Schema

Do not design your own edit graph. Use:

```txt
schemas/edit_graph.schema.json
```

## Doc 06 Source of Truth

Use only:

```txt
docs/06_Screen_Inventory_and_Wireframe_Specification.md
```

Do not use archived 06 files.

## Tauri + Metal Spike

If Spike 001 fails, stop and report. Do not proceed with full UI implementation.

## RAW Decoder Spike

If Spike 002 does not select a decoder path, do not implement decoder-dependent features.
