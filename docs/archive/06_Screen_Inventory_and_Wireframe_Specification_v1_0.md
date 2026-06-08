# 06 — SilicaRAW Screen Inventory & Wireframe Specification

Version: v1.0  
Status: GO WITH CONDITIONS  
Depends on:
- 04 UI/UX Design Specification
- 05 Design System Specification
- 05.5 Component Library Specification

---

## 1. Purpose

This document defines the complete screen inventory, layout structure, and wireframe-level composition for SilicaRAW.

The goal is to make sure every screen:

- Supports the core RAW photo editing workflow
- Feels like a macOS Apple Pro App
- Uses the approved design system
- Uses the approved component library
- Keeps AI and MCP secondary
- Can be broken into implementation tasks

This document is not a high-fidelity visual design file. It is a product and engineering bridge document.

---

## 2. Core UX Model

SilicaRAW is organized around three primary modes:

```txt
Library
Develop
Export
```

These modes map to the user's core workflow:

```txt
Import / Browse / Cull
↓
Edit / Compare / Refine
↓
Export / Deliver / Archive
```

Additional screens exist, but they must not compete with the three primary modes.

---

## 3. Global App Frame

All major screens use the same global app frame.

```txt
┌─────────────────────────────────────────────────────────────┐
│ Unified Toolbar                                             │
├───────────────┬─────────────────────────────┬───────────────┤
│ Left Sidebar  │ Main Content / Viewer       │ Right Inspector│
│               │                             │               │
├───────────────┴─────────────────────────────┴───────────────┤
│ Optional Filmstrip / Status / Timeline Area                  │
└─────────────────────────────────────────────────────────────┘
```

### Layout meaning

```txt
Top    = mode, commands, search, status
Left   = navigation / presets / collections
Center = photo content
Right  = contextual controls
Bottom = filmstrip, selection, progress, compare context
```

### Rules

- Left sidebar is never used for editing controls.
- Right inspector is never used for folder navigation.
- Center always prioritizes photo content.
- AI never becomes a primary app mode.
- MCP never appears in the main toolbar by default.
- Export is a primary workflow but not a permanent visual distraction.

---

## 4. Screen Inventory

## 4.1 Primary Screens

| ID | Screen | Purpose | Priority |
|---|---|---|---|
| S001 | Welcome / Empty State | First launch, open folder, recent libraries | P0 |
| S002 | Library Grid | Browse and cull photos | P0 |
| S003 | Library Loupe | Inspect single photo before editing | P0 |
| S004 | Develop | Main RAW editing screen | P0 |
| S005 | Export Dialog / Export Panel | Export one or many photos | P0 |
| S006 | Preferences | App-level settings | P0 |

## 4.2 Secondary Screens

| ID | Screen | Purpose | Priority |
|---|---|---|---|
| S007 | Import Progress | Show folder scan/import progress | P0 |
| S008 | Compare View | Compare similar photos | P1 |
| S009 | Before / After View | Compare original and edited photo | P1 |
| S010 | Preset Manager | Manage and organize presets | P1 |
| S011 | Collection Manager | Create/edit collections | P1 |
| S012 | Batch Edit / Sync Edits | Copy/paste or sync adjustments | P1 |

## 4.3 Advanced / Future Screens

| ID | Screen | Purpose | Priority |
|---|---|---|---|
| S013 | AI Review | Review AI culling/masking suggestions | P2 |
| S014 | Mask Editor | Detailed mask management | P2 |
| S015 | Plugin Manager | Manage plugins and preset packs | P3 |
| S016 | MCP Agent Access | Enable/disable agent access | P3 |
| S017 | Action Log | Review AI/MCP/plugin actions | P3 |
| S018 | Model Manager | Manage MLX models | P3 |
| S019 | About / Credits | App version, license, contributors | P0 |

---

## 5. S001 — Welcome / Empty State

### Purpose

Help first-time users start quickly without feeling overwhelmed.

### User goals

- Open a folder
- Open a recent library
- Try sample photos
- Understand what SilicaRAW is

### Wireframe

```txt
┌─────────────────────────────────────────────────────┐
│                                                     │
│                    SilicaRAW                        │
│     Open-source RAW photo editor for Apple Silicon  │
│                                                     │
│              [ Open Folder ]                        │
│              [ Open Recent ▾ ]                      │
│              [ Open Sample Project ]                │
│                                                     │
│      Built for macOS · Metal-first · Local-first    │
│                                                     │
└─────────────────────────────────────────────────────┘
```

### Components

```txt
SrEmptyState
SrButton
SrRecentLibraryList
SrAppBrandMark
```

### Rules

- No AI marketing as primary message.
- “RAW photo editor” must be clear.
- Recent libraries appear after first use.
- Sample project is optional but useful for GitHub demos.

### Acceptance criteria

```txt
[ ] User can open folder
[ ] User can open recent library
[ ] User can open sample project, if bundled
[ ] Empty state uses design tokens
[ ] Window feels macOS-native and quiet
```

---

## 6. S002 — Library Grid

### Purpose

Main browsing and culling screen.

### User goals

- Browse imported photos
- Rate photos
- Reject photos
- Filter by rating/status
- Open photo in Develop
- Start export

### Wireframe

```txt
┌──────────────────────────────────────────────────────────────┐
│ Sidebar ⌘B | Library Develop Export | Search | Export        │
├───────────────┬──────────────────────────────┬───────────────┤
│ Library       │ Sort: Capture Time  Filter   │ Histogram     │
│ - All Photos  │ ┌────┐ ┌────┐ ┌────┐ ┌────┐ │ Metadata      │
│ - Recent      │ │img │ │img │ │img │ │img │ │ Rating        │
│ - Favorites   │ └────┘ └────┘ └────┘ └────┘ │ Quick Adjust  │
│               │ ┌────┐ ┌────┐ ┌────┐ ┌────┐ │               │
│ Folders       │ │img │ │img │ │img │ │img │ │               │
│ Collections   │ └────┘ └────┘ └────┘ └────┘ │               │
└───────────────┴──────────────────────────────┴───────────────┘
```

### Components

```txt
SrToolbar
SrSidebar
SrThumbnailGrid
SrThumbnailCell
SrInspector
SrHistogram
SrMetadataRow
SrRatingControl
SrSearchField
SrFilterBar
```

### Required behavior

```txt
G = Grid
D = Develop selected photo
1–5 = rating
0 = clear rating
X = reject
P = pick
Cmd+F = search
Double click thumbnail = Loupe or Develop, based on preference
```

### Performance requirements

```txt
[ ] Virtualized grid
[ ] Smooth scrolling with 10,000 photos
[ ] Thumbnail cache used
[ ] Rating/reject updates immediately
```

### Acceptance criteria

```txt
[ ] User can browse photo grid
[ ] User can select single/multiple photos
[ ] User can rate/reject with keyboard
[ ] User can filter by rating/reject status
[ ] User can open Develop mode
[ ] UI uses component library only
```

---

## 7. S003 — Library Loupe

### Purpose

Inspect a single photo quickly without entering full Develop workflow.

### Wireframe

```txt
┌──────────────────────────────────────────────────────────────┐
│ Sidebar | Library Develop Export | Search | Export           │
├───────────────┬──────────────────────────────┬───────────────┤
│ Library       │                              │ Histogram     │
│ Folders       │          Large Photo          │ Metadata      │
│ Collections   │                              │ Rating        │
│               │                              │ Quick Adjust  │
├───────────────┴──────────────────────────────┴───────────────┤
│ [thumb][thumb][thumb][selected][thumb][thumb]                │
└──────────────────────────────────────────────────────────────┘
```

### Components

```txt
SrLoupeViewer
SrFilmstrip
SrInspector
SrToolbar
SrSidebar
```

### Required behavior

```txt
Space = Loupe
Arrow keys = next/previous
Double click = 100% zoom
Pinch = zoom
Two-finger pan = pan
D = Develop
```

### Acceptance criteria

```txt
[ ] User can inspect one photo
[ ] User can move through photos quickly
[ ] User can rate/reject from Loupe
[ ] Filmstrip can be hidden
[ ] Viewer background is neutral
```

---

## 8. S004 — Develop

### Purpose

Main RAW editing screen.

### User goals

- Adjust exposure/color/detail/crop/masks
- View histogram
- Compare before/after
- Use presets
- Export final photo

### Wireframe

```txt
┌──────────────────────────────────────────────────────────────────┐
│ Sidebar | Library Develop Export | Before/After | Export          │
├───────────────┬────────────────────────────────┬─────────────────┤
│ Presets       │                                │ Histogram       │
│ - Favorites   │                                │ Basic           │
│ - Film        │          Edited Photo           │  Exposure       │
│ - Portrait    │                                │  Contrast       │
│ - Food        │                                │  Highlights     │
│               │                                │ Tone            │
│ History       │                                │ Color           │
│ Snapshots     │                                │ Detail          │
│               │                                │ Lens            │
│               │                                │ Geometry        │
│               │                                │ Mask            │
├───────────────┴────────────────────────────────┴─────────────────┤
│ [thumb][thumb][thumb][selected][thumb][thumb]                    │
└──────────────────────────────────────────────────────────────────┘
```

### Components

```txt
SrToolbar
SrPresetSidebar
SrLoupeViewer
SrInspector
SrHistogram
SrInspectorSection
SrAdjustmentSlider
SrToneCurve
SrColorMixer
SrMaskPanel
SrFilmstrip
SrBeforeAfterControl
SrPresetCard
```

### Inspector sections

```txt
Histogram
Basic
Tone
Color
Detail
Lens
Geometry
Mask
Metadata
Export
```

### Basic section controls

```txt
Profile
White Balance
Temperature
Tint
Exposure
Contrast
Highlights
Shadows
Whites
Blacks
Texture
Clarity
Dehaze
Vibrance
Saturation
```

### Required behavior

```txt
Slider drag = immediate Metal preview update
Double click slider = reset
Option drag = fine adjust
Shift drag = coarse adjust
B = before/after
R = crop
M = mask
Cmd+C / Cmd+V = copy/paste edits, when applicable
Cmd+E = export
```

### Acceptance criteria

```txt
[ ] User can edit RAW non-destructively
[ ] Histogram updates with current preview
[ ] Slider changes are responsive
[ ] User can apply/reset sections
[ ] User can view before/after
[ ] User can export edited photo
[ ] AI tools appear only inside relevant edit sections
```

---

## 9. S005 — Export Dialog / Export Panel

### Purpose

Export edited photos in practical formats.

### Wireframe

```txt
┌──────────────────────────────────────────┐
│ Export 24 Photos                         │
├──────────────────────────────────────────┤
│ Preset                                   │
│ [Full Quality JPEG ▾]                    │
│                                          │
│ Destination                              │
│ [Choose Folder...]                       │
│                                          │
│ Format                                   │
│ JPEG / PNG / TIFF / HEIC / WebP          │
│                                          │
│ Quality        [────●────] 90            │
│ Resize         [ ] Resize long edge      │
│ Color Space    sRGB / Display P3 / ...   │
│ Metadata       Preserve / Remove GPS     │
│ Naming         Original / Custom         │
│                                          │
│              [Cancel] [Export]           │
└──────────────────────────────────────────┘
```

### Components

```txt
SrExportDialog
SrExportPresetCard
SrButton
SrSegmentedControl
SrAdjustmentSlider
SrSelect
SrCheckbox
```

### Export presets

```txt
Full Quality JPEG
Web JPEG
Instagram
Google Maps
Blog
TIFF Archive
Custom
```

### Acceptance criteria

```txt
[ ] User can export one photo
[ ] User can batch export many photos
[ ] User can select format and quality
[ ] User can choose color space
[ ] User can preserve/remove metadata
[ ] Export progress is visible
[ ] Export can run without blocking browsing
```

---

## 10. S006 — Preferences

### Purpose

App-wide settings.

### Sections

```txt
General
Appearance
Editing
Performance
Storage
Export
Shortcuts
Advanced
```

### Wireframe

```txt
┌──────────────────────────────────────────────┐
│ Preferences                                  │
├───────────────┬──────────────────────────────┤
│ General       │ Appearance                   │
│ Appearance    │ Theme: System / Dark / Light │
│ Editing       │ Accent: System / Silica      │
│ Performance   │                              │
│ Storage       │ Cache Size                   │
│ Export        │                              │
│ Shortcuts     │                              │
│ Advanced      │                              │
└───────────────┴──────────────────────────────┘
```

### Components

```txt
SrSettingsSidebar
SrSettingsSection
SrSegmentedControl
SrSelect
SrCheckbox
SrButton
```

### Acceptance criteria

```txt
[ ] User can change theme
[ ] User can choose system or Silica accent
[ ] User can manage cache
[ ] User can see storage location
[ ] Advanced MCP is hidden behind Advanced
```

---

## 11. S007 — Import Progress

### Purpose

Show progress while scanning and generating thumbnails.

### Wireframe

```txt
┌─────────────────────────────────────────────┐
│ Importing Photos                            │
│                                             │
│ Scanning files        1,240 / 3,280          │
│ Generating previews   420 / 3,280            │
│ Extracting metadata   890 / 3,280            │
│                                             │
│ [Pause] [View Errors]                        │
└─────────────────────────────────────────────┘
```

### Components

```txt
SrImportProgress
SrProgressBar
SrButton
SrToast
```

### Rules

- Import must not block already imported browsing.
- Errors are collected into a reviewable list.
- Import progress can be minimized.

---

## 12. S008 — Compare View

### Purpose

Compare similar shots for culling.

### Wireframe

```txt
┌──────────────────────────────────────────────────────────────┐
│ Compare | Sync Zoom [✓] | Pick | Reject                       │
├──────────────────────────────┬───────────────────────────────┤
│ Photo A                      │ Photo B                       │
│                              │                               │
│ ★★★★☆                        │ ★★★★★                         │
└──────────────────────────────┴───────────────────────────────┘
```

### Modes

```txt
2-up
Survey
Candidate/Select
```

### Acceptance criteria

```txt
[ ] User can compare 2+ photos
[ ] User can sync zoom/pan
[ ] User can rate/reject without leaving compare view
[ ] User can mark selected photo as pick
```

---

## 13. S009 — Before / After View

### Purpose

Compare original and edited versions.

### Modes

```txt
Toggle
Split vertical
Split horizontal
Side by side
```

### Wireframe

```txt
┌──────────────────────────────────────────────┐
│ Before / After                               │
├──────────────────────┬───────────────────────┤
│ Before               │ After                 │
│                      │                       │
└──────────────────────┴───────────────────────┘
```

### Acceptance criteria

```txt
[ ] B toggles before/after
[ ] Split position is draggable
[ ] Viewer remains responsive
[ ] Overlay labels are subtle
```

---

## 14. S010 — Preset Manager

### Purpose

Create, edit, organize, import, and export presets.

### Sections

```txt
Built-in
User Presets
Imported Packs
Favorites
```

### Acceptance criteria

```txt
[ ] User can create preset from current edit
[ ] User can rename/delete user presets
[ ] User can favorite presets
[ ] User can import/export preset packs
```

---

## 15. S011 — Collection Manager

### Purpose

Manage manual and smart collections.

### Collection types

```txt
Manual Collection
Smart Collection
Recent Import
Favorites
Rejected
```

### Smart collection filters

```txt
Rating
File type
Camera
Lens
Date
Flag
Edited status
Exported status
```

---

## 16. S012 — Batch Edit / Sync Edits

### Purpose

Apply settings from one photo to many photos.

### Wireframe

```txt
┌──────────────────────────────────────────┐
│ Sync Edits to 24 Photos                  │
├──────────────────────────────────────────┤
│ [✓] Basic                                │
│ [✓] Tone Curve                           │
│ [✓] Color Mixer                          │
│ [ ] Crop                                 │
│ [ ] Masks                                │
│ [✓] Detail                               │
│                                          │
│             [Cancel] [Sync]              │
└──────────────────────────────────────────┘
```

### Acceptance criteria

```txt
[ ] User can copy edits
[ ] User can paste edits
[ ] User can choose which edit groups to sync
[ ] Batch action is undoable
```

---

## 17. S013 — AI Review

### Purpose

Review AI suggestions without letting AI take control.

### Use cases

```txt
Blur review
Duplicate grouping
Best shot ranking
Subject mask preview
Sky mask preview
Auto tone suggestion
```

### Wireframe

```txt
┌──────────────────────────────────────────────────────────────┐
│ AI Review: 18 blurry photos found                            │
├──────────────────────────────────────────────────────────────┤
│ [thumb] Reason: Motion blur likely       [Reject] [Keep]     │
│ [thumb] Reason: Focus missed             [Reject] [Keep]     │
│ [thumb] Reason: Duplicate lower score    [Reject] [Keep]     │
│                                                              │
│                         [Apply Decisions] [Cancel]           │
└──────────────────────────────────────────────────────────────┘
```

### Rules

- AI cannot reject/export/delete without user approval.
- Every suggestion must be reviewable.
- Quality score must provide basic explanation.

---

## 18. S014 — Mask Editor

### Purpose

Detailed mask editing and management.

### Components

```txt
SrMaskPanel
SrMaskOverlay
SrAdjustmentSlider
SrButton
SrPopover
```

### Rules

- MLX-generated masks appear beside manual masks.
- User can rename, invert, disable, delete masks.
- Mask overlay must not permanently obscure the photo.

---

## 19. S015 — Plugin Manager

### Purpose

Manage local plugins, preset packs, export extensions, and AI model plugins.

### Priority

P3, not v1.

### Rules

- Plugins are disabled by default until explicitly enabled.
- Plugin permissions must be visible.
- External plugin marketplace is future only.

---

## 20. S016 — MCP Agent Access

### Purpose

Allow advanced users to expose SilicaRAW actions to agent tools.

### Location

```txt
Preferences
→ Advanced
→ Agent Access
```

### Wireframe

```txt
┌──────────────────────────────────────────┐
│ Agent Access                             │
├──────────────────────────────────────────┤
│ Enable MCP Server           [ Off ]      │
│                                          │
│ Permission Level                         │
│ ( ) Read-only                            │
│ ( ) Edit metadata                        │
│ ( ) Apply edits                          │
│ ( ) Export                               │
│                                          │
│ [View Action Log]                        │
└──────────────────────────────────────────┘
```

### Rules

- Default OFF.
- No delete command in v1.
- Export requires explicit permission.
- Action log required.

---

## 21. S017 — Action Log

### Purpose

Show actions performed by AI, plugins, or MCP agents.

### Example rows

```txt
12:31  AI Blur Review suggested 18 rejects
12:35  User applied 12 AI suggestions
12:41  MCP exported 10 photos to /Exports
```

### Acceptance criteria

```txt
[ ] User can review automated actions
[ ] User can filter by AI / MCP / Plugin
[ ] Sensitive file paths can be hidden in privacy mode
```

---

## 22. S018 — Model Manager

### Purpose

Manage MLX models.

### Priority

P3.

### Model categories

```txt
Denoise
Upscale
Subject Mask
Sky Mask
Quality Score
Auto Tone
```

### Rules

- Model downloads require user approval.
- Model version must be visible.
- Offline mode must remain possible.

---

## 23. S019 — About / Credits

### Purpose

Show version, license, contributors, dependencies.

### Must include

```txt
Version
Commit hash
License
Contributors
Third-party licenses
Website/GitHub
```

---

## 24. Cross-Screen Flows

## 24.1 First launch flow

```txt
Welcome
→ Open Folder
→ Import Progress
→ Library Grid
→ Develop
→ Export
```

## 24.2 Culling flow

```txt
Library Grid
→ Rate / Reject
→ Compare View
→ Filter Picks
→ Export
```

## 24.3 Editing flow

```txt
Library Grid
→ Develop
→ Basic Adjustments
→ Color / Detail
→ Before / After
→ Export
```

## 24.4 AI-assisted flow

```txt
Library Grid
→ Run AI Culling
→ AI Review
→ Apply Decisions
→ Develop Selected Photos
→ Export
```

## 24.5 Agent flow

```txt
Preferences
→ Advanced
→ Enable MCP
→ Permission Prompt
→ Agent Action
→ Action Log
```

---

## 25. Implementation Order

### Phase 0 — App frame

```txt
SrToolbar
SrSidebar
SrInspector
Main content region
Panel resizing
Dark/light tokens
```

### Phase 1 — Welcome + Library Grid

```txt
Welcome screen
Open folder action
Import progress placeholder
Virtualized thumbnail grid
Sidebar navigation
Basic inspector metadata
```

### Phase 2 — Loupe + Culling

```txt
Loupe viewer
Filmstrip
Rating/reject shortcuts
Search/filter
```

### Phase 3 — Develop MVP

```txt
Develop layout
Histogram placeholder
Basic adjustment sliders
Before/after placeholder
Edit history placeholder
```

### Phase 4 — Export

```txt
Export dialog
Export presets
Progress feedback
```

### Phase 5 — Advanced screens

```txt
Compare
Preset manager
Batch sync edits
AI review
MCP settings
Action log
```

---

## 26. Screen QA Checklist

Each screen must satisfy:

```txt
[ ] Uses global app frame where applicable
[ ] Uses approved components
[ ] Uses design tokens only
[ ] Photo content remains visually dominant
[ ] Keyboard shortcuts work
[ ] Empty/loading/error states exist
[ ] Responsive enough for laptop screen sizes
[ ] Does not expose AI/MCP as primary unless screen is specifically advanced
[ ] Works in dark mode
[ ] Does not break in light mode
```

---

## 27. Agent Review

### Product Designer Agent

Status: GO WITH CONDITIONS

Notes:

- Screen inventory is complete enough for v1 planning.
- Main app frame is consistent.
- Need high-fidelity mockups after wireframes.

Conditions:

- Make responsive behavior explicit for 13-inch MacBook screens.
- Create actual visual mockups for Welcome, Library, Develop, Export.

### Photographer Workflow Agent

Status: GO

Notes:

- Library → Develop → Export flow is correct.
- Compare, Before/After, Batch Sync are appropriately prioritized.
- AI is useful but not disruptive.

### macOS UX Agent

Status: GO WITH CONDITIONS

Conditions:

- Toolbar must remain quiet.
- Preferences should feel like a macOS settings window.
- Keyboard and trackpad gestures must be tested on real Mac hardware.

### Frontend Engineer Agent

Status: GO WITH CONDITIONS

Conditions:

- Virtualized grid and filmstrip must be validated early.
- Panel resizing and persistence should be implemented in app frame first.
- Component gallery should precede full screen implementation.

### OSS Maintainer Agent

Status: GO

Notes:

- Screen IDs and acceptance criteria make GitHub issues easy to create.
- Advanced screens are correctly deferred.

---

## 28. Final Decision

Screen Inventory & Wireframe Specification v1.0: GO WITH CONDITIONS

Blocking conditions before UI implementation:

1. Build component gallery.
2. Implement global app frame first.
3. Validate 13-inch, 14-inch, 16-inch MacBook layouts.
4. Validate virtualized grid performance.
5. Create high-fidelity mockups for:
   - Welcome
   - Library Grid
   - Develop
   - Export Dialog

Next document:

07 — RAW Editing Feature Specification
