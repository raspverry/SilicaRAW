---
title: UI Visual and Responsive QA
status: active
audience: all
updated: 2026-06-09
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

## Links

- [UI MVP Baseline](ui-mvp-baseline.md)
- [UI Mockups](ui-mockups.md)
- [Local DMG Distribution Plan](../roadmaps/local-dmg-distribution-plan.md)
