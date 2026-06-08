# Initial GitHub Issue List

## Feasibility

1. [repo] Create monorepo structure
2. [app] Add Tauri desktop shell
3. [ci] Add fmt/lint/test baseline
4. [docs] Add architecture guardrails
5. [spike] Tauri + Metal viewer
6. [spike] RAW decode comparison
7. [spike] Color-managed preview/export
8. [spike] SQLite catalog persistence
9. [spike] MLX runtime

## Design Foundation

10. [ui] Implement design tokens
11. [ui] Build component gallery
12. [ui] Implement app frame components
13. [ui] Implement SrAdjustmentSlider

## Catalog

14. [storage] SQLite migration foundation
15. [storage] Initial catalog schema
16. [catalog] Folder scanner
17. [library] Virtualized grid with mock thumbnails
18. [library] Rating/reject/pick persistence
19. [cache] Thumbnail cache v0

## Rendering

20. [render] Integrate Metal viewer
21. [render] Render request model
22. [render] TextureManager v0
23. [render] Exposure/contrast shader pass
24. [viewer] Before/after composite

## Develop

25. [edit] Edit graph types
26. [storage] Active edit state storage
27. [develop] Basic panel v0
28. [render] Basic Metal adjustment pass v0
29. [edit] Undo/redo for edit changes

## Export

30. [export] Export dialog UI
31. [export] JPEG export v0
32. [export] sRGB/ICC export v0
33. [export] Batch export queue

---

# v1.1 Added Critical Issues

## Critical Patch Issues

```txt
[critical] Add Tauri + Metal fallback decision record
[critical] Add RAW decoder decision gate
[critical] Implement edit graph v0.1 schema validation
[critical] Add SQLite indexes in initial migration
[critical] Add benchmark fixture reporting
[critical] Add dependency approval policy
[critical] Finalize provisional license strategy before architecture gate
```

## Milestone Binding

```txt
M0 Feasibility:
- Tauri + Metal viewer spike
- RAW decode comparison spike
- Color path spike
- SQLite persistence spike
- MLX runtime spike
- Provisional license decision

M1 Foundation:
- Monorepo
- Tauri shell, only if Spike 001 path A/B
- SwiftUI/AppKit shell if Spike 001 path C
- Dependency policy
- Schema validation tests

M2 Storage:
- Initial schema
- Indexes
- Edit graph v0.1
- Sidecar v0.1
```
