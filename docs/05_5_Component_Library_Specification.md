# 05.5 — SilicaRAW Component Library Specification

Version: v1.0  
Status: GO WITH CONDITIONS  
Depends on: 05 Design System Specification  
Goal: Define reusable UI components before screen-level wireframes.

---

## 1. Purpose

The component library ensures that SilicaRAW remains visually consistent, maintainable, and Apple-like as features grow.

Every screen must be built from these components unless a new component is reviewed and added to this document.

This document is intentionally strict. It exists to prevent visual drift.

---

## 2. Component Architecture

### 2.1 Layers

```txt
Foundation
├─ Tokens
├─ Typography
├─ Color
├─ Spacing
├─ Motion
└─ Icons

Primitive Components
├─ Button
├─ IconButton
├─ TextField
├─ Slider
├─ Select
├─ Checkbox
├─ SegmentedControl
└─ Popover

Editor Components
├─ Toolbar
├─ Sidebar
├─ Inspector
├─ Histogram
├─ ThumbnailCell
├─ Filmstrip
├─ RatingControl
├─ MetadataRow
├─ PresetCard
├─ AdjustmentSlider
├─ MaskRow
├─ ExportPresetCard
└─ BeforeAfterControl

Workflow Components
├─ ImportProgress
├─ ExportProgress
├─ EmptyState
├─ AIReviewBanner
├─ PermissionPrompt
├─ ActionLogRow
└─ Toast
```

### 2.2 Naming convention

Use `Sr` prefix for reusable components.

Examples:

```txt
SrButton
SrToolbar
SrSidebar
SrInspector
SrAdjustmentSlider
SrHistogram
SrThumbnailCell
```

---

## 3. Global Component States

Every interactive component must define:

```txt
Default
Hover
Pressed
Focused
Selected
Disabled
Loading
Error, if applicable
```

No component may ship with only default state.

---

## 4. Primitive Components

## 4.1 SrButton

### Purpose

Primary actions, secondary actions, destructive actions, and low-emphasis actions.

### Variants

Only four variants:

```txt
Primary
Secondary
Ghost
Destructive
```

### Usage

Primary:

- Import
- Export
- Apply
- Open Folder

Secondary:

- Cancel
- Review
- Reset
- Save Preset

Ghost:

- Toolbar utility actions
- Toggle panels
- Low-emphasis controls

Destructive:

- Remove from catalog
- Clear cache
- Revoke agent access

### Anatomy

```txt
[Icon?] Label [Shortcut?]
```

### Size

```txt
Small: 28px
Medium: 32px
Large: 40px
```

### Rules

- Only one primary button per dialog.
- Destructive buttons must not be visually close to primary confirmation.
- Ghost buttons must not be used for critical actions.
- Icon-only buttons require tooltip.

---

## 4.2 SrIconButton

### Purpose

Compact toolbar and viewer actions.

### Anatomy

```txt
Icon
Tooltip
Optional selected state
```

### Sizes

```txt
Toolbar: 32px
Compact: 28px
Viewer overlay: 36px
```

### Required behavior

- Tooltip on hover
- Keyboard focus ring
- Selected state for toggles
- Disabled state with reduced opacity

---

## 4.3 SrSegmentedControl

### Purpose

Mode switching or small mutually exclusive options.

### Examples

```txt
Library / Develop / Export
Grid / Loupe / Compare
Before / After / Split
```

### Rules

- Maximum 5 segments.
- Do not use for unrelated actions.
- Selected segment uses system accent or active surface, not heavy fill.

---

## 4.4 SrTextField / SearchField

### Purpose

Search, naming collections, export filenames.

### Variants

```txt
Standard
Search
Compact
```

### Rules

- Search field should support clear button.
- Use placeholder text sparingly.
- Keyboard shortcut Cmd+F focuses search in Library.

---

## 4.5 SrSlider

### Purpose

Generic slider primitive.

For photo adjustments, use `SrAdjustmentSlider`.

### Behavior

```txt
Drag = normal adjustment
Option-drag = fine adjustment
Shift-drag = coarse adjustment
Double-click = reset
Enter on value = manual numeric input
```

---

## 5. App Frame Components

## 5.1 SrToolbar

### Purpose

Top-level command and mode area.

SilicaRAW’s toolbar should remain quiet and functional.

### Layout

```txt
Left:
- Sidebar toggle
- Library selector / current folder

Center:
- Mode segmented control
  Library | Develop | Export

Right:
- Search
- View options
- Export
- Settings
```

### Height

```txt
48px
```

### Rules

- Do not overload the toolbar.
- AI and MCP do not appear as top-level toolbar items.
- Export may appear as a primary action in Develop and Library.
- Search appears in Library; hidden or secondary in Develop.

---

## 5.2 SrSidebar

### Purpose

Navigation between folders, collections, and smart collections.

### Sections

```txt
Library
- All Photos
- Recent Imports
- Favorites
- Rejected

Folders
- User-added folders

Collections
- Manual collections
- Smart collections
```

### Item anatomy

```txt
Icon
Label
Count
Optional disclosure arrow
```

### Width

```txt
Default: 260px
Min: 220px
Max: 360px
```

### Rules

- Sidebar controls navigation only.
- Editing controls never appear in the sidebar.
- Counts use tertiary text.
- Active item uses subtle selected surface and accent indicator.

---

## 5.3 SrInspector

### Purpose

Right-side context-sensitive controls.

SilicaRAW uses the inspector for edit controls, metadata, export options, and mask details.

### Width

```txt
Default: 320px
Min: 280px
Max: 420px
```

### Modes

```txt
Library Inspector
Develop Inspector
Export Inspector
```

### Develop sections

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

### Rules

- Histogram remains at top in Develop.
- Basic section is open by default.
- Advanced sections are collapsible.
- The inspector must remain scrollable independent of the viewer.
- AI tools appear inside relevant sections, not as a separate AI inspector.

---

## 6. Photo Browsing Components

## 6.1 SrThumbnailGrid

### Purpose

High-performance photo browsing.

### Requirements

- Virtualized rendering is mandatory.
- Supports 1,000–50,000 photos.
- Supports keyboard navigation.
- Supports multi-select.
- Supports filtering and sorting.

### Layout

```txt
Adaptive grid
Thumbnail aspect fit
Metadata/rating overlay optional
```

### Rules

- Do not render all thumbnails at once.
- Selection must be visually clear but not loud.
- Reject state must be visible but subtle.
- Rating badges must not obscure the image.

---

## 6.2 SrThumbnailCell

### Anatomy

```txt
Thumbnail image
Selection ring
Rating row
Reject/Pick badge
File type badge
Optional AI score
```

### States

```txt
Default
Hover
Selected
Focused
Rejected
Missing file
Loading
```

### Visual rules

- Selected: subtle accent outline + surface highlight.
- Rejected: dimmed opacity + small X badge.
- Missing: placeholder icon + warning badge.
- AI score: hidden by default; shown in AI review mode.

---

## 6.3 SrFilmstrip

### Purpose

Compact sequential navigation in Develop.

### Position options

```txt
Bottom
Left
Hidden
```

Default:

```txt
Bottom
```

### Requirements

- Virtualized thumbnails.
- Keyboard sync with main viewer.
- Drag to resize.
- User preference persists.

### Rules

- Filmstrip should not be mandatory.
- Users with small screens must be able to hide it.
- Active photo is highlighted with subtle accent border.

---

## 6.4 SrLoupeViewer

### Purpose

Main photo viewing surface.

### Behaviors

```txt
Pinch = zoom
Two-finger pan = pan
Double click = 100%
Space drag = pan
F = fullscreen
B = before/after
```

### Rules

- Background must be neutral dark.
- Zoom controls appear only when relevant.
- Viewer overlays must fade or stay low-emphasis.
- The image must never be covered by persistent decorative UI.

---

## 6.5 SrCompareView

### Purpose

Compare multiple similar photos.

### Modes

```txt
2-up
Survey
Candidate/Select
```

### Use cases

- Culling
- Duplicate review
- Best shot selection

### Rules

- Ratings and reject controls must be available.
- Zoom/pan sync option required.
- Compare mode should be keyboard-first.

---

## 7. Editing Components

## 7.1 SrHistogram

### Purpose

Luminance and RGB histogram display.

### Placement

Top of Develop inspector.

### Variants

```txt
Compact
Expanded
RGB Overlay
Luminance
```

### Rules

- Must be readable but not visually loud.
- Clipping indicators appear as small toggles.
- Histogram is not decorative; it must reflect current edit preview.

---

## 7.2 SrInspectorSection

### Purpose

Collapsible control grouping.

### Anatomy

```txt
Disclosure chevron
Section title
Optional reset button
Optional enable toggle
Content
```

### Rules

- Basic open by default.
- Tone, Color, Detail, Lens, Geometry, Mask closed by default unless active.
- Reset section action must be visible on hover or section focus.
- Section state persists per user.

---

## 7.3 SrAdjustmentSlider

### Purpose

Photo-editing adjustment with label, value, range, reset, and precision behavior.

### Anatomy

```txt
Label                           Value
[-------------●----------------------]
```

### Required features

- Numeric value display
- Manual numeric input
- Double click reset
- Option-drag fine control
- Shift-drag coarse control
- Keyboard increment/decrement
- Reset to default

### Example controls

```txt
Exposure
Contrast
Highlights
Shadows
Whites
Blacks
Temperature
Tint
Vibrance
Saturation
Sharpening
Noise Reduction
Vignette
```

### Rules

- All adjustment sliders must look and behave identically.
- Values must use tabular figures.
- Zero/default position should be visually understandable.
- Slider must update Metal preview immediately.

---

## 7.4 SrToneCurve

### Purpose

Professional tone curve editing.

### Modes

```txt
Parametric
Point Curve
RGB Channels
```

### Requirements

- Add/remove points
- Reset curve
- Channel selection
- Smooth interaction
- Keyboard nudging for selected point

### Rules

- Curve graph must be larger than a normal slider.
- Avoid cramped curve UI in narrow inspector; allow expanded popover.

---

## 7.5 SrColorMixer

### Purpose

HSL/color editing.

### Modes

```txt
Hue
Saturation
Luminance
```

Rows:

```txt
Red
Orange
Yellow
Green
Aqua
Blue
Purple
Magenta
```

Each row uses `SrAdjustmentSlider`.

---

## 7.6 SrMaskPanel

### Purpose

Manual and MLX-generated masks.

### Add mask menu

```txt
Brush
Linear Gradient
Radial Gradient
Subject
Sky
Background
Color Range
Luminance Range
```

### Rules

- Subject/Sky/Background can be MLX-powered but appear as normal mask options.
- Generated masks must be previewed before applying.
- Mask list shows name, visibility toggle, invert, delete.
- Mask overlay color must be user-adjustable eventually.

---

## 7.7 SrPresetCard

### Purpose

Preset selection and preview.

### Anatomy

```txt
Preview thumbnail
Preset name
Category
Favorite state
```

### Rules

- Presets should be previewable before applying.
- Built-in presets must be tasteful and restrained.
- No aggressive Instagram filter feeling by default.

---

## 8. Workflow Components

## 8.1 SrImportProgress

### Purpose

Communicate large folder import progress.

### Shows

```txt
Files scanned
Thumbnails generated
Metadata extracted
Errors
Pause/Resume
```

### Rules

- Import should not block browsing already imported images.
- Errors must be reviewable after import.

---

## 8.2 SrExportDialog

### Purpose

Batch and single-photo export.

### Sections

```txt
Destination
Format
Quality
Resize
Color Space
Metadata
Watermark
Naming
```

### Presets

```txt
Full Quality JPEG
Web JPEG
Instagram
Google Maps
Blog
TIFF Archive
Custom
```

### Rules

- Export presets must be editable.
- Color space selection must be explicit.
- Metadata remove/preserve option is required.
- Google Maps preset should be practical, not visually branded.

---

## 8.3 SrAIReviewBanner

### Purpose

Show AI suggestions without taking control.

### Example

```txt
SilicaRAW found 18 blurry photos.
[Review] [Ignore]
```

### Rules

- AI never applies destructive decisions automatically.
- Bulk actions require confirmation.
- “Why?” explanation should be available for quality scoring.

---

## 8.4 SrPermissionPrompt

### Purpose

MCP/plugin permission confirmation.

### Shows

```txt
Requesting tool
Requested action
Affected photos
Permission level
Allow once
Allow for session
Deny
```

### Rules

- No silent export through agent.
- No delete command in v1.
- Action log entry required for any agent-driven edit/export.

---

## 8.5 SrToast

### Purpose

Non-blocking feedback.

### Examples

```txt
Preset applied
Export complete
10 photos rejected
MCP access denied
```

### Rules

- Toasts should not stack endlessly.
- Critical errors use dialog, not toast.
- Toast duration: 3–5 seconds.

---

## 9. Screen Composition Guidelines

### 9.1 Library screen

Must use:

```txt
SrToolbar
SrSidebar
SrThumbnailGrid
SrInspector
SrImportProgress
SrSearchField
```

### 9.2 Develop screen

Must use:

```txt
SrToolbar
SrPresetSidebar
SrLoupeViewer
SrInspector
SrHistogram
SrAdjustmentSlider
SrFilmstrip
SrBeforeAfterControl
```

### 9.3 Export screen

Must use:

```txt
SrToolbar
SrExportDialog or SrExportPanel
SrExportPresetCard
SrProgress
```

### 9.4 Settings screen

Must use:

```txt
SrSidebar
SrSettingsSection
SrSegmentedControl
SrPermissionPrompt
SrActionLogRow
```

---

## 10. Keyboard and Interaction Standards

### Global shortcuts

```txt
G = Library / Grid
D = Develop
E = Export
1–5 = Rating
0 = Clear rating
X = Reject
P = Pick
F = Fullscreen
Space = Loupe / Pan modifier
B = Before/After
C = Compare
R = Crop
M = Mask
Cmd+E = Export
Cmd+Z = Undo
Cmd+Shift+Z = Redo
Cmd+F = Search
```

### Slider shortcuts

```txt
Arrow = small increment
Shift+Arrow = large increment
Option+drag = fine adjust
Double click = reset
```

---

## 11. Component QA Checklist

A component cannot be accepted unless:

```txt
[ ] Uses design tokens only
[ ] Has all required states
[ ] Has keyboard/focus behavior
[ ] Works in dark mode
[ ] Does not break in light mode
[ ] Has tooltip if icon-only
[ ] Has accessibility labels
[ ] Does not introduce a new color/radius/spacing
[ ] Performs acceptably with large data
[ ] Matches Apple Pro App feel
```

---

## 12. Agent Review

### Product Designer Agent

Status: GO

Notes:

- Component set is now broad enough to support screen wireframes.
- Accent and AI restraint are correctly defined.
- Slider and inspector behavior are appropriately strict.

### macOS / Apple HIG Agent

Status: GO WITH CONDITIONS

Conditions:

- Toolbar should remain configurable but not overloaded.
- Sidebar must be navigation-only.
- Inspector must be context-sensitive and predictable.

### Photographer Workflow Agent

Status: GO WITH CONDITIONS

Conditions:

- Compare view and filmstrip must be fast.
- Adjustment sliders must be responsive with immediate preview.
- Export presets need real-world usefulness.

### Frontend Engineer Agent

Status: GO WITH CONDITIONS

Conditions:

- Thumbnail grid and filmstrip virtualization are mandatory.
- Component library should be built in isolation before full screens.
- Token enforcement should be included in code review.

### OSS Maintainer Agent

Status: GO

Notes:

- Component rules are strict enough for contributor governance.
- New component proposal process should be added later.

---

## 13. Final Decision

Component Library Specification v1.0: GO WITH CONDITIONS

Blocking conditions before implementation:

1. Create token CSS files.
2. Implement primitive components first.
3. Implement Storybook-like component preview or internal component gallery.
4. Validate thumbnail grid virtualization.
5. Validate slider interaction performance with Metal preview loop.

Next document:

06 — Screen Inventory & Wireframe Specification
