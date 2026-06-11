---
title: UI MVP Baseline
status: active
audience: all
updated: 2026-06-11
source_of_truth: docs/05_Design_System_Specification.md
---

# UI MVP Baseline

## Summary

Task 5.5 adds the product UI vertical slice that must exist before local install QA can mean anything to a human tester. The goal is not every future screen. The goal is a connected, token-driven UI MVP that can run the local alpha workflow from the app window:

```txt
create/open library -> import by reference -> browse grid -> cull -> develop exposure/contrast -> export JPEG sRGB
```

Phase 5.5 is a screen and command-wiring milestone, not the final installed-app readiness gate. Phase 5.6 follows it to replace placeholder/static behavior with the product runtime behaviors required before clean-Mac DMG QA.

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

## Task 5.5.4 Import Flow

Task 5.5.4 adds the M009 import progress structure inside the Library workbench:

- Import runs by reference through the Tauri `import_folder` command.
- The UI keeps the original-file safety statement visible before import starts.
- Import progress appears as a modal-style panel over the library surface, not as a page layout that resizes the grid.
- Unsupported and error counts are always visible, with a View Errors affordance.
- Static progress rows are placeholders until real progress events are explicitly scoped.
- Pause, cancel, recursive import, import history persistence, and native folder picking remain future work.

## Task 5.5.5 Library Grid

Task 5.5.5 replaces the placeholder grid with the M003 Library Grid MVP:

- The grid renders imported catalog rows from the desktop `list_library_photos` command.
- Selected photo, rating, pick, reject, missing, and unsupported states are visible in the grid or inspector.
- Rating, Pick, and Reject inspector actions route through the existing `set_photo_flags` command.
- Empty and loading states are represented without copying or decoding originals.
- Static thumbnail art remains a UI placeholder until real thumbnail cache generation and virtualization are scoped.
- Loupe opening, advanced filters, and real histogram values remain future work in this baseline. Phase 11 later replaces placeholder metadata rows with catalog-backed metadata inspector UI.

## Task 5.5.6 Preview/Loupe

Task 5.5.6 adds the M004 Loupe MVP inside Library mode:

- Loupe opens from the currently selected Library grid photo.
- Runtime preview state uses the existing desktop `open_photo_preview` command.
- JPEG/raster candidates render the ready preview surface.
- RAW candidates render the blocked decode state and must not imply RAW decoding is implemented.
- Unsupported catalog entries render a clear unsupported state.
- The bottom filmstrip mirrors the current grid selection and keeps M004 navigation visible.
- Real image pixels, Metal output, RAW decoding, and Develop edits remain future work in this baseline. Phase 11 later shares the catalog-backed metadata inspector with Loupe.

## Task 5.5.7 Develop Panel

Task 5.5.7 adds the M005 Develop Panel MVP:

- Develop mode uses the selected Library photo as its editing context.
- The center surface shows preview readiness, selected file name, Before/After/Split controls, and a bottom filmstrip.
- The right inspector exposes Basic exposure and contrast controls using the `SrAdjustmentSlider` anatomy: label, range, numeric input, reset, keyboard adjustment, and default reset.
- Draft slider and numeric input changes call only the preview path.
- `Commit Edit` is the explicit persistence action for final exposure and contrast values.
- Unsupported and missing files disable Develop edit controls rather than implying an editable preview.
- RAW candidates may accept a valid edit graph draft while still showing blocked decode preview state.
- Real image pixels, Metal output, RAW decoding, masks, full tone/color/detail controls, and sidecar writing remain future work.

## Task 5.5.8 Export Dialog

Task 5.5.8 adds the M007 Export Dialog MVP:

- Export mode and the toolbar Export action open the same modal dialog.
- The dialog uses the selected Library photo as the export target.
- The local-alpha settings are locked to JPEG, sRGB, and quality 90.
- The user can enter a local output path.
- The UI states that original files are not modified and blocks output paths that equal the referenced original path.
- Runtime export calls `export_photo_jpeg_srgb`.
- Static smoke mode does not claim to write a file; it shows that desktop runtime is required for the JPEG and catalog export record.
- RAW, missing, and unsupported candidates show blocked states instead of implying exportable pixels.
- Native folder picking, multi-photo export, export presets, alternate formats, resizing, metadata policy editing, real image pixels, RAW decoding, Metal output, and sidecar writing remain future work.

## Task 5.5.9 UI Workflow Smoke Harness

Task 5.5.9 adds a lightweight static harness for the connected local alpha UI workflow:

- `scripts/harness/check.sh` runs `scripts/harness/check-ui-workflow-smoke.py` after the static UI contract check.
- The smoke harness verifies the path `open/create library -> import by reference -> grid/cull -> loupe -> develop -> export`.
- It checks required element IDs, Tauri command wiring, import-by-reference copy safety text, Develop exposure/contrast bounds, locked JPEG sRGB export settings, static runtime messaging, and the guard that blocks exporting over the referenced original source path.
- It intentionally avoids browser automation and new dependencies so the check can run locally and in CI as part of the existing harness.
- It does not require MLX, MCP, plugin runtime, cloud, telemetry, RAW decoding, or Metal rendering.

## Task 5.5.10 Visual and Responsive QA

Task 5.5.10 checks the implemented M003/M005/M007 surfaces against their compact and large mockup families:

- Browser QA covers Library grid, Develop, and Export dialog at `1280x800`, `1440x900`, and `1728x965`.
- The pass checks horizontal overflow, visible control/text clipping, toolbar mode/action overlap, photo-first hierarchy, and Export/Develop compact usability.
- The 1280px toolbar density issue was fixed by allowing the mode switcher and search field to shrink before they collide.
- Real image pixels and thumbnail cache generation remain outside this static UI QA task.

See [UI Visual and Responsive QA](ui-visual-responsive-qa.md) for the recorded notes.

## Phase 11 Workspace Layout Preferences

Task 11.4 makes workspace layout an app-session preference, not frontend-only state.

Defaults are owned by `silica-core`:

| Preference | Default | Invalid Value Behavior |
|---|---:|---|
| `sidebar_collapsed` | `false` | Reset to `false` |
| `inspector_collapsed` | `false` | Reset to `false` |
| `filmstrip_visible` | `true` | Reset to `true` |
| `thumbnail_size` | `168` | Clamp to `132..=220` |
| `sort` | `imported_at_desc` | Reset to `imported_at_desc` |
| `filters.min_rating` | `null` | Clamp numeric values to `0..=5`; non-numeric resets to `null` |
| `filters.picked` | `null` | Non-boolean resets to `null` |
| `filters.rejected` | `null` | Non-boolean resets to `null` |
| `filters.file_type` | `null` | Unknown values reset to `null` |
| `filters.search` | `""` | Non-string resets to `""` |

Reset layout means only `AppSession.layout` returns to these defaults. It must not clear recents, last library, selected photo, culling flags, edit state, catalog rows, sidecars, caches, or original-file references.

The default sort remains `imported_at_desc` so Task 11.5 can introduce paged, sorted, and filtered query APIs without changing the first page ordering contract.

## QA Strategy

UI QA should happen in vertical slices, not after every future screen exists.

1. First QA the connected core workflow through M001/M003/M005/M007-level screens.
2. Then verify responsive variants for M003, M005, and M007.
3. Then complete Phase 5.6 runtime behavior: real JPEG/JPG pixels, native/selectable paths, persisted edit-state readback, cache clearing, fixture generation, and installed/runtime smoke.
4. Then expand secondary screens such as preferences, mask active, and AI review when they are explicitly scoped.

Minimum UI MVP checks:

- No text overlap at 1280px, 1440px, and 1728px widths.
- No native viewer placeholder or proof layer covers web UI.
- Keyboard focus is visible.
- Library/edit/export commands surface success and error states.
- Original-file safety is visible in import/export UI and verified by backend tests.

## Links

- [Local DMG Distribution Plan](../roadmaps/local-dmg-distribution-plan.md)
- [UI Mockups](ui-mockups.md)
- [UI Visual and Responsive QA](ui-visual-responsive-qa.md)
- [Product Alpha Runtime Completion](product-alpha-runtime-completion.md)
- [Metal Rendering](metal-rendering.md)
