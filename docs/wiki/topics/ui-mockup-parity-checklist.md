---
title: UI Mockup Parity Checklist
status: active
audience: designers, maintainers, agents
updated: 2026-06-20
source_of_truth: docs/wiki/roadmaps/local-alpha-quality-closure-plan.md
---

# UI Mockup Parity Checklist

## Summary

This checklist compares the current visual QA screenshots against the committed `MockupUI/` targets for the local alpha UI polish phase.

The current screenshots pass the automated layout gate for overflow, toolbar overlap, and obvious clipping. They do not yet meet the product-level visual target. The largest remaining issue is that several screens still read like a diagnostic shell instead of a photo-first editor.

## Inputs

- Mockup references: `MockupUI/M003`, `M004`, `M005`, `M007`, `M008`, `M009`, and `M010`.
- Current visual QA artifacts: `.tmp/final-visual-responsive-qa/screenshots`.
- Current visual QA result file: `.tmp/final-visual-responsive-qa/visual-qa-results.json`.
- Current runner result: 28 surfaces, 84 screenshots, 3 desktop viewports.
- Review baseline: dark professional desktop photo editor, content-first, image-first, dense but readable controls.

Regenerate current screenshots before using this page for implementation:

```bash
python3 scripts/harness/run-final-visual-qa.py
```

## Severity Terms

| Severity | Meaning |
|---|---|
| P0 | Blocks the screen from reading as the intended product workflow. |
| P1 | The workflow is understandable, but hierarchy, density, or state treatment harms confidence. |
| P2 | Lower-risk polish that should not delay the core editor path. |

## Surface Checklist

| Surface | References | Current artifact | Severity | Actionable delta |
|---|---|---|---|---|
| Library Grid | `MockupUI/M003_Library_Grid_populated.png`, `M011`, `M012` | `desktop-1440-M003-library-populated.png` | P1 | Q2.2 moves the photo grid ahead of disposable cache maintenance when photos exist. Q2.3 reduces thumbnail state noise by removing repeated file-type and empty-rating labels from unsupported/missing cards. Q2.4 replaces gradient/checker fixtures with photo-like reference JPEGs and prevents unavailable rows from leaking stale thumbnails. Q2.5 makes the right inspector less competitive with browsing by tightening section, button, and metadata row rhythm. |
| Library Loupe | `MockupUI/M004_Library_Loupe.png` | `desktop-1440-M004-loupe.png`, `M026`, `M027` | P1 | Q2.2 hides Library header and maintenance chrome in Loupe so the viewer becomes the primary read. Q2.4 adds separate ready, unsupported, and missing-original evidence; unavailable states show no preview image and no ready histogram. Q2.5 reduces inspector density and disabled-button noise. |
| Develop | `MockupUI/M005_Develop_default.png`, `M013`, `M014` | `desktop-1440-M005-develop.png`, `M028` | P0 | Q2.4 adds explicit unsupported Develop evidence with no preview image and unsupported mask copy. Q2.5 aligns right-panel rhythm and verifies ready Develop mask support copy stays `Manual`. Remaining work belongs to Q2.6: reduce boundary-copy prominence and resolve remaining blocked/future visual contradictions. |
| Export Dialog | `MockupUI/M007_Export_Dialog.png`, `M015`, `M016` | `desktop-1440-M007-export.png`, `M022` | P1 | Q2.5 lowers Export summary and recent-export heading weight, aligns label/value rhythm, normalizes two-line selection card height, and separates batch failure file names from block reasons. Remaining work belongs to Q2.6 if export states still contradict action readiness. |
| Preferences Appearance | `MockupUI/M008_Preferences_Appearance.png` | `desktop-1440-M020-preferences-appearance.png` | P2 | Current scope is intentionally smaller than the mockup. Q2.5 aligns Preferences two-line section selectors and disabled control weight with Export and inspector rhythm. Remaining work belongs to Q2.6 if Advanced disabled content remains too prominent. |
| Import Progress | `MockupUI/M009_Import_Progress.png` | `desktop-1440-M009-import-progress.png` | P1 | Import sheet is honest but visually weak. Increase modal prominence, clarify completed/in-progress step contrast, align summary cards, and suppress background competition. |
| AI Review | `MockupUI/M010_AI_Review.png` | `desktop-1440-M023-ai-review.png` | P0 | Read-only AI Review is honest but too small and status-like. Treat results as the primary workspace, separate summary/criteria/action preview, and reduce repeated unavailable/review-only copy. |

## Task Routing

Use this routing when implementing Phase Q2. Keep the execution order from the Local Alpha Quality Closure Plan.

| Task | Primary screens | Fix scope |
|---|---|---|
| Q2.2 Restore Photo-First Library Hierarchy | M003, M004 | Completed: make the grid/loupe photo-first and demote maintenance when photos exist. |
| Q2.3 Fix Thumbnail Card State Density | M003, M025 | Completed: keep card badges, filename, rating, pick/reject, and unsupported state readable without repeated warning noise. |
| Q2.4 Make Image Preview States Honest | M004, M005, M026, M027, M028 | Completed: render supported JPEG/JPG from photo-like legal fixtures, block stale thumbnail leakage for unavailable rows, and keep unsupported/missing preview surfaces image-free with explicit copy. |
| Q2.5 Unify Inspector Density and Panel Rhythm | M003, M004, M005, M007, M008, M020, M022 | Completed: align headings, row spacing, labels, button sizes, disabled states, failure rows, and status-copy rhythm across right panels and dialogs. |
| Q2.6 Resolve Known Visual Contradictions | M007, M008, M010, plus M017/M021/M022/M023/M024 | Keep future or blocked paths honest without making unavailable features louder than the editor workflow. |

## Product Constraints

- Do not implement new product features while resolving these parity deltas.
- Do not enable RAW decoding, MLX, MCP, plugin runtime, cloud, telemetry, auto-update, Homebrew, or Mac App Store paths.
- Do not use mockup PNGs as runtime fixtures or image-processing evidence.
- Do not commit private sample images.
- Do not hide unsupported or missing-original states; demote them visually only when they are not the user's current task.

## Validation

For docs-only updates, run:

```bash
python3 scripts/harness/check-md-links.py
```

For UI-affecting implementation tasks, run:

```bash
python3 scripts/harness/check-static-ui.py
python3 scripts/harness/check-ui-workflow-smoke.py
python3 scripts/harness/run-final-visual-qa.py
scripts/harness/check.sh
```

Manual review must inspect at least the standard `1440x900` screenshot for each touched surface and the compact `1280x800` screenshot for any toolbar, sidebar, filmstrip, card, or dialog density change.

## Notes for LLM Agents

Passing DOM metrics does not mean the UI meets the mockup target. Use this page to choose the smallest visual fix that moves the current task forward. Keep visual polish tied to the local alpha workflow: import, browse, rate/pick/reject, preview, adjust exposure/contrast, persist, and export JPEG sRGB.
