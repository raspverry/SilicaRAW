# 05 — SilicaRAW Design System Specification

Version: v1.0  
Status: GO WITH CONDITIONS  
Product: SilicaRAW  
Principle: Apple Pro App feel, photo editor first, AI/MCP secondary.

---

## 1. Purpose

The SilicaRAW design system exists to prevent visual drift as the project grows.

SilicaRAW must not feel like a typical open-source utility. It should feel like a focused Apple Pro App: quiet, fast, consistent, and content-first.

The design system governs:

- Color
- Typography
- Spacing
- Radius
- Borders
- Materials
- Motion
- Icons
- Component anatomy
- Interaction states
- Accessibility
- Implementation tokens

All UI code must consume design tokens. Hard-coded visual values are prohibited except in token definitions.

---

## 2. Design Philosophy

### 2.1 Product identity

SilicaRAW is a professional RAW photo editor for macOS and Apple Silicon.

It is not:

- An AI playground
- A SaaS dashboard
- A Material Design app
- A Lightroom clone with different colors
- A Linux-style technical utility

It is:

- A focused RAW editor
- A macOS-first creative tool
- A Metal-first editing interface
- An MLX-enhanced intelligent editor
- A modular open-source Pro App

### 2.2 Guiding sentence

> SilicaRAW should feel like an Apple Pro App, not an open-source utility.

### 2.3 Visual attitude

The UI should disappear behind the photograph.

The photograph is the hero. Controls should be precise, quiet, and available, but not visually loud.

---

## 3. Reference Products

### 3.1 Primary references

- Apple Photos: content-first browsing, clean hierarchy, macOS-native feeling
- Final Cut Pro: professional three-region layout, browser/viewer/inspector model
- Pixelmator Pro: Mac-first creative editing, professional tools with approachable UX
- Lightroom: RAW development workflow and panel terminology
- Capture One: professional color/editing expectations
- RapidRAW: lightweight open-source RAW editor direction

### 3.2 What to avoid

- Darktable/RawTherapee visual complexity
- Material Design widgets
- Tailwind dashboard aesthetics
- Neon/glow cyber UI
- Excessive glassmorphism
- Heavy gradients
- Random icon packs
- Arbitrary per-screen styling

---

## 4. Design Principles

### Principle 1 — Photo Editor First

The first impression must be:

> “This is a serious photo editor.”

AI and MCP must not dominate the primary navigation.

### Principle 2 — Content First

Photos receive visual priority.

Controls use low-contrast surfaces, subtle borders, and restrained accent color.

### Principle 3 — Consistency Over Novelty

Every new component must reuse tokens and established component anatomy.

Novel UI patterns require explicit design review.

### Principle 4 — Dark First, Light Compatible

Dark mode is the default because photo editing benefits from a neutral dark environment.

Light mode is supported through tokens but does not drive the initial visual identity.

### Principle 5 — Pro, but not hostile

Advanced controls may exist, but they must be progressively disclosed.

Basic tools should be immediately understandable.

### Principle 6 — Fast Feedback

Editing controls are judged by response time.

A beautiful slider that feels slow is a failed slider.

### Principle 7 — Native macOS Behavior

The app should respect:

- macOS window controls
- macOS menu conventions
- keyboard shortcuts
- sidebar/toolbar/inspector expectations
- system accent color option
- accessibility settings

### Principle 8 — AI as tool, not persona

AI capabilities should appear as normal photo-editing tools:

- Auto Tone
- Subject Mask
- Sky Mask
- Denoise
- Enhance

Avoid:

- AI Magic
- Copilot Center
- Agent Mode as primary UI
- AI chat-first editing

---

## 5. Color System

### 5.1 Color strategy

The application must use a neutral dark interface so that photos remain visually dominant.

Accent color should be used sparingly for selection, focus, primary actions, and active states.

Silica Amber is a brand accent, not a general UI paint bucket.

Default app accent behavior:

1. Use macOS/system blue for native selection and accessibility-friendly focus by default.
2. Offer Silica Amber as an optional brand accent.
3. Use Silica Amber in marketing, app icon, splash/empty states, and selected brand moments.

### 5.2 Dark mode tokens

```css
:root {
  --sr-bg-primary: #101010;
  --sr-bg-secondary: #181818;
  --sr-bg-tertiary: #202020;

  --sr-surface-panel: #1C1C1E;
  --sr-surface-card: #242426;
  --sr-surface-raised: #2A2A2C;
  --sr-surface-hover: #2C2C2E;
  --sr-surface-active: #333336;

  --sr-border-subtle: #2F2F31;
  --sr-border-default: #3A3A3C;
  --sr-border-strong: #4A4A4D;

  --sr-text-primary: #F5F5F7;
  --sr-text-secondary: #B8B8BD;
  --sr-text-tertiary: #7A7A80;
  --sr-text-disabled: #5C5C61;

  --sr-accent-system: #0A84FF;
  --sr-accent-silica: #D6A84F;
  --sr-accent-silica-hover: #E2B75E;
  --sr-accent-silica-muted: #6E5527;

  --sr-success: #30D158;
  --sr-warning: #FF9F0A;
  --sr-danger: #FF453A;
  --sr-info: #64D2FF;
}
```

### 5.3 Light mode tokens

```css
[data-theme="light"] {
  --sr-bg-primary: #F5F5F7;
  --sr-bg-secondary: #FFFFFF;
  --sr-bg-tertiary: #ECECEF;

  --sr-surface-panel: #FFFFFF;
  --sr-surface-card: #F7F7F8;
  --sr-surface-raised: #FFFFFF;
  --sr-surface-hover: #EFEFF1;
  --sr-surface-active: #E5E5EA;

  --sr-border-subtle: #D8D8DD;
  --sr-border-default: #C7C7CC;
  --sr-border-strong: #AEAEB2;

  --sr-text-primary: #1D1D1F;
  --sr-text-secondary: #515154;
  --sr-text-tertiary: #86868B;
  --sr-text-disabled: #AEAEB2;
}
```

### 5.4 Usage rules

- Background colors only for layout surfaces.
- Accent color only for active, selected, primary, or focus states.
- Semantic colors only for status, warnings, errors, and destructive actions.
- Never use accent colors to decorate panels.
- Never make photo thumbnails compete with accent color.

---

## 6. Typography

### 6.1 Font stack

```css
font-family:
  system-ui,
  -apple-system,
  BlinkMacSystemFont,
  "SF Pro Text",
  "Helvetica Neue",
  sans-serif;
```

### 6.2 Type scale

| Token | Size | Line Height | Weight | Usage |
|---|---:|---:|---|---|
| `--sr-type-display` | 28px | 34px | 700 | Empty state title, marketing-only |
| `--sr-type-title` | 22px | 28px | 650 | Dialog title |
| `--sr-type-heading` | 17px | 24px | 600 | Major section heading |
| `--sr-type-body` | 15px | 22px | 400 | Body text |
| `--sr-type-control` | 13px | 18px | 500 | Controls, panel labels |
| `--sr-type-caption` | 11px | 14px | 400 | Metadata, hints, small values |

### 6.3 Rules

- Inspector labels use 13px medium.
- Slider values use 13px tabular numbers where possible.
- Metadata rows use 11px or 13px depending on density.
- Avoid large typography in the editor surface.
- The photograph, not typography, should dominate the screen.

---

## 7. Spacing System

SilicaRAW uses an 8pt spacing system with limited exceptions.

```css
--sr-space-2xs: 2px;
--sr-space-xs: 4px;
--sr-space-sm: 8px;
--sr-space-md: 12px;
--sr-space-lg: 16px;
--sr-space-xl: 24px;
--sr-space-2xl: 32px;
--sr-space-3xl: 48px;
--sr-space-4xl: 64px;
```

Layout rules:

- Toolbar height: 48px
- Left sidebar default width: 260px
- Left sidebar min/max: 220px / 360px
- Right inspector default width: 320px
- Right inspector min/max: 280px / 420px
- Filmstrip default height: 112px
- Filmstrip min/max: 88px / 160px
- Panel internal padding: 16px
- Inspector section gap: 16px
- Control row gap: 8px

---

## 8. Radius System

```css
--sr-radius-xs: 4px;
--sr-radius-sm: 6px;
--sr-radius-md: 10px;
--sr-radius-lg: 14px;
--sr-radius-xl: 20px;
--sr-radius-full: 999px;
```

Usage:

- Buttons: 10px
- Cards: 14px
- Dialogs: 20px
- Thumbnail selection ring: 6px
- Sliders/segmented controls: full or 10px depending on component

Rule:

> Rounded corners should feel like macOS, not mobile SaaS.

---

## 9. Border, Elevation, and Materials

Borders are preferred over heavy shadows.

Use subtle borders for:

- Inspector separation
- Sidebar separation
- Cards
- Thumbnail active state
- Toolbar divider

Shadows are rare.

Use elevation only for:

- Dialogs
- Popovers
- Floating compare controls
- Context menus

```css
--sr-shadow-popover: 0 12px 36px rgba(0, 0, 0, 0.35);
--sr-shadow-dialog: 0 24px 80px rgba(0, 0, 0, 0.45);
```

Because Tauri uses web UI, avoid pretending to be a fully native NSVisualEffectView everywhere.

Use restrained translucent effects only in:

- Toolbar
- Floating viewer controls
- Popover backgrounds

Do not use heavy glassmorphism across panels.

---

## 10. Motion System

```css
--sr-motion-fast: 120ms;
--sr-motion-normal: 180ms;
--sr-motion-slow: 260ms;

--sr-ease-standard: cubic-bezier(0.2, 0.8, 0.2, 1);
--sr-ease-out: cubic-bezier(0.16, 1, 0.3, 1);
--sr-ease-in-out: cubic-bezier(0.4, 0, 0.2, 1);
```

Usage:

- Hover: 120ms
- Button press: 120ms
- Panel expand/collapse: 180ms
- Viewer zoom: 180ms
- Dialog open/close: 260ms
- Before/after split drag: immediate, not animated except release snap

Respect reduced motion settings.

---

## 11. Iconography

Use SF Symbols-inspired geometry.

Rules:

- Outline icons by default
- Filled icons only for selected states
- 16px for dense controls
- 18px for toolbar
- 20px for empty states/sidebar highlights
- No mixed icon packs
- No colorful icons inside the editor except app icon/brand moments

---

## 12. Accessibility

All text and interactive controls must meet accessible contrast expectations.

Minimum keyboard support:

- Open folder
- Switch Library/Develop/Export
- Rate 1–5
- Reject
- Pick
- Next/previous photo
- Before/after
- Export
- Undo/redo
- Search
- Toggle panels

Focus rings must use system accent by default.

Minimum recommended interactive height:

- Toolbar button: 28px
- Sidebar item: 32px
- Inspector control row: 28px
- Slider row: 44px including label/value/track

---

## 13. Implementation Rules

Recommended style structure:

```txt
apps/desktop/src/styles/
├─ tokens.css
├─ themes.css
├─ typography.css
├─ motion.css
├─ components.css
└─ reset.css
```

Forbidden in components:

```css
color: #D6A84F;
padding: 13px;
border-radius: 7px;
```

Required:

```css
color: var(--sr-accent-silica);
padding: var(--sr-space-lg);
border-radius: var(--sr-radius-md);
```

---

## 14. Design Review Checklist

Every UI PR must answer:

- Does it use tokens?
- Does it reuse an existing component?
- Does it preserve photo-first hierarchy?
- Does it work in dark mode?
- Does it work in light mode, even if less polished?
- Does it respect keyboard navigation?
- Does it match the Apple Pro App feel?
- Does it avoid AI/MCP visual dominance?
- Does it perform acceptably with large photo sets?

---

## 15. Final Decision

Design System v1.0: GO WITH CONDITIONS

Required follow-up documents:

1. Component Library Specification
2. Screen Inventory & Wireframes
3. Color Management UI Specification
4. Accessibility Checklist
5. UI Performance Test Plan
