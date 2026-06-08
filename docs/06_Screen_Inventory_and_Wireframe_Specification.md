# 06 — SilicaRAW Screen Inventory & Wireframe Specification v1.1

Status: GO WITH CONDITIONS  
Authority: SINGLE SOURCE OF TRUTH FOR SCREEN INVENTORY, RESPONSIVE LAYOUT, AND MOCKUP REQUIREMENTS.  
Supersedes old v1.0 / patch variants now archived under `docs/archive/`.

---

## 1. Purpose

This document defines the complete screen inventory, screen-level UX rules, wireframe structure, responsive behavior, state handling, focus order, and implementation-readiness criteria for SilicaRAW.

SilicaRAW is a photo editor first. Every screen must support:

```txt
Import / Browse / Cull
↓
Develop / Edit / Compare
↓
Export / Deliver
```

SilicaRAW must not drift into:

```txt
AI dashboard
SaaS dashboard
open-source utility
technical demo
```

---

## 2. Core UX Principle

```txt
Library / Develop / Export = primary modes
AI Review / Model Manager / MCP Access = secondary or advanced
```

AI appears as normal editing functionality: Auto Tone, Subject Mask, Sky Mask, Denoise.  
MCP lives only in Preferences → Advanced → Agent Access and is OFF by default.

---

## 3. Global App Frame

```txt
┌─────────────────────────────────────────────────────────────┐
│ Unified Toolbar                                             │
├───────────────┬─────────────────────────────┬───────────────┤
│ Left Sidebar  │ Main Content / Viewer       │ Right Inspector│
├───────────────┴─────────────────────────────┴───────────────┤
│ Optional Filmstrip / Status / Progress Area                  │
└─────────────────────────────────────────────────────────────┘
```

Rules:

```txt
Left Sidebar = navigation / folders / collections / presets.
Center = photo content.
Right Inspector = metadata / histogram / editing / export.
Bottom = filmstrip / status / progress.
AI is never primary navigation.
MCP is never primary toolbar item.
```

---

## 4. Responsive Layout Rules

### Compact Desktop — 1280–1439px

```txt
Target: 13-inch MacBook
Left sidebar collapsible
Right inspector remains visible in Develop
Filmstrip can auto-hide
Search collapses to icon
Secondary actions move into More menu
No horizontal scroll
```

### Standard Desktop — 1440–1719px

```txt
Target: 14-inch MacBook / small external display
Sidebar visible
Inspector visible
Bottom filmstrip visible in Develop
Search and Export visible
```

### Large Desktop — 1720px+

```txt
Target: 16-inch MacBook / external display
Expanded inspector
More grid columns
Filmstrip bottom or left optional
Compare / Survey views more comfortable
```

---

## 5. Layout Persistence

Persist:

```txt
Sidebar width
Inspector width
Sidebar collapsed state
Inspector collapsed state
Filmstrip visibility
Filmstrip position
Filmstrip height / width
Last selected mode
Last selected library
Grid thumbnail size
Sort state
Filter state
Collapsed inspector sections
Theme
Accent preference
```

Reset command: View → Reset Workspace Layout.

---

## 6. Toolbar Behavior

Default:

```txt
Left: Sidebar toggle, current source
Center: Library | Develop | Export
Right: Search, View options, Export, Settings, More
```

Compact:

```txt
Search collapses to icon
View options move into More
Settings move into More
Export remains reachable
```

---

## 7. Screen Inventory

| ID | Screen | Purpose | Priority |
|---|---|---|---|
| S001 | Welcome / Empty State | First launch, open folder, recent libraries | P0 |
| S002 | Library Grid | Browse and cull photos | P0 |
| S003 | Library Loupe | Inspect one photo | P0 |
| S004 | Develop | Main RAW editing screen | P0 |
| S005 | Export Dialog / Panel | Export photos | P0 |
| S006 | Preferences | App settings | P0 |
| S007 | Import Progress | Import status | P0 |
| S008 | Compare View | Compare similar photos | P1 |
| S009 | Before / After View | Compare edit result | P1 |
| S010 | Preset Manager | Manage presets | P1 |
| S011 | Collection Manager | Manage collections | P1 |
| S012 | Batch Edit / Sync Edits | Sync edits | P1 |
| S013 | AI Review | Review AI suggestions | P2 |
| S014 | Mask Editor | Detailed masks | P2 |
| S015 | Plugin Manager | Manage plugins | P3 |
| S016 | MCP Agent Access | Agent access | P3 |
| S017 | Action Log | AI/MCP/plugin logs | P3 |
| S018 | Model Manager | MLX models | P3 |
| S019 | About / Credits | Version/license | P0 |

---

## 8. Required State Matrix

All applicable screens must define:

```txt
Empty
Loading
Partial loading
Error
Missing file
Unsupported RAW
Permission denied
AI model unavailable
Catalog unavailable
Preview generation failed
```

---

## 9. Screen Requirements

### S001 Welcome

Must show:

```txt
SilicaRAW
Open-source RAW photo editor for Apple Silicon
Open Folder
Open Recent
Open Sample Project
Built for macOS · Metal-first · Local-first
```

### S002 Library Grid

Must show toolbar, left navigation, virtualized thumbnail grid, right inspector, histogram, metadata, rating, quick adjust, AI culling suggestions as secondary, bottom photo count and thumbnail size.

### S003 Library Loupe

Must show one large photo, neutral dark viewer, file name, rating, pick/reject, fit/100%, metadata inspector, bottom filmstrip.

### S004 Develop

Must show:

```txt
Develop selected
Left: Presets / History / Snapshots
Center: large edited photo
Right inspector:
1. Histogram, always top
2. Basic, open
3. Tone, collapsed
4. Color, collapsed
5. Detail, collapsed
6. Lens, collapsed
7. Geometry, collapsed
8. Mask, collapsed unless active
9. Metadata, collapsed
10. Export compact
Bottom filmstrip where layout allows
```

Develop density:

```txt
Inspector default width: 320px
Compact inspector: ~280px
Slider row: 44px
Histogram: 120px compact / 160px expanded
```

### S005 Export Dialog

Must show:

```txt
Export Photos
Selected photo count
Preset
Destination
File Naming
Format: JPEG / PNG / TIFF / HEIC / WebP
Quality
Resize
Color Space: sRGB / Display P3 / Adobe RGB / ProPhoto RGB
Embed ICC Profile
Metadata: Preserve / Remove GPS / Remove all
Preview thumbnail
Summary
Estimated file size
Save Preset
Cancel
Export
Original files will not be modified
```

### S006 Preferences → Appearance

Must show theme, accent, UI preview, sidebar behavior, compact behavior, filmstrip position, reset workspace layout.

### S007 Import Progress

Must show folder path, original files stay in place, overall progress, scanning, metadata, thumbnails, previews, unsupported detection, errors, Pause, Cancel, View Errors, Minimize.

### S013 AI Review

Must show AI Review, “Suggestions only. Nothing changes until you apply.”, tabs Blur/Duplicates/Quality/Masks, reason, confidence, Keep/Reject/Review, summary, criteria, action preview, Apply Decisions.

---

## 10. High-Fidelity Mockup Requirements

Required:

```txt
M001 Welcome
M002 Library Grid empty
M003 Library Grid populated
M004 Library Loupe
M005 Develop default
M006 Develop mask active
M007 Export Dialog
M008 Preferences Appearance
M009 Import Progress
M010 AI Review
M011 M003 compact 1280
M012 M003 large 1728
M013 M005 compact 1280
M014 M005 large 1728
M015 M007 compact 1280
M016 M007 large 1728
```

---

## 11. Screen QA Checklist

```txt
[ ] Uses approved components
[ ] Uses design tokens
[ ] Photo content remains primary
[ ] Keyboard shortcuts work
[ ] Empty/loading/error states exist
[ ] Missing/unsupported file states exist
[ ] Responsive behavior defined
[ ] Focus order defined
[ ] Reduced motion respected
[ ] AI/MCP not primary unless advanced screen
[ ] Works in dark mode
[ ] Does not break in light mode
```

---

## 12. Final Decision

```txt
06 Screen Inventory & Wireframe Specification v1.1:
GO WITH CONDITIONS
```

This is the only authoritative screen/wireframe document. Archived 06 files must not be used for implementation.
