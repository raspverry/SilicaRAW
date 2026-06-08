# 02 — SilicaRAW Product Requirements Document

Status: GO WITH CONDITIONS

## Product Summary

SilicaRAW is a macOS/Apple Silicon-first open-source RAW photo editor. It supports browsing, culling, non-destructive editing, color-managed export, and later MLX-assisted tools.

## Primary Goals

1. Open and browse RAW photo folders.
2. Cull photos with ratings, pick/reject, filters, and compare flows.
3. Edit RAW photos non-destructively.
4. Provide fast Metal-powered preview updates.
5. Export reliable color-managed files.
6. Keep originals untouched.
7. Support MLX and MCP later without compromising the editor core.

## Non-Goals for v1

- Cloud sync
- Mobile app
- Windows/Linux support
- Photoshop-style layers/text/vector tools
- Full object removal/generative fill
- Plugin marketplace
- Dangerous MCP tools
- Full HDR/print/tethering workflows

## Core Modes

```txt
Library → Develop → Export
```

## v1 Must Include

- Library grid
- Loupe viewer
- Ratings, reject, pick
- Folder import
- SQLite catalog
- Thumbnail/preview cache
- Develop screen
- Basic adjustments
- Tone curve
- HSL/color mixer
- Basic color grading
- Detail controls
- Crop/rotate
- Lens correction baseline
- Presets
- Undo/redo
- Before/after
- Copy/paste edits
- Batch sync
- Export JPEG/PNG/TIFF/HEIC where feasible
- sRGB and Display P3 export
- ICC embedding
- Metadata controls
- Catalog backups
- Sidecar JSON

## v1 May Include

- Auto Tone
- Subject Mask
- Sky Mask
- AI Review
- Read-only MCP
- Declarative preset plugins

## v1 Must Not Include

- Object removal
- Generative fill
- Cloud sync
- Plugin marketplace
- Arbitrary executable plugins
- Dangerous MCP operations

## Performance Requirements

- Smooth thumbnail grid with cached thumbnails
- Responsive slider preview via Metal
- No full-resolution re-render on every slider tick
- Export runs without freezing UI
- Catalog operations stay responsive for large folders

## Safety Requirements

- Original files never modified
- Cache deletion never deletes edits or originals
- Migration backup before destructive schema changes
- MCP/plugin actions permissioned and logged

## Final Verdict

GO WITH CONDITIONS.

The PRD is valid, but implementation depends on the Metal, color, storage, and RAW decode feasibility spikes.
