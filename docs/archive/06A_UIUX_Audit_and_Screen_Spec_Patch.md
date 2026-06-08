# 06A — UI/UX Audit & Screen Specification Patch

Version: v1.1  
Status: REQUIRED PATCH BEFORE IMPLEMENTATION  
Reviewed document: 06 Screen Inventory & Wireframe Specification v1.0

---

## 1. Executive Verdict

The current `06 Screen Inventory & Wireframe Specification` is a good foundation, but it is not yet sufficient for implementation.

Current readiness:

```txt
Screen inventory completeness:      85 / 100
Workflow clarity:                   82 / 100
Apple-like UX consideration:         78 / 100
Implementation readiness:            68 / 100
Responsive layout readiness:         55 / 100
Accessibility readiness:             58 / 100
Photo editor usability readiness:    76 / 100
```

Final verdict:

```txt
GO AS FOUNDATION
NO-GO FOR DIRECT IMPLEMENTATION
```

The document is good enough to proceed to the next planning layer, but not good enough to hand directly to Codex/Claude Code as UI implementation instructions.

It needs this patch.

---

## 2. What Was Good

### 2.1 Correct primary workflow

The document correctly centers the product around:

```txt
Library
Develop
Export
```

This protects the product from becoming an AI/MCP app instead of a photo editor.

### 2.2 Correct global layout

The document correctly defines:

```txt
Top    = toolbar
Left   = navigation/presets
Center = photo
Right  = inspector
Bottom = filmstrip/status
```

This matches the Apple Pro App mental model.

### 2.3 Correct AI/MCP restraint

The document correctly keeps:

```txt
AI = inside editing workflows
MCP = advanced settings
```

This is the right UX decision.

### 2.4 Good screen IDs

Screen IDs S001–S019 make it easy to create GitHub issues and implementation milestones.

---

## 3. Critical Gaps

## Gap 1 — Responsive Layout Is Not Specific Enough

The document says to validate 13-inch/14-inch/16-inch MacBook layouts, but does not define how layouts should adapt.

This is not enough.

### Required patch

Define responsive layout tiers.

```txt
Compact Desktop
- Width: 1280–1439
- Target: 13-inch MacBook
- Left sidebar collapsible by default
- Right inspector remains visible in Develop
- Filmstrip can auto-hide
- Toolbar uses compact icons

Standard Desktop
- Width: 1440–1719
- Target: 14-inch MacBook / external small display
- Left sidebar visible
- Right inspector visible
- Filmstrip bottom visible

Large Desktop
- Width: 1720+
- Target: 16-inch MacBook / external monitor
- Left sidebar visible
- Right inspector expanded
- Filmstrip bottom or left optional
- Compare/survey layouts become more useful
```

### New acceptance criteria

```txt
[ ] 1280px width does not require horizontal scrolling
[ ] Develop screen remains usable at 13-inch MacBook width
[ ] Inspector can collapse but must be quickly restorable
[ ] Filmstrip can hide automatically in compact width
[ ] Toolbar actions collapse into overflow menu when needed
```

---

## Gap 2 — Develop Screen Needs More Detail

Develop is the most important screen. The current wireframe is structurally correct, but not detailed enough.

It needs actual editing density rules.

### Required patch

Develop screen must define:

```txt
Histogram area height
Inspector section order
Default open/closed sections
Slider row height
Value input behavior
Reset behavior
Mask overlay behavior
Filmstrip collapse behavior
Before/after control placement
```

### Develop v1 layout rule

```txt
Right Inspector:
1. Histogram — always visible at top
2. Basic — open by default
3. Tone — collapsed
4. Color — collapsed
5. Detail — collapsed
6. Lens — collapsed
7. Geometry — collapsed
8. Mask — collapsed unless active
9. Metadata — collapsed
10. Export — compact action at bottom or toolbar
```

### Slider density

```txt
Slider row height: 44px
Label/value row: 18px
Track area: 18px
Gap between sliders: 8px
Section internal padding: 16px
```

### New acceptance criteria

```txt
[ ] Basic adjustments fit without excessive scrolling on 14-inch MacBook
[ ] Slider values are directly editable
[ ] Double-click reset works on every adjustment
[ ] Section reset is available
[ ] Histogram remains visible when Basic section is open
[ ] Before/after control does not obscure photo
```

---

## Gap 3 — UI States Are Too Generic

The document lists screens, but not enough per-screen states.

Every screen needs:

```txt
Empty
Loading
Partial loading
Error
Permission denied
Missing file
Offline / local-only
Long-running task
```

### Required state matrix

| Screen | Empty | Loading | Error | Partial | Missing |
|---|---|---|---|---|---|
| Welcome | Yes | No | Yes | No | No |
| Library Grid | Yes | Yes | Yes | Yes | Yes |
| Library Loupe | Yes | Yes | Yes | Yes | Yes |
| Develop | Yes | Yes | Yes | Yes | Yes |
| Export | No | Yes | Yes | Yes | Yes |
| Import Progress | No | Yes | Yes | Yes | No |
| AI Review | Yes | Yes | Yes | Yes | Yes |
| MCP Settings | Yes | No | Yes | No | No |

### New acceptance criteria

```txt
[ ] Missing file state exists
[ ] Corrupt RAW state exists
[ ] Unsupported RAW format state exists
[ ] Export failure state exists
[ ] Permission denied state exists
[ ] AI model unavailable state exists
```

---

## Gap 4 — Accessibility Needs Screen-Level Rules

The current document mentions shortcuts, but accessibility is not screen-specific.

### Required patch

Each screen must define:

```txt
Focus order
Keyboard-only path
Screen reader labels
Reduced motion behavior
High contrast behavior
Minimum target size
```

### Example: Library Grid focus order

```txt
Toolbar
→ Sidebar
→ Filter bar
→ Thumbnail grid
→ Right inspector
→ Filmstrip/status, if visible
```

### Example: Develop focus order

```txt
Toolbar
→ Preset sidebar
→ Main viewer
→ Right inspector histogram
→ Basic section controls
→ Filmstrip
```

### New acceptance criteria

```txt
[ ] User can rate/reject photos without mouse
[ ] User can edit a slider using keyboard
[ ] User can export using keyboard
[ ] Icon-only buttons have labels/tooltips
[ ] Reduced motion disables non-essential transitions
[ ] Focus ring is always visible
```

---

## Gap 5 — Toolbar Overflow Is Missing

A Mac-like toolbar must not become crowded.

The current toolbar says:

```txt
Sidebar | Library Develop Export | Search | Export
```

But at smaller widths, this can break.

### Required patch

Toolbar must support overflow.

```txt
Left group:
- Sidebar toggle
- Current source / folder

Center group:
- Mode switcher

Right group:
- Search
- View options
- Export
- More menu
```

At compact width:

```txt
Search collapses to icon
View options move into More
Export remains visible when selection exists
```

### New acceptance criteria

```txt
[ ] Toolbar does not overflow at 1280px
[ ] Search can collapse
[ ] Secondary actions move into More menu
[ ] Export remains reachable
```

---

## Gap 6 — Panel Persistence Is Missing

Professional apps remember layout.

### Required patch

Persist:

```txt
Sidebar width
Inspector width
Filmstrip visibility
Filmstrip position
Collapsed inspector sections
Last mode
Last library
Grid thumbnail size
Sort/filter state
```

### New acceptance criteria

```txt
[ ] User layout preferences restore after restart
[ ] Reset layout command exists
[ ] Compact-mode automatic changes do not permanently destroy user settings
```

---

## Gap 7 — Gesture and Trackpad UX Is Too Thin

For a Mac-first editor, gestures are not optional.

### Required patch

Viewer gestures:

```txt
Pinch = zoom
Two-finger pan = pan image
Double click = 100% / fit toggle
Space + drag = pan
Two-finger horizontal scroll = filmstrip navigation
Option-scroll = fine zoom
```

### New acceptance criteria

```txt
[ ] Trackpad zoom feels native
[ ] Pan inertia does not fight image editing
[ ] Filmstrip scrolling is smooth
[ ] Gestures do not accidentally adjust sliders
```

---

## Gap 8 — Color Management UI Is Only Mentioned

Export has color space selection, but Develop/Preview color behavior is not specified.

This is risky for a photo editor.

### Required patch

Add visible but restrained color indicators:

```txt
Viewer:
- Soft proof toggle, future
- Display profile status, hidden under metadata/info

Export:
- Color Space: sRGB, Display P3, Adobe RGB, ProPhoto RGB
- Embed ICC profile toggle
- Remove GPS metadata toggle
```

### New acceptance criteria

```txt
[ ] Export color space is explicit
[ ] ICC profile embedding is explicit
[ ] Default export is sRGB for web compatibility
[ ] Display P3 export is available
```

---

## Gap 9 — Import Flow Needs More UX

Import progress is defined, but not the pre-import decision.

### Required patch

Open Folder should ask:

```txt
Add folder to catalog
or
Open temporarily
```

Optional future:

```txt
Copy files into library
Leave files in place
```

For v1, choose:

```txt
Leave files in place
```

### New acceptance criteria

```txt
[ ] User understands original files are not moved
[ ] User understands edits are non-destructive
[ ] Import can be cancelled
[ ] Import errors can be reviewed
```

---

## Gap 10 — High-Fidelity Mockup Requirement Is Correct But Incomplete

The document says high-fidelity mockups are needed for:

```txt
Welcome
Library
Develop
Export
```

This is correct, but insufficient.

### Required high-fidelity mockup list

```txt
M001 Welcome
M002 Library Grid, empty
M003 Library Grid, populated
M004 Library Loupe
M005 Develop, default
M006 Develop, mask active
M007 Export Dialog
M008 Preferences > Appearance
M009 Import Progress
M010 AI Review
```

### Required responsive mockup set

For each of M003, M005, M007:

```txt
1280 width
1440 width
1728 width
```

---

## 4. Patched GO Criteria

The 06 document can be considered implementation-ready only after these patches are integrated.

### Required additions

```txt
[ ] Responsive layout tiers
[ ] Per-screen state matrix
[ ] Focus order per primary screen
[ ] Toolbar overflow behavior
[ ] Panel persistence behavior
[ ] Develop screen density rules
[ ] Trackpad gesture rules
[ ] Import decision flow
[ ] Color management UI rules
[ ] High-fidelity mockup list
```

---

## 5. Agent Re-Review

### Product Designer Agent

Verdict: NO-GO FOR IMPLEMENTATION

Reason:

The screen inventory is strong, but lacks responsive behavior, per-screen states, and high-fidelity mockup requirements.

Required patch:

- Add responsive tiers
- Add state matrix
- Add mockup requirements

### Apple HIG Agent

Verdict: GO WITH CONDITIONS

Reason:

The structure follows toolbar/sidebar/inspector conventions, but toolbar overflow, keyboard/focus behavior, and window resizing need stronger rules.

Required patch:

- Toolbar overflow
- Focus order
- Reduced motion
- Native-feeling gestures

### Photographer Workflow Agent

Verdict: GO WITH CONDITIONS

Reason:

Core workflow is correct, but Develop screen needs more precise editing density and culling details.

Required patch:

- Develop density rules
- Compare/culling keyboard behavior
- Before/after placement
- Export defaults

### Frontend Engineering Agent

Verdict: NO-GO FOR IMPLEMENTATION

Reason:

The document does not yet specify responsive breakpoints, virtualization thresholds, state handling, or layout persistence.

Required patch:

- Responsive tiers
- Virtualization acceptance
- Panel persistence
- Component gallery before screen implementation

### OSS Maintainer Agent

Verdict: GO WITH CONDITIONS

Reason:

Screen IDs and acceptance criteria are useful for issues, but PR review criteria need more measurable UI requirements.

Required patch:

- Add screen QA matrix
- Add implementation checklist per screen

---

## 6. Final Verdict

```txt
06 Screen Inventory v1.0:
GOOD FOUNDATION
NOT SUFFICIENT FOR DIRECT UI IMPLEMENTATION

06 + This Patch:
GO WITH CONDITIONS
```

The product direction is correct. The UI/UX thinking is mostly correct. But without this patch, implementation would likely drift and become inconsistent.

---

## 7. Required Next Action

Before proceeding to high-fidelity UI implementation, create:

```txt
06.1 Responsive Layout Specification
06.2 Primary Screen State Matrix
06.3 Focus & Keyboard Navigation Specification
06.4 High-Fidelity Mockup Brief
```

These can be merged into a single `06_v1_1` revised screen specification.
