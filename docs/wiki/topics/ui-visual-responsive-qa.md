---
title: UI Visual and Responsive QA
status: active
audience: all
updated: 2026-06-20
source_of_truth: docs/wiki/roadmaps/post-alpha-product-roadmap.md#task-221-expanded-visual-qa-surface-set
---

# UI Visual and Responsive QA

## Summary

Final visual QA verifies the connected desktop UI against the current product surface set at compact, standard desktop, and large desktop widths.

The checked route now covers:

```txt
Welcome -> Library -> Loupe -> Develop -> Masks -> History -> Preferences -> Export
```

This is a visual and responsive QA pass only. It does not add product functionality, RAW decoding, Metal rendering, native folder picking, MLX, MCP, plugin runtime, or broad fallback behavior.

The earlier Phase 5.5 notes remain historical context. The current final runner is `scripts/harness/run-final-visual-qa.py`.

## Phase 22 Expanded Surface Set

Task 22.1 expanded final visual QA beyond the original MVP path. Task 24.4 adds the AI Review surface. Task 24.5 adds the AI approval state. The blocked-gate UI hardening path adds the unsupported-grid state. The runner now checks 25 surfaces at `1280x800`, `1440x900`, and `1728x965`, producing 75 screenshots with no horizontal overflow, toolbar overlap, clipped controls, or seeded-state failures.

Task Q2.1 adds a separate [UI Mockup Parity Checklist](ui-mockup-parity-checklist.md) for product-level visual review. The checklist exists because automated visual QA can pass while a screen still fails the mockup's photo-first hierarchy, density, or state-treatment target.

Task Q2.2 updates the visual QA state seeding for populated Library and Loupe parity. Populated Library screenshots now place the photo grid before disposable cache maintenance when photos exist, and Loupe screenshots hide Library maintenance chrome so the viewer surface is the first read.

Expanded Task 22.1 surfaces:

| Surface | State |
|---|---|
| `M015-library-filters` | Library search, file type, metadata, rating, culling, and sort filters seeded |
| `M016-library-metadata` | Catalog-backed selected-photo metadata readback |
| `M017-develop-history` | Develop history entries plus undo/redo command state |
| `M018-develop-expanded` | Expanded Develop panels including tone, HSL, Detail boundary, and Lens/Geometry |
| `M019-mask-editor` | Manual Mask Editor readback and unavailable AI/MLX controls |
| `M020-preferences-appearance` | Preferences dialog Appearance pane |
| `M021-preferences-advanced` | Disabled advanced access gates |
| `M022-export-workflow` | Export color, metadata, batch progress, failures, and recent exports |
| `M023-ai-review` | Read-only blur review, disabled approval, and non-mutating action preview |
| `M024-ai-approval` | Approvable AI suggestion, enabled approve/reject controls, and undo/rejection trust copy |
| `M025-library-unsupported-grid` | Unsupported imported rows in the Library grid without card overlap or repeated noisy warnings |

Current QA command:

```bash
python3 scripts/harness/run-final-visual-qa.py
```

## Phase 17 Histogram Note

Task 17.3 replaces the inspector's fake histogram placeholder with command-backed luminance bars from real histogram data. The histogram surface must keep the existing inspector footprint, avoid text overlap at the checked desktop widths, and show explicit blocked or unavailable text when real data is not available.

## Phase 17 Reset Preset Note

Task 17.4 makes the existing left Develop preset rows active, keeps Reset All inside the Develop inspector action area, and keeps Before/After in the preview toolbar as a two-state view-only control. These controls must use existing dark editor tokens and must not add horizontal preset strips or duplicate preset badges that crowd the current workbench.

## Phase 17 Develop P0 Visual QA Refresh

Task 17.5 reruns final visual QA after the full P0 Develop control set, real histogram display, reset, Before/After, and basic presets are present.

Current QA command:

```bash
python3 scripts/harness/run-final-visual-qa.py
```

The current runner starts a local static server, generates legal synthetic JPEG/JPG fixtures, drives the UI through direct Chrome DevTools Protocol commands, captures screenshots under `.tmp/final-visual-responsive-qa/screenshots`, and writes DOM metrics to `.tmp/final-visual-responsive-qa/visual-qa-results.json`. It prefers `SILICARAW_CHROME`, then system Chrome, then local Chrome for Testing. No extra project dependency is required.

Phase 17 checked 12 surfaces at `1280x800`, `1440x900`, and `1728x965`, producing 36 screenshots with zero horizontal overflow, zero toolbar overlap, zero clipped controls, and zero Develop state failures.

Develop-specific checks now verify:

- Selected-photo state is visible in the Develop header.
- Histogram status does not report an empty selection when a visual QA photo is selected.
- Before/After controls are available in the selected-photo Develop state.
- Exactly one basic preset is active and no preset rows are disabled.

Visual fixes from this pass:

- Made the final visual QA runner independent of the blocked `agent-browser screenshot` path by using direct Chrome CDP.
- Updated Develop visual fixture state so histogram, Before/After, and basic preset controls match the selected-photo screen.
- Added a small histogram status badge so text remains readable over histogram bars.

## Phase 18 Tone Curve Panel Note

Task 18.1.3 adds a compact Tone Curve panel to the Develop inspector. The panel exposes only the supported RGB point-curve midpoint control and keeps channel and parametric controls hidden and disabled until those modes have end-to-end runtime support.

Visual QA now verifies the tone curve panel at `1280x800`, `1440x900`, and `1728x965` with no horizontal overflow, no toolbar overlap, no clipped controls, the `Point RGB` support state, and unsupported curve controls remaining disabled.

## Phase 18 HSL Panel Note

Task 18.2.3 adds a compact HSL Mixer panel to the Develop inspector. The panel exposes the schema-owned red, orange, yellow, green, aqua, blue, purple, and magenta channels with hue, saturation, and luminance controls bounded to `-100..100`.

Visual QA now seeds the blue channel and verifies the HSL support state plus hue, saturation, and luminance values across `1280x800`, `1440x900`, and `1728x965` with no horizontal overflow, no toolbar overlap, and no clipped controls.

## Phase 18 Detail Panel Note

Task 18.3.3 adds a disabled Detail readback panel to the Develop inspector. The panel shows schema-owned sharpening and non-MLX noise-reduction values, keeps all Detail and MLX Denoise controls disabled, and surfaces the renderer/export boundary instead of implying an active pixel effect.

Visual QA now seeds a blocked non-neutral Detail state and verifies the blocked status, renderer/export boundary copy, readback value, and disabled controls across `1280x800`, `1440x900`, and `1728x965`.

## Phase 18 Lens Geometry Panel Note

Task 18.4.3 adds a Lens & Geometry panel to the Develop inspector. The panel exposes supported normalized crop, clear crop, quarter-turn rotate, and horizontal/vertical flip controls while keeping lens correction and transform controls disabled until runtime support exists.

Visual QA now seeds a supported crop/flip state and verifies geometry readiness, crop values, flip state, lens unavailable copy, transform unsupported copy, and disabled unsupported controls across `1280x800`, `1440x900`, and `1728x965`.

## Phase 18 Edit Clipboard Note

Task 18.5.3 adds a Develop Copy & Sync panel with explicit selected-page scope, subset checkboxes, and separate Copy, Paste to Primary, and Batch Sync actions. Copy, paste, and batch sync are gated to JPEG/JPG Develop photos; Detail and Lens clipboard subsets remain disabled because their runtime behavior is not supported by the current alpha.

Visual QA now seeds an edit clipboard state and verifies source readback, selected-photo count, Basic/Tone/Geometry subset choice, disabled unsupported subsets, enabled paste/sync actions, and plan rows across `1280x800`, `1440x900`, and `1728x965`.

## Phase 19 Mask Editor Note

Task 19.5 adds a compact Develop Mask panel for committed manual mask readback. The panel shows selected-photo scope, manual brush/linear/radial rows, the active mask geometry summary, and local exposure/contrast readback while keeping Add Mask, Subject/Sky, AI, and MLX paths unavailable.

Visual QA now adds `M006-mask-active` and verifies active manual mask state, selected-photo scope, disabled unsupported mask controls, readback-only local adjustment values, RAW mask blocked boundary copy, and no horizontal overflow, toolbar overlap, or clipped controls across `1280x800`, `1440x900`, and `1728x965`.

## Final Phase 5.6.12 Refresh

Final QA command:

```bash
python3 scripts/harness/run-final-visual-qa.py
```

The script starts a local static server from the repository root, generates legal synthetic JPEG/JPG fixtures, captures screenshots under `.tmp/final-visual-responsive-qa/screenshots`, and writes DOM metrics to `.tmp/final-visual-responsive-qa/visual-qa-results.json`.

Final checked surfaces:

| Surface | Reference |
|---|---|
| Welcome | `M001_Welcome.png` |
| Library empty | `M002_Library_Grid_empty.png` |
| Library populated | `M003_Library_Grid_populated.png`, `M011`, `M012` |
| Loupe | `M004_Library_Loupe.png` |
| Develop | `M005_Develop_default.png`, `M013`, `M014` |
| Export | `M007_Export_Dialog.png`, `M015`, `M016` |
| Maintenance minimal | `M008_Preferences_Appearance.png` as the local-alpha maintenance subset |
| Import progress | `M009_Import_Progress.png` |
| Sidebar collapsed | Phase 11 layout persistence state |
| Inspector collapsed | Phase 11 layout persistence state |
| Layout reset | Phase 11 layout persistence state |

Final DOM results:

| Viewport | Surfaces | Horizontal Overflow | Toolbar Overlap | Control Clipping | Result |
|---|---:|---|---|---|---|
| `1280x800` | 11 | false | false | 0 | Pass |
| `1440x900` | 11 | false | false | 0 | Pass |
| `1728x965` | 11 | false | false | 0 | Pass |

Final screenshot review:

- M001 welcome keeps the first-launch structure, honest alpha capability copy, empty recents, and no fake demo project rows.
- M002 empty library shows a real empty grid state without fictional photos.
- M003 populated library uses generated legal JPEG fixture pixels for thumbnails and keeps culling controls aligned at compact width.
- M004 Loupe and M005 Develop scale real preview images to the viewer instead of rendering small natural-size fixture pixels.
- M007 Export shows selected-photo thumbnail pixels in the dialog preview instead of placeholder art.
- M008-minimal is represented by the local-alpha maintenance/cache-clear subset with precise destructive-scope copy.
- M009 Import progress keeps overall and per-step progress states synchronized.
- M010/M011/M012 cover Phase 11 sidebar-collapsed, inspector-collapsed, and reset layout states with no horizontal overflow or clipped controls.
- Task 11.8.1 adds catalog-backed metadata rows to the shared Library/Loupe inspector; full visual regression screenshots remain covered by later visual QA runs.

Fixes from this final pass:

- Scaled `.sr-loupe-image` and `.sr-develop-image` to fill their viewer boxes with `object-fit: contain`.
- Added `.sr-export-preview-image` so Export preview uses selected thumbnail pixels when available.
- Added import step state markers and UI updates so completed imports no longer show pending step rows.
- Shortened cache-maintenance status copy and restored system font styling for that output.

## Method

Local static UI server:

```bash
cd apps/desktop/static
python3 -m http.server 4173 --bind 127.0.0.1
```

Browser QA:

- Tooling: Playwright MCP against `http://127.0.0.1:4173/index.html`
- Viewports: `1280x800`, `1440x900`, `1728x965`
- States: Library open grid, Develop mode, Export mode with dialog open
- DOM checks: horizontal overflow, visible control/text clipping candidates, toolbar mode/action overlap
- Visual checks: screenshot review against the relevant mockup family

Relevant mockups:

| Surface | Primary | Compact | Large |
|---|---|---|---|
| Library grid | `M003_Library_Grid_populated.png` | `M011_Library_Grid_compact_1280.png` | `M012_Library_Grid_large_1728.png` |
| Develop | `M005_Develop_default.png` | `M013_Develop_compact_1280.png` | `M014_Develop_large_1728.png` |
| Export | `M007_Export_Dialog.png` | `M015_Export_Dialog_compact_1280.png` | `M016_Export_Dialog_large_1728.png` |

## Results

| Viewport | Library Grid | Develop | Export Dialog | Result |
|---|---|---|---|---|
| `1280x800` | 4-column grid, sidebar, inspector, and status bar fit without horizontal overflow. | Preview, filmstrip, and compact inspector remain usable. | Dialog remains centered and usable. | Pass after toolbar density fix. |
| `1440x900` | Grid and inspector preserve photo-first hierarchy. | Preview remains primary; controls fit without clipping. | Dialog settings and summary fit without clipping. | Pass. |
| `1728x965` | Grid expands to 6 columns and leaves stable inspector space. | Preview scales wider while inspector remains readable. | Dialog stays centered and does not over-expand. | Pass. |

Observed automated DOM results after the fix:

```txt
horizontalOverflow: false
visible clipping candidates: 0
toolbar mode/action overlap: 0
```

## Fix Applied

The initial 1280px Develop screenshot showed the toolbar mode switcher and search/actions region colliding visually. The root cause was the mode switcher keeping a fixed `min-width` while its grid track could become narrower.

The CSS fix:

- Lets the mode switcher shrink inside its toolbar grid track.
- Uses a denser mode switcher and search width below 1440px.
- Keeps the wider mode switcher for larger desktop widths where there is enough toolbar space.

## Notes

- Static thumbnail art remains a placeholder until real thumbnail cache generation is scoped.
- User-supplied sample imagery should be used later for import, thumbnail, preview, and local install QA. It was not committed or wired into this static UI QA task because real image pixels are outside Task 5.5.10 scope.
- Temporary screenshots captured during this pass were review artifacts, not committed assets.
- Phase 14.8 reuses viewport targets `1280x800`, `1440x900`, and `1728x965` for native viewer bridge QA. That pass is recorded separately in [Native Viewer QA Checklist](../../../checklists/NATIVE_VIEWER_QA.md) because feature-gated native viewer proof must not be confused with the default static visual QA path.

## Links

- [UI MVP Baseline](ui-mvp-baseline.md)
- [UI Mockups](ui-mockups.md)
- [UI Mockup Parity Checklist](ui-mockup-parity-checklist.md)
- [Product Alpha Runtime Completion](product-alpha-runtime-completion.md)
- [Local DMG Distribution Plan](../roadmaps/local-dmg-distribution-plan.md)
