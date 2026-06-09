---
title: UI MVP Baseline
status: active
audience: all
updated: 2026-06-09
source_of_truth: docs/05_Design_System_Specification.md
---

# UI MVP Baseline

## Summary

Task 5.5 adds the product UI vertical slice that must exist before local install QA can mean anything to a human tester. The goal is not every future screen. The goal is a connected, token-driven UI MVP that can run the local alpha workflow from the app window:

```txt
create/open library -> import by reference -> browse grid -> cull -> develop exposure/contrast -> export JPEG sRGB
```

## Source Hierarchy

Use these sources in this order when implementing UI:

1. `docs/05_Design_System_Specification.md`
2. `docs/05_5_Component_Library_Specification.md`
3. `docs/06_Screen_Inventory_and_Wireframe_Specification.md`
4. `MockupUI/MANIFEST.md` and the relevant `MockupUI/*.png`
5. This wiki baseline

Do not use `docs/archive/` screen specifications for implementation.

## Mockup Mapping

The Task 5.5 UI MVP should prioritize these mockups:

| Workflow | Primary Mockup | Later Responsive References |
|---|---|---|
| Welcome and library open/create | `M001_Welcome.png` | none |
| Library grid and culling | `M003_Library_Grid_populated.png` | `M011_Library_Grid_compact_1280.png`, `M012_Library_Grid_large_1728.png` |
| Preview/loupe | `M004_Library_Loupe.png` | none |
| Develop controls | `M005_Develop_default.png` | `M013_Develop_compact_1280.png`, `M014_Develop_large_1728.png` |
| Export | `M007_Export_Dialog.png` | `M015_Export_Dialog_compact_1280.png`, `M016_Export_Dialog_large_1728.png` |
| Import feedback | `M009_Import_Progress.png` | none |

`M006_Develop_mask_active.png`, `M008_Preferences_Appearance.png`, and `M010_AI_Review.png` remain references for later work unless a task explicitly expands scope.

## Task 5.5.1 Decisions

`ui-ux-pro-max` was used on 2026-06-09 to generate and cross-check a UI design-system recommendation for a professional macOS RAW photo editor. The useful recommendations were:

- Dark-first interface.
- High contrast text.
- Restrained motion.
- Visible keyboard focus.
- Responsive checks at compact and desktop widths.

The generated purple/cyan palette, newsletter/content-first pattern, heavy glow direction, and Inter font recommendation were rejected because they conflict with SilicaRAW's documented Apple Pro App direction. SilicaRAW keeps:

- Neutral dark photo-editing surfaces.
- System font stack.
- System blue focus/selection by default.
- Silica Amber as a restrained brand accent.
- Photo-first hierarchy.
- No decorative neon, gradient-orb, cyber, SaaS dashboard, or marketing-page visual language.

## Static Frontend Baseline

Task 5.5.1 starts the static frontend token baseline:

```txt
apps/desktop/static/styles/
|- tokens.css
|- base.css
`- app-frame.css
```

Rules:

- UI implementation must consume `tokens.css`.
- Hard-coded visual values outside token/base/component definitions should be treated as review findings.
- The current static shell may remain minimal until Task 5.5.2 adds the app frame.
- Tauri command calls should use the configured global API, `window.__TAURI__.core.invoke`, while `app.withGlobalTauri` remains enabled in `tauri.conf.json`.

## Task 5.5.2 App Frame

Task 5.5.2 establishes the global shell used by later UI slices:

- Top toolbar with macOS-style window affordance space, compact actions, and Library/Develop/Export mode navigation.
- Left sidebar, central work surface, right inspector, and bottom status bar regions.
- Mode panels for Library, Develop, and Export in each region, with inactive panels using the native `hidden` attribute.
- Visible navigation state through `aria-pressed` and `data-active-mode`.
- Existing create/open library command wiring preserved in the Library mode panel.

This task intentionally stops at frame and navigation structure. The Welcome screen, import flow, populated library grid, loupe, Develop controls, and Export dialog are implemented in later Task 5.5 slices.

## Visual Parity Gate

Mockup parity is evaluated in two layers:

- Frame parity: toolbar height, mode navigation, sidebar width, inspector width, bottom status area, tokenized colors, focus states, and shared spacing/type rhythm.
- Screen parity: populated content, command states, photo thumbnails, filmstrip, Develop sliders, and Export dialog details.

Task 5.5.2 must pass frame parity before later screen tasks proceed. It does not need to pass full screen parity against M003, M005, or M007 because those screenshots include populated grid, loupe/develop viewer, filmstrip, and export dialog work from later tasks.

Design consistency rules for every Task 5.5 screen:

- Use the existing `tokens.css`, `base.css`, and `app-frame.css` scales before introducing new component values.
- Preserve the 8pt spacing rhythm except for documented 1px borders and fixed app chrome.
- Keep toolbar, sidebar, inspector, and bottom status sizing consistent across modes unless a mode-specific mockup explicitly differs.
- Do not add raw color literals to screen CSS; add semantic tokens first.
- Do not use decorative gradients, emoji icons, or one-off typography.

## Task 5.5.3 Welcome State

Task 5.5.3 adds the M001 first-launch state inside Library mode:

- The app frame defaults to `data-library-state="welcome"`.
- Welcome hides left sidebar, right inspector, and bottom status chrome while preserving the shared toolbar.
- Open Folder and Create Library use the existing Tauri `open_library` and `create_library` command names.
- A path field remains visible until a native folder picker is introduced by a later scoped task.
- Open Recent is an affordance that selects a displayed recent path in the static shell.
- Open Sample Project is visible but disabled until sample project support is explicitly scoped.
- Successful Tauri command completion switches the app frame to `data-library-state="open"`.

## QA Strategy

UI QA should happen in vertical slices, not after every future screen exists.

1. First QA the connected core workflow through M001/M003/M005/M007-level screens.
2. Then verify responsive variants for M003, M005, and M007.
3. Then expand secondary screens such as preferences, mask active, and AI review.

Minimum UI MVP checks:

- No text overlap at 1280px, 1440px, and 1728px widths.
- No native viewer placeholder or proof layer covers web UI.
- Keyboard focus is visible.
- Library/edit/export commands surface success and error states.
- Original-file safety is visible in import/export UI and verified by backend tests.

## Links

- [Local DMG Distribution Plan](../roadmaps/local-dmg-distribution-plan.md)
- [UI Mockups](ui-mockups.md)
- [Metal Rendering](metal-rendering.md)
