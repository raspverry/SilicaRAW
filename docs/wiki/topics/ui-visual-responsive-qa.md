---
title: UI Visual and Responsive QA
status: active
audience: all
updated: 2026-06-11
source_of_truth: docs/wiki/topics/ui-mvp-baseline.md
---

# UI Visual and Responsive QA

## Summary

Task 5.5.10 verifies the connected UI MVP against the M003, M005, and M007 mockup families at compact, standard desktop, and large desktop widths.

The checked surfaces are:

```txt
Library grid -> Develop exposure/contrast -> Export JPEG sRGB dialog
```

This is a visual and responsive QA pass only. It does not add real image pixels, thumbnail cache generation, RAW decoding, Metal rendering, native folder picking, or export formats beyond JPEG sRGB.

This page remains the record for the static Phase 5.5 pass. It is not the final Phase 6 readiness gate. Phase 5.6.12 reruns visual/responsive QA after real JPEG/JPG thumbnails, loupe preview, Develop preview, path UX, cache UI, and demo-state cleanup are implemented.

Task 5.6.12 is now the Phase 6 readiness visual gate. The Phase 5.5 notes below remain historical context only.

## Final Phase 5.6.12 Refresh

Final QA command:

```bash
python3 scripts/harness/run-final-visual-qa.py
```

The script starts a local static server from the repository root, generates legal synthetic JPEG/JPG fixtures, uses Agent Browser to drive UI states, captures screenshots under `.tmp/final-visual-responsive-qa/screenshots`, and writes DOM metrics to `.tmp/final-visual-responsive-qa/visual-qa-results.json`.

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
- [Product Alpha Runtime Completion](product-alpha-runtime-completion.md)
- [Local DMG Distribution Plan](../roadmaps/local-dmg-distribution-plan.md)
