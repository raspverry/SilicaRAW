# 06 — SilicaRAW Screen Inventory & Wireframe Specification v1.1

Status: GO WITH CONDITIONS

## Core UX Model

```txt
Library → Develop → Export
```

## Global App Frame

```txt
Toolbar
├─ Sidebar
├─ Main Content / Viewer
└─ Inspector
Bottom: optional filmstrip/status/progress
```

## Responsive Tiers

### Compact Desktop: 1280–1439px

- Sidebar collapsible
- Inspector remains available in Develop
- Filmstrip can auto-hide
- Search collapses to icon
- Toolbar uses overflow menu

### Standard Desktop: 1440–1719px

- Sidebar visible
- Inspector visible
- Filmstrip visible by default in Develop

### Large Desktop: 1720px+

- Expanded inspector
- Compare/survey views more comfortable
- Filmstrip bottom or left optional

## Screen Inventory

| ID | Screen | Priority |
|---|---|---|
| S001 | Welcome / Empty State | P0 |
| S002 | Library Grid | P0 |
| S003 | Library Loupe | P0 |
| S004 | Develop | P0 |
| S005 | Export Dialog / Panel | P0 |
| S006 | Preferences | P0 |
| S007 | Import Progress | P0 |
| S008 | Compare View | P1 |
| S009 | Before / After View | P1 |
| S010 | Preset Manager | P1 |
| S011 | Collection Manager | P1 |
| S012 | Batch Edit / Sync Edits | P1 |
| S013 | AI Review | P2 |
| S014 | Mask Editor | P2 |
| S015 | Plugin Manager | P3 |
| S016 | MCP Agent Access | P3 |
| S017 | Action Log | P3 |
| S018 | Model Manager | P3 |
| S019 | About / Credits | P0 |

## Develop Inspector Order

1. Histogram, always top
2. Basic, open by default
3. Tone, collapsed
4. Color, collapsed
5. Detail, collapsed
6. Lens, collapsed
7. Geometry, collapsed
8. Mask, collapsed unless active
9. Metadata, collapsed
10. Export, compact action or toolbar

## Required Screen States

- Empty
- Loading
- Partial loading
- Error
- Missing file
- Unsupported RAW
- Permission denied
- AI model unavailable, where applicable

## Required High-Fidelity Mockups

- Welcome
- Library Grid empty
- Library Grid populated
- Library Loupe
- Develop default
- Develop mask active
- Export Dialog
- Preferences Appearance
- Import Progress
- AI Review

Mock up Library, Develop, Export at 1280, 1440, and 1728px.

## Implementation Order

1. Component gallery
2. Global app frame
3. Welcome + Library grid
4. Loupe + culling
5. Develop MVP
6. Export
7. Compare/preset/batch/AI/MCP advanced screens

## Final Verdict

GO WITH CONDITIONS.

Do not implement full UI until component gallery, global frame, responsive behavior, and high-fidelity mockups are ready.
