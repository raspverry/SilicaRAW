---
title: Product Alpha Runtime Completion
status: active
audience: all
updated: 2026-06-10
source_of_truth: docs/wiki/roadmaps/local-dmg-distribution-plan.md
---

# Product Alpha Runtime Completion

## Summary

Phase 5.6 turns the Phase 5.5 UI vertical slice from a static/command-wired shell into a usable local alpha app. The goal is not to pass packaging QA early. The goal is to make the installed app complete enough that Phase 6 can validate real behavior from `/Applications`.

Phase 5.5 proved the screen structure and command paths. Phase 5.6 must prove the runtime product loop:

```txt
create/open library -> import supported raster photos by reference -> real grid thumbnail -> real loupe preview -> visible exposure/contrast preview -> commit edit -> export JPEG sRGB -> clear disposable caches -> reopen and restore state
```

## Alpha Capability Contract

The installed local alpha must be honest about what works.

Required before Phase 6:

- JPEG/JPG, PNG, TIF, and TIFF originals are the supported standard-raster visible photo path.
- Supported-raster grid thumbnails, loupe preview, Develop preview, edit persistence, and JPEG sRGB export must work through the installed app.
- RAW files may be imported as catalog entries only if they show a clear decode-blocked state.
- Unsupported files must never look editable or exportable.
- Original source files must remain unmodified.

Deferred until explicit later tasks:

- RAW decoding.
- Native Metal viewer output.
- Display P3 export for PNG/TIFF.
- HEIC, WebP, and other raster formats as guaranteed installed-alpha edit/export inputs.
- Masks, AI review, MLX, MCP, plugins, cloud sync, telemetry, auto-update, Homebrew, and Mac App Store distribution.

## Current Runtime Gap Audit

This audit records why Phase 6 clean-Mac testing must wait for Phase 5.6.

| Area | Current State | Product Gap | Required Before Phase 6 |
|---|---|---|---|
| Library path UX | Native/selectable create/open path UX is implemented. | None for the current local alpha path. | Completed in Task 5.6.3. |
| Import path UX | Native/selectable import folder UX is implemented. | None for the current local alpha path. | Completed in Task 5.6.3. |
| Export path UX | Native save-location UX is implemented and cancel is distinct from error. | None for the current local alpha path. | Completed in Task 5.6.3. |
| Command responses | Tauri commands return structured response envelopes. | None for covered local alpha commands. | Completed in Task 5.6.2. |
| Grid | Supported-raster rows render real cached thumbnail pixels. | RAW/missing/unsupported rows intentionally remain blocked or placeholder states. | Completed in Task 5.6.4 and widened for PNG/TIFF source inputs. |
| Loupe | Supported-raster rows render real cached Loupe preview pixels. | RAW/missing/unsupported rows intentionally remain blocked or placeholder states. | Completed in Task 5.6.5 and widened for PNG/TIFF source inputs. |
| Develop | Supported-raster exposure/contrast drafts render real adjusted preview pixels and Develop reads committed edit state on selection/open. | RAW/missing/unsupported rows intentionally remain blocked or placeholder states. | Pixel preview completed in Task 5.6.6; persisted readback completed in Task 5.6.7. |
| Edit state restore | Frontend reads the active catalog edit state through `get_photo_edit_state` and restores sliders plus clean/dirty state. | None for the current supported-raster local alpha path. | Completed in Task 5.6.7. |
| Export support | Export path writes JPEG/PNG/TIFF sRGB outputs from supported raster sources. | Display P3 remains JPEG-only until color-managed PNG/TIFF proof exists. | Completed with raster export support and source-input widening. |
| Cache clear | Product command and Library maintenance UI clear only `thumbnails`, `previews`, `render-cache`, `ai-cache`, then recreate those directories. | None for the current local alpha cache clear scope. | Completed in Task 5.6.8. |
| Recents/session | Clean launch shows an empty recent state and no fictional library rows. | Real persisted recents/session restore remains deferred beyond the current local alpha hardening pass. | Fake demo state removed in Task 5.6.9. |
| Culling controls | Library inspector supports rating 0 through 5, pick/reject toggles, and clear culling flags with coherent mutual exclusion. | None for the current local alpha culling scope. | Completed in Task 5.6.9. |
| Runtime QA | Harness now generates legal synthetic QA fixtures, records a developer installed-app preflight report, runs a connected desktop command runtime smoke, and has a final visual/responsive QA refresh. | Installed DMG execution on a clean Mac remains a later Phase 6 gate. | Tasks 5.6.10 through 5.6.12 complete the developer-side runtime and visual gates. |

## Required Phase 5.6 Task Order

1. `Task 5.6.1: Runtime Gap Audit and Alpha Capability Contract`
2. `Task 5.6.2: Structured Desktop Command Responses`
3. `Task 5.6.3: Native Path Picker UX`
4. `Task 5.6.4: Real JPEG Thumbnail Cache MVP`
5. `Task 5.6.5: Real JPEG Loupe Preview MVP`
6. `Task 5.6.6: Real JPEG Develop Preview MVP`
7. `Task 5.6.7: Persisted Edit-State Readback in UI`
8. `Task 5.6.8: Product Cache Clear Command and Maintenance UI`
9. `Task 5.6.9: Remove Fake Demo State and Harden Culling UX`
10. `Task 5.6.10: Legal QA Fixture Generator and Installed-App Preflight`
11. `Task 5.6.11: Connected Runtime UI Smoke`
12. `Task 5.6.12: Final Visual and Responsive QA Refresh`

## Acceptance Gate

Phase 6 can resume only after Phase 5.6 proves:

- A tester can complete the local alpha workflow without editing code or relying on static demo rows.
- The app shows real supported-raster pixels in grid, loupe, and Develop surfaces.
- Exposure/contrast edits are visible, committed, and restored after reopen.
- Export writes a separate JPEG sRGB file and never overwrites the source.
- Product cache clearing deletes only disposable cache data.
- The installed/runtime smoke path passes before clean-Mac DMG testing starts.

## Links

- [Local DMG Distribution Plan](../roadmaps/local-dmg-distribution-plan.md)
- [Installed-App Developer Preflight](../../../checklists/INSTALLED_APP_PREFLIGHT.md)
- [Connected Developer Runtime Smoke](../../../checklists/CONNECTED_RUNTIME_SMOKE.md)
- [UI Visual and Responsive QA](ui-visual-responsive-qa.md)
- [UI MVP Baseline](ui-mvp-baseline.md)
- [UI Mockups](ui-mockups.md)
- [Data Safety](data-safety.md)
- [Catalog](catalog.md)

## Notes for LLM Agents

Do not treat Phase 6 as a workaround for missing app behavior. If a Phase 6 checklist item needs a feature that still requires shell commands, static demo data, or manual filesystem manipulation outside the app, add or complete a Phase 5.6 task first.
