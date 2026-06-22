---
title: Wiki Log
status: active
audience: all
updated: 2026-06-20
source_of_truth: none
---

# Wiki Log

## Summary

This append-only log records meaningful changes to the SilicaRAW wiki.

## Entries

## [2026-06-22] quality | Visual QA docs drift check added

- Completed Q4.2 of the local alpha quality closure plan.
- Added `scripts/harness/check-visual-qa-docs.py` to compare the final visual QA runner's surfaces and viewports with the wiki.
- Wired the drift check into `scripts/harness/check.sh` so runner/docs mismatch is caught without generating screenshots.

## [2026-06-22] quality | Final visual QA gate defined

- Completed Q4.1 of the local alpha quality closure plan.
- Added a path-filtered GitHub Actions workflow that runs final visual QA only for UI-affecting paths.
- Updated the PR template, contributor workflow, and UI Visual QA topic so contributors and agents know when screenshot QA is required.

## [2026-06-22] quality | Modal text fit verified

- Completed Q3.5 of the local alpha quality closure plan.
- Final visual QA now captures `M029-shortcuts` across all responsive breakpoints.
- The runner checks modal copy clipping, shortcuts dialog viewport containment, active shortcut rows, disabled remapping copy, and scroll overflow behavior for Preferences, Export, shortcuts, and import review containers.

## [2026-06-22] quality | Resize boundaries verified

- Completed Q3.4 of the local alpha quality closure plan.
- Final visual QA now captures `1023px`, `1180px`, `1279px`, `1280px`, `1440px`, and `1728px`.
- The runner checks that standard desktop widths keep sidebar labels readable and that narrow rail label hiding is limited to the narrow breakpoint.

## [2026-06-22] interaction | Focus return path verified

- Completed Q3.3 of the local alpha quality closure plan.
- Hidden import review, import panel, Loupe, and AI Review surfaces now return focus to a visible trigger or the grid when closed from inside the surface.
- UI workflow smoke now checks the focus-return markers alongside the keyboard/dismissal contracts.

## [2026-06-22] interaction | Shortcuts discovery verified

- Completed Q3.2 of the local alpha quality closure plan.
- Shortcuts remain read-only and discoverable from Welcome, Preferences > Shortcuts, and `?`.
- UI workflow smoke now checks the shortcuts discovery paths and disabled custom-remapping state.

## [2026-06-22] interaction | Escape dismiss priority verified

- Completed Q3.1 of the local alpha quality closure plan.
- `Escape` now follows the documented top-surface order through the global dismiss handler, with grid multi-selection handled as the lowest-priority dismissible state.
- UI workflow smoke now checks the dismiss order so dialogs, import review, import panel, Loupe, AI Review, and multi-select do not regress into competing `Escape` behavior.

## [2026-06-20] quality | Visual contradictions resolved

- Completed Q2.6 of the local alpha quality closure plan.
- Export Display P3 state now keeps field, summary, and safety copy coherent in visual QA.
- Advanced Preferences, Develop History, and AI Review states now stay readable without making deferred or approval-gated paths louder than the editor workflow.

## [2026-06-20] quality | Inspector rhythm unified

- Completed Q2.5 of the local alpha quality closure plan.
- Right inspector, Export summary, Preferences selector, and disabled-control rhythm now share compact desktop panel treatment.
- Batch export failures now separate file name from block reason, and ready Develop QA asserts mask support copy stays `Manual`.

## [2026-06-20] quality | Preview states made honest

- Completed Q2.4 of the local alpha quality closure plan.
- Legal visual QA JPEG fixtures now render as photo-like reference scenes instead of gradient/checker mockup blocks.
- Loupe and Develop now separate ready, unsupported, and missing-original preview states; unavailable rows do not show stale thumbnail bytes or ready histogram evidence.

## [2026-06-20] quality | Thumbnail card density reduced

- Completed Q2.3 of the local alpha quality closure plan.
- Unsupported and missing thumbnail cards now keep one text state badge instead of repeating file-type and empty rating labels.
- Supported thumbnail ratings now use a fixed-width footer slot to reduce filename/rating pressure at compact width.

## [2026-06-20] quality | Photo-first Library hierarchy restored

- Completed Q2.2 of the local alpha quality closure plan.
- Populated Library now shows the photo grid before disposable cache maintenance when photos exist.
- Loupe now hides Library header and maintenance chrome so the preview viewer is the primary surface.

## [2026-06-20] quality | UI mockup parity checklist added

- Completed Q2.1 of the local alpha quality closure plan.
- Added a UI mockup parity checklist comparing current visual QA artifacts against M003, M004, M005, M007, M008, M009, and M010.
- Routed actionable visual deltas into Q2.2 through Q2.6 without requesting new product features.

## [2026-06-20] quality | Cache clear symlink boundary verified

- Completed Q1.5 of the local alpha quality closure plan.
- Disposable cache clear now handles cache-root symlinks without following protected targets.
- Added storage coverage for symlinked cache roots and nested symlinks into originals.

## [2026-06-20] quality | Schema-invalid sidecar dry-run tightened

- Completed Q1.4 of the local alpha quality closure plan.
- Schema-invalid sidecars now produce dry-run issues only, not rebuild entries.
- Clarified that sidecar rebuild precedence applies only after sidecar schema validation passes.

## [2026-06-20] quality | Missing originals downgraded

- Completed Q1.3 of the local alpha quality closure plan.
- Missing referenced originals now block preview, histogram, Develop, and export readiness.
- Desktop grid mapping hides stale thumbnail readiness for missing source rows without mutating catalog state.

## [2026-06-20] local-alpha | Quality closure route added

- Added the Local Alpha Quality Closure Plan for data trust, photo-first UI polish, interaction QA, installed app QA, and DMG readiness.
- Routed current LLM context from ad hoc blocked-gate UI hardening to the full local alpha closure path while Task 27.2 remains externally blocked.
- Corrected the final visual QA topic to match the current 25-surface, 75-screenshot runner inventory.

## [2026-06-19] blocked-beta | UI hardening route added

- Added the blocked public beta UI hardening plan for sidebar hierarchy, responsive breakpoints, Escape dismissal, and shortcuts discoverability.
- Routed current LLM context to this blocked-gate hardening path while Task 27.2 remains externally blocked by signing/notarization prerequisites.
- Kept this work separate from public beta release-candidate claims.

## [2026-06-16] phase-19 | Mask editor visual QA completed

- Completed Task 19.5 with a compact Develop Mask panel for committed manual mask readback.
- Kept Add Mask, Subject/Sky, AI, and MLX paths disabled/unavailable in the local alpha UI.
- Added M006 mask-active static, workflow, and final visual QA coverage.
- Marked Phase 19 complete and routed the current LLM path to Task 20.1 Export Settings Model and Presets.

## [2026-06-16] phase-19 | Mask compositing preview/export completed

- Completed Task 19.4 with supported JPEG/JPG export compositing for committed manual linear, radial, and brush masks.
- Added export mask evidence without storing brush alpha bytes in export settings.
- Kept RAW-derived masked export blocked before output/artifact writes.
- Routed Phase 19 to Task 19.5 for mask editor visual QA.

## [2026-06-16] phase-19 | Brush mask storage and rasterization completed

- Completed Task 19.3 with schema-owned durable manual brush strokes in `masks[].brush`.
- Added disposable brush raster cache artifacts under `render-cache/masks/` with `mask_raster` cache records.
- Added core brush preview/commit APIs while keeping final masked export blocked until Task 19.4.
- Routed Phase 19 to Task 19.4 for preview/export compositing parity.

## [2026-06-16] phase-19 | Linear and radial manual masks completed

- Completed Task 19.2 with graph-owned manual linear/radial mask helpers and undoable core commit APIs.
- Added supported JPEG/JPG develop-preview mask application while keeping mask editor UI deferred to Task 19.5.
- Added an export guard so active manual masks block before JPEG output until Task 19.4 compositing exists.
- Routed Phase 19 to Task 19.3 for brush mask storage and rasterization.

## [2026-06-16] phase-19 | Mask schema audit completed

- Completed Task 19.1 with explicit schema-owned manual gradient mask geometry.
- Recorded manual source provenance boundaries and deferred brush durable storage to Task 19.3.
- Routed Phase 19 to Task 19.2 for linear and radial manual masks.

## [2026-06-13] phase-19 | Manual mask task cards added

- Added the Phase 19 Manual Masks brief.
- Split Phase 19 into schema audit, gradient masks, brush storage/rasterization, preview/export compositing, and visual QA task cards.
- Routed the current LLM path and master execution plan to Task 19.1.

## [2026-06-13] wiki | LLM routing split

- Added a short [Current LLM Route](llm/current-route.md) for active Phase 19 routing.
- Added [Completed LLM Context](llm/completed-context.md) so historical phase routes no longer crowd the main LLM index.
- Added [Phase 18 Summary](phases/phase-18-summary.md) and routed agents away from replaying completed Phase 18 task cards by default.

## [2026-06-13] phase-18 | Edit clipboard UI completed

- Completed Task 18.5.3 with a Develop Copy & Sync panel for graph-only edit clipboard copy, paste-to-primary, and selected-page batch sync.
- Kept subset choice explicit, Detail and Lens unsupported subsets disabled, and no-runtime/static paths honest with no writes.
- Extended static UI, workflow smoke, and final visual QA checks; Phase 18 is complete and routing now moves to Phase 19 task cards.

## [2026-06-13] phase-18 | Lens geometry mutators added

- Completed Task 18.4.1 with graph-only mutators for lens toggles, distortion, vignetting, transform, rotation, flips, crop, and crop clearing.
- Added round-trip and invalid-range tests in `silica-edit`, including normalized crop frame bounds.
- Kept preview, export, UI, storage, sidecar, original metadata writes, and lens profile database behavior out of scope and routed Phase 18 to Task 18.4.2.

## [2026-06-13] phase-18 | Detail panel UI completed

- Completed Task 18.3.3 with a disabled Develop Detail readback panel for sharpening and non-MLX noise reduction.
- Kept all Detail controls and MLX Denoise disabled while showing blocked preview/export messaging for non-neutral Detail state.
- Extended static UI, workflow smoke, and final visual QA checks for the Detail renderer/export boundary and routed Phase 18 to Task 18.4.1.

## [2026-06-13] phase-18 | Detail runtime boundary added

- Completed Task 18.3.2 with an explicit no-supported-detail-runtime boundary.
- Carried neutral Detail state through render/export/core/desktop request contracts while blocking non-neutral preview, commit, histogram, and export paths honestly.
- Kept MLX denoise, model loading, fallback effects, and active Detail UI enablement out of scope and routed Phase 18 to Task 18.3.3.

## [2026-06-13] phase-18 | Detail mutators added

- Completed Task 18.3.1 with graph-only sharpening and non-MLX noise reduction mutators.
- Added round-trip and invalid-range tests in `silica-edit` while leaving `detail.mlx_denoise` unchanged and inert.
- Kept preview, export, UI, storage, sidecar, MLX runtime, and model behavior out of scope and routed Phase 18 to Task 18.3.2.

## [2026-06-13] phase-18 | HSL panel UI completed

- Completed Task 18.2.3 with a compact Develop HSL Mixer panel for all eight schema-owned color channels.
- Wired hue, saturation, and luminance controls to the existing HSL preview and commit desktop commands while preserving non-persistent draft preview behavior.
- Extended static UI, workflow smoke, and final visual QA checks for HSL command wiring, control bounds, and seeded blue-channel state; routed Phase 18 to Task 18.3.1.

## [2026-06-13] phase-18 | HSL runtime parity added

- Completed Task 18.2.2 with HSL color mixer preview, commit, desktop command, histogram, and JPEG export parity for supported local paths.
- Kept draft HSL preview non-persistent and persisted committed HSL changes through one undoable history checkpoint.
- Recorded HSL color mixer settings in export evidence without adding broad color-correctness, RAW renderer, UI, sidecar, MLX, MCP, or plugin claims and routed Phase 18 to Task 18.2.3.

## [2026-06-13] phase-18 | HSL color mixer mutator added

- Completed Task 18.2.1 with graph-only HSL color mixer mutation for all eight schema-owned channels.
- Added unsupported channel-name rejection and hue, saturation, and luminance range validation through the existing edit graph validator.
- Kept preview, export, UI, storage, sidecar, and color-correctness behavior out of scope and routed Phase 18 to Task 18.2.2.

## [2026-06-13] phase-18 | Tone curve panel UI completed

- Completed Task 18.1.3 with a compact Develop Tone Curve panel for the supported RGB point midpoint control.
- Routed preview and commit through existing desktop/core tone curve commands and kept channel/parametric controls hidden and disabled.
- Extended static UI, workflow smoke, and final visual QA checks and routed Phase 18 to Task 18.2.1.

## [2026-06-13] phase-18 | Tone curve runtime parity added

- Completed Task 18.1.2 with point tone curve preview, commit, desktop command, and JPEG export parity.
- Kept draft preview non-persistent and persisted committed tone curves through one undoable history checkpoint.
- Blocked parametric tone curve runtime behavior instead of silently approximating unsupported state and routed Phase 18 to Task 18.1.3.

## [2026-06-13] phase-18 | Tone curve mutator added

- Completed Task 18.1.1 with a graph-only tone curve mutator.
- Added validation for non-empty curve endpoints and strictly increasing `x` values.
- Rejected parametric tone curve mutation until schema-owned parameters exist and routed Phase 18 to Task 18.1.2.

## [2026-06-13] phase-18 | Professional editing task cards added

- Added the Phase 18 Professional Editing Baseline brief.
- Split Phase 18 into atomic graph, runtime parity, UI/QA, and batch-sync task cards.
- Routed the LLM index and master execution plan to Task 18.1.1.

## [2026-06-13] phase-17 | Develop P0 visual QA completed

- Completed Task 17.5 with 36 final visual QA screenshots across 12 surfaces and three desktop widths.
- Replaced the blocked `agent-browser screenshot` path with a dependency-free Chrome DevTools Protocol runner.
- Added Develop visual checks for selected-photo state, histogram state, Before/After availability, and active basic presets; Phase 17 is now complete.

## [2026-06-13] phase-17 | Reset before-after presets added

- Completed Task 17.4 with validated P0 Basic reset and built-in preset graph helpers.
- Reset and preset application commit one undoable edit checkpoint through the existing history transaction path.
- Added static Develop Before/After view-only controls and routed Phase 17 execution to Task 17.5.

## [2026-06-13] phase-17 | Real histogram cache display added

- Completed Task 17.3 with real RGB/luminance histogram data for supported local JPEG/JPG committed Develop state.
- Stored histogram JSON only under disposable `render-cache/` and kept missing, blocked, and unsupported states explicit.
- Replaced the static fake histogram placeholder with command-backed luminance bars and routed Phase 17 execution to Task 17.4.

## [2026-06-13] phase-17 | Color presence preview commit export parity added

- Completed Task 17.2.3 with color presence preview, commit, history, and export parity for supported local JPEG/JPG paths.
- Recorded vibrance and saturation in export settings for raster and RAW-derived export routes without broad RAW or color-correctness claims.
- Added desktop command/static Develop controls and routed Phase 17 execution to Task 17.3.

## [2026-06-13] phase-17 | Tone recovery preview commit export parity added

- Completed Task 17.2.2 with tone recovery preview, commit, history, and export parity for supported local JPEG/JPG paths.
- Recorded highlights, shadows, whites, and blacks in export settings for raster and RAW-derived export routes without broad RAW or color-correctness claims.
- Added desktop command/static Develop controls and routed Phase 17 execution to Task 17.2.3.

## [2026-06-13] phase-17 | White balance preview commit export parity added

- Completed Task 17.2.1 with white-balance-family preview, commit, history, and export parity for supported local JPEG/JPG paths.
- Recorded WB export settings for raster and RAW-derived export routes without broad RAW or color-correctness claims.
- Added desktop command/static Develop controls and routed Phase 17 execution to Task 17.2.2.

## [2026-06-13] phase-17 | Color presence edit graph mutator added

- Completed Task 17.1.3 with a graph-only mutator for vibrance and saturation.
- Kept preview, export, UI, storage, sidecar, and color-correctness behavior unchanged.
- Routed Phase 17 execution to Task 17.2.1.

## [2026-06-13] phase-17 | Tone recovery edit graph mutator added

- Completed Task 17.1.2 with a graph-only mutator for highlights, shadows, whites, and blacks.
- Kept preview, export, UI, storage, and sidecar behavior unchanged.
- Routed Phase 17 execution to Task 17.1.3.

## [2026-06-13] phase-17 | White balance edit graph mutator added

- Completed Task 17.1.1 with a graph-only mutator for white balance, temperature, and tint.
- Kept schema, preview, export, UI, storage, and sidecar behavior unchanged.
- Routed Phase 17 execution to Task 17.1.2.

## [2026-06-13] phase-17 | Develop P0 task cards added

- Added the Phase 17 Develop P0 Expansion brief.
- Split Phase 17 into atomic control-family task cards for mutators, preview/commit/export parity, histogram, reset/before-after/presets, and visual QA.
- Routed the LLM index and master execution plan to Task 17.1.1.

## [2026-06-12] phase-16 | Sidecar history status added

- Completed Task 16.6 with storage/core sidecar status read APIs.
- Marked clean sidecars `catalog_newer` after edit commits, flag commits, undo, and redo without rewriting sidecar files.
- Preserved `conflict` and `sidecar_newer` states, kept `sidecar.flags` unchanged, and completed Phase 16.

## [2026-06-12] phase-16 | Append-only action log added

- Completed Task 16.5 with schema version 8 action log side-effect/evidence fields and lookup indexes.
- Added storage/core append/read APIs and Core logging for import by reference, sidecar write, JPEG export, RAW-derived JPEG export, and disposable cache clear.
- Kept plugins, MCP, MLX, permission UI, raw SQLite extension writes, and original mutation claims out of scope.

## [2026-06-12] phase-16 | Develop history panel contract added

- Completed Task 16.4 with storage/core history listing, desktop `get_photo_history`, and a Develop history panel backed only by runtime checkpoints.
- Kept static HTML history rows empty; the UI renders loading, empty, error, disabled, undo, and redo states from real command responses.
- Verified row selection uses the existing core undo/redo command path instead of direct checkpoint jumps.

## [2026-06-12] phase-16 | Undo redo commands added

- Completed Task 16.3 with storage, core, and desktop undo/redo command boundaries for edit checkpoints and culling flags.
- Added catalog schema version 7 with `edit_history.history_state` and state/sequence lookup.
- Tested transaction restore behavior, redo invalidation, desktop command responses, and export-output preservation.

## [2026-06-12] phase-16 | Edit history persistence added

- Completed Task 16.2 with catalog schema version 6 and ordered edit history checkpoint columns.
- Persisted one `silica.action` edit checkpoint per committed exposure/contrast edit, including schema-valid before/after edit graphs.
- Added tests for reopen persistence, draft no-history behavior, and migration idempotence.

## [2026-06-12] phase-16 | Action semantics contract recorded

- Completed Task 16.1 with `silica.action` payload version 1 semantics.
- Defined per-photo edit and flag checkpoint units, redo invalidation, disabled undo/redo states, and logged-only action behavior.
- Reconfirmed slider drafts create no history entries and undo never deletes exports, restores cache bytes, reverses sidecars silently, or mutates originals.

## [2026-06-12] phase-16 | Action trust design gate recorded

- Completed Task 16.0 with an Action Trust topic for undoable, redoable, logged-only, non-reversible, and blocked action classes.
- Linked action trust from catalog, data-safety, edit-graph, schema reference, and LLM routing.
- Kept the task docs-only: no code, migrations, runtime behavior, dependencies, or broad fallback systems added.

## [2026-06-12] phase-16 | Undo history action trust task cards added

- Added the Phase 16 Undo History Action Trust brief and task cards 16.0 through 16.6.
- Split the trust gate into design, semantics, edit history persistence, undo/redo commands, history panel contract, append-only action log, and sidecar sync status tasks.
- Routed the next task to 16.0 while keeping broad Develop controls, masks, MLX, plugins, MCP, and original-file mutation out of scope.

## [2026-06-12] phase-15 | RAW export manual color QA recorded

- Completed Task 15.6 with a Preview.app manual QA record for one Class A fixture-backed RAW-derived JPEG sRGB export.
- Recorded local ignored QA artifact paths, source/output/ICC hashes, macOS/display context, and known limitations.
- Marked Phase 15 complete while keeping broad RAW support and broad visual color correctness claims blocked.

## [2026-06-12] phase-15 | RAW-derived JPEG sRGB export added

- Completed Task 15.5 with a fixture-gated full-resolution RAW export source artifact path under `render-cache/raw-export-sources/`.
- Added RAW-derived JPEG sRGB export evidence for source SHA-256, output SHA-256, embedded ICC state, ICC profile SHA-256, decoder backend, input profile, working space, and original hash preservation.
- Kept viewer texture cache, preview cache, unsupported RAW classes, broad RAW/color correctness claims, and original-file mutation out of the export source-of-truth path.

## [2026-06-12] phase-15 | Exposure contrast Metal draft path added

- Completed Task 15.4 by carrying exposure/contrast draft values on viewer preview render requests.
- Added Core-side edit graph validation for Metal draft requests without catalog/history writes.
- Kept commit as the only validated edit graph persistence path and preserved preview/export separation.

## [2026-06-12] phase-15 | Metal preview display identity added

- Completed Task 15.3 by promoting decoded RAW preview artifacts into disposable native viewer texture identity.
- Preserved latest-request-wins scheduling and cleared stale texture identity when decode is blocked.
- Kept image bytes, export source-of-truth behavior, catalog writes, sidecar writes, and original-file mutation out of the native viewer bridge.

## [2026-06-12] phase-15 | RAW preview artifact path added

- Completed Task 15.2 with fixture-backed Core Image RAW preview artifact writing behind `core-image-raw-probe`.
- Added bounded JPEG sRGB preview artifact identity, stale source SHA rejection, canonical source/output overwrite rejection, and original hash preservation checks.
- Added product cache path bounding under library `previews/` with cache escape rejection, while keeping RAW class E, broad RAW support, full-resolution export, and Metal display out of scope.

## [2026-06-12] phase-15 | Decoded image handoff contract added

- Completed Task 15.1 with typed decode, render, and core handoff contracts.
- Added fixture-backed decoded image identity fields for source SHA, decoder backend, dimensions, orientation, profile state, working space, disposable cache identity, and pixel format.
- Kept image bytes, cache file generation, product RAW artifact writing, catalog writes, sidecar writes, export writes, and original-file mutation out of scope.

## [2026-06-12] phase-15 | Vertical slice evidence gate completed

- Completed Task 15.0 with [Phase 15 Vertical Slice Evidence Gate](../../checklists/PHASE_15_VERTICAL_SLICE_EVIDENCE.md).
- Rechecked local ignored RAW classes A-D through the Core Image RAW probe harness and Color Class F through the color profile probe harness.
- Allowed Phase 15 to proceed to Task 15.1 while keeping RAW class E, broad RAW support, visual color correctness, committed fixture media, and original-file mutation blocked.

## [2026-06-12] phase-15 | RAW color Metal vertical slice task cards added

- Added the Phase 15 RAW Color Metal Vertical Slice brief and task cards 15.0 through 15.6.
- Routed Phase 15 through the master execution plan split: evidence gate, decoded image handoff, RAW preview artifact, Metal preview display, exposure/contrast draft path, RAW-derived JPEG sRGB export, and manual color QA.
- Kept broad RAW support, broad color correctness, product texture allocation beyond scoped tasks, MLX/MCP/plugin scope, and original-file mutation out of Phase 15 planning.

## [2026-06-12] phase-14 | Viewer QA harness and checklist added

- Completed Task 14.8 with `checklists/NATIVE_VIEWER_QA.md` and `scripts/harness/check-native-viewer-qa.py`.
- Added default harness routing that checks native viewer QA documentation and static reserved-layout evidence without requiring feature-gated native viewer execution in default CI.
- Recorded feature-gated macOS commands, manual proof fields, and viewport targets `1280x800`, `1440x900`, and `1728x965`.

## [2026-06-12] phase-14 | Disposable texture lifecycle boundary added

- Completed Task 14.7 with `silica-render` disposable texture identity, drawable size, lifecycle state, and release reason contracts.
- Added a feature-gated desktop native viewer texture boundary for photo change, drawable resize, library close, and app close cleanup.
- Kept real product texture allocation, image pixel upload, persistent GPU cache state, catalog writes, sidecar writes, and original-file mutation out of scope.

## [2026-06-12] phase-14 | Render request boundary added

- Completed Task 14.6 with typed `silica-render` viewer preview requests, viewport, future texture identity, and latest-request-wins scheduler behavior.
- Added a feature-gated desktop native viewer render bridge that schedules requests without catalog writes and records neutral evidence.
- Kept RAW decoding, color transform output, shader passes, image pixel upload, and product texture allocation out of scope.

## [2026-06-12] phase-14 | Native viewer input ownership proof added

- Completed Task 14.5 with feature-gated input ownership proof state for mouse down, drag, scroll, magnify, native-owned viewer samples, and web-owned outside samples.
- Added a neutral macOS input smoke evidence path and `checklists/NATIVE_VIEWER_INPUT_QA.md`.
- Kept telemetry, analytics, persistent input logging, product image rendering, RAW pixels, and texture cache behavior out of scope.

## [2026-06-12] phase-14 | Native viewer lifecycle proof added

- Completed Task 14.4 with feature-gated product module lifecycle proof state for reserved host geometry, drawable size, backing scale, install/uninstall lifecycle, render timing, and cleanup reason.
- Added a neutral macOS smoke evidence path and `checklists/NATIVE_VIEWER_LIFECYCLE_QA.md`.
- Kept final product image viewer installation, product texture allocation, RAW pixels, shader passes, and default feature-off app behavior out of scope.

## [2026-06-12] phase-14 | Reserved viewer layout handshake added

- Completed Task 14.3 with reserved native viewer host markers for Loupe and Develop surfaces plus an inert feature-off geometry reporting API.
- Extended the static UI harness to require exact reserved hosts, external controls, and native viewer geometry functions.
- Kept AppKit view installation, Metal rendering, texture cache, shader passes, RAW pixels, and native input ownership out of scope.

## [2026-06-12] phase-14 | Native viewer feature gate added

- Completed Task 14.2 with a non-default `native-metal-viewer` feature and macOS-only `native_metal_viewer` product module shell.
- Added a module contract test proving the product shell is separate from `metal_host_spike.rs` and not compiled into the default app path.
- Kept RAW pixels, shader passes, texture cache, AppKit view installation, and UI layout changes out of scope.

## [2026-06-12] phase-14 | AppKit Metal viewer bridge contract added

- Completed Task 14.1 with the product bridge contract for reserved viewer layout, AppKit/Metal lifecycle, event ownership, render request boundaries, disposable texture boundaries, and Path B stop gates.
- Preserved the rule that `metal_host_spike.rs` remains proof code and `native-metal-viewer` product work must be feature-gated.
- Narrowed the open architecture question for Task 14.8 physical mouse and trackpad checklist evidence.

## [2026-06-12] roadmap | Post-alpha master execution plan added

- Added a Phase 14 through v1.0 master execution plan so maintainers and agents do not recreate phase-wide plans before each phase.
- Recorded wave order, dependency graph, stop gates, validation matrix, and future task splits for Phase 15, Phase 16, Phase 17, Phase 20, Phase 22, Phase 23, Phase 27, and Phase 28.
- Linked the master plan from the wiki index, LLM routing index, roadmap overview, post-alpha roadmap, Phase 14 plan, Phase 14 brief, task index, wiki README, and root README.

## [2026-06-12] phase-14 | Product Metal viewer bridge plan added

- Added the Phase 14 Product Metal Viewer Bridge plan, phase brief, and task cards 14.0 through 14.8.
- Split Path B product viewer work into atomic gates for bridge contract, feature-gated module shell, reserved layout, lifecycle, input, render request, texture lifecycle, and QA.
- Kept RAW pixels, exposure/contrast Metal rendering, shader breadth, and color correctness out of Phase 14.

## [2026-06-12] phase-13 | Explicit export color options added

- Completed Task 13.8 with sRGB as the default JPEG export path and Display P3 as an explicit ICC-backed option.
- Added desktop command parsing that rejects unsupported export color profile strings instead of silently falling through.
- Marked Phase 13 implementation complete while keeping visual color correctness blocked pending tolerance results and manual review.

## [2026-06-12] phase-13 | Color metadata contract added

- Completed Task 13.7 with schema-owned edit graph profile metadata helpers and validation.
- Export records now preserve output SHA-256, ICC embedded state, ICC profile SHA-256, and profile metadata source in existing `export_settings_json`.
- Kept unknown profile data explicit and avoided new hidden schema fields.

## [2026-06-12] phase-13 | ICC export proof added

- Completed Task 13.6 with JPEG ICC embedding and inspection for default sRGB export and explicit Display P3 export API proof.
- Recorded output SHA-256, ICC profile SHA-256, embedded ICC state, output profile, separate output path behavior, and original-preservation coverage.
- Added the manual Preview.app or Photos QA checklist while keeping color correctness and user-facing Display P3 export blocked.

## [2026-06-12] phase-13 | Color support matrix added

- Completed Task 13.5 with a fixture-backed color probe support matrix.
- Recorded sRGB, Display P3, and untagged JPEG probe status, profile state, working/output profile, transform path, original hash state, and evidence source.
- Kept color correctness, export ICC embedding, Display P3 export options, and committed fixture media blocked pending later proof tasks.

## [2026-06-12] phase-13 | Color probe harness added

- Completed Task 13.4 with a manifest-driven local color probe harness.
- The harness validates Class F fixture safety, runs the feature-gated `silica-render` probe, and records structured source hash, profile, transform-path, file size, modified-time, status, and original-preservation evidence.
- No ColorSync transform, export ICC embedding, rendered pixels, or color correctness claim was added.

## [2026-06-12] phase-13 | Color profile probe added

- Completed Task 13.3 with a non-default `silica-render` `color-probe` feature.
- The probe records source SHA-256, embedded ICC marker state, input profile classification, working space, output profile, transform path, and failure category.
- No ColorSync transform, ICC export embedding, rendered pixels, or color correctness claim was added.

## [2026-06-12] phase-13 | Local color fixture manifest added

- Completed Task 13.2 locally with ignored synthetic Color Class F fixtures and `.tmp/legal-color-fixtures/color-fixtures.json`.
- Recorded hashes for sRGB, Display P3, and untagged JPEG fixtures.
- Kept fixture media and manifest out of git; profile probing remains Task 13.3 and Task 13.4.

## [2026-06-12] phase-13 | Color fixture source review added

- Completed Task 13.1 source review for local-only synthetic Color Class F fixtures.
- Accepted generated local fixtures for sRGB, Display P3, and untagged JPEG subclasses.
- Kept fixture media and Apple system-profile-derived outputs blocked from git until redistribution permission is reviewed.

## [2026-06-12] phase-13 | Color proof plan added

- Added Phase 13 color pipeline proof plan, phase brief, and task cards 13.0 through 13.8.
- Routed current LLM work from completed Phase 12 to Phase 13.
- Kept color correctness, Display P3 export claims, RAW color behavior, and new dependencies behind explicit evidence gates.

## [2026-06-12] phase-12 | RAW proof phase completed

- Phase 12 completion gate is satisfied for legal fixture classes A-D.
- Class E remains blocked pending source review, and LibRaw remains deferred because no fixture-backed Core Image gap was recorded.
- Next product roadmap area is Phase 13 color pipeline proof.

## [2026-06-12] phase-12 | Product RAW support mapping added

- Completed Task 12.6 with an evidence-driven product RAW support mapping API.
- The path-only RAW plan remains conservative; only successful A-D Core Image probe evidence can return metadata-only `Supported`.
- No RAW pixels, export expansion, cache generation, broad support claim, original mutation, or LibRaw dependency was added.

## [2026-06-12] phase-12 | RAW fixture probe evidence recorded

- Completed local Task 12.5.2 through Task 12.5.4 for raw.pixls.us candidates A-D.
- The ignored local manifest probe succeeded on macOS 26.4, and original hashes remained unchanged.
- Classes A-D are now fixture-backed Core Image-supported in the RAW support matrix; class E remains pending source review.

## [2026-06-12] phase-12 | RAW fixture source review added

- Completed Task 12.5.1 source review for raw.pixls.us candidates.
- Accepted local-only CC0 candidates for fixture classes A-D.
- Kept fixture class E pending and kept all RAW media out of git.

## [2026-06-12] phase-12 | RAW proof plan page added

- Added a public wiki Phase 12 RAW proof plan page.
- Added Task 12.5 as the legal RAW fixture evidence gate before any product RAW support claim.
- Linked the plan from the wiki index, Phase 12 brief, LLM routing index, task cards, and post-alpha roadmap.

## [2026-06-12] phase-12 | Product RAW decode contract added

- Completed Task 12.4 product RAW decode API contract.
- RAW candidates now return an explicit blocked-pending-evidence plan, and non-RAW candidates return an unsupported-class plan.
- No RAW pixels, UI display, export path, cache generation, or color correctness claim was added.

## [2026-06-12] phase-12 | Core Image support matrix recorded

- Completed Task 12.3 with every RAW fixture class marked `blocked_pending_evidence`.
- No legal RAW fixture manifest is available, so no fixture class graduated to Core Image-supported.
- LibRaw remains deferred because no fixture-backed Core Image gap has been recorded.

## [2026-06-12] phase-12 | RAW fixture probe harness added

- Added Task 12.2.1 RAW fixture probe report types, manifest loader, ignored fixture test, and manual harness command.
- The harness rejects unsafe or incomplete manifest entries before probing.
- Running the ignored fixture probe remains blocked until `SILICARAW_RAW_FIXTURE_MANIFEST` points to a legal local RAW fixture manifest.

## [2026-06-12] phase-12 | Core Image RAW probe backend added

- Completed Task 12.1.3 macOS Core Image probe backend.
- The non-default probe records source metadata, SHA-256, explicit errors, and Core Image dimensions when available.
- Product RAW pixels, fixture-backed support claims, LibRaw, and UI RAW display remain out of scope.

## [2026-06-12] wiki | Phase 12 design gate task card added

- Added Task 12.0 as a wiki task card so Phase 12 routing starts at the design gate.
- Linked the Phase 12 brief and task index to the new card.
- Kept the full implementation details in the existing design and plan docs.

## [2026-06-12] phase-12 | RAW probe contract added

- Completed Task 12.1.2 probe type contract.
- `silica-decode` now exposes proof-only RAW probe request/result/status/error types and an unsupported fallback route.
- Existing preview readiness remains unchanged; product RAW pixels are still out of scope.

## [2026-06-12] phase-12 | Core Image RAW probe feature gated

- Completed Task 12.1.1 Core Image dependency and feature gate.
- Added the non-default `core-image-raw-probe` feature to `silica-decode`.
- Documented direct Core Image binding and SHA-256 dependencies without adding product RAW pixels or LibRaw.

## [2026-06-11] phase-12 | RAW proof plan added

- Added the Phase 12 RAW proof design gate and implementation plan.
- The plan keeps Core Image probing feature-gated, fixture-backed, and separate from product RAW pixels.
- The Phase 12 brief now links to the design, plan, and task cards before implementation starts.

## [2026-06-11] wiki | LLM routing added

- Added the LLM routing index, completed Phase 11 summary, Phase 12 brief, and Phase 12 task cards.
- The default agent route now points to small phase/task pages before large roadmap or design-spec files.
- Wiki conventions now describe routing pages and task cards as token-saving navigation aids.

## [2026-06-11] phase-11 | Connected runtime smoke extended

- Completed Task 11.9.5 connected runtime smoke for Phase 11.
- The smoke now covers recents, relaunch restore, missing-library fallback, paged grid, stored metadata, recursive import review issues, and original-file safety.
- `check-connected-runtime-smoke.py` now requires a Phase 11 completion marker from the exact desktop smoke test.

## [2026-06-11] phase-11 | Opt-in recursive import added

- Added Task 11.9.4 opt-in recursive import.
- Recursive scanning is disabled by default and only runs when `FolderImportOptions.recursive` or the desktop `Include subfolders` checkbox is selected.
- Recursive issues reuse the structured import issue model and skip symlinks instead of following them.

## [2026-06-11] phase-11 | Import issue review UI added

- Added Task 11.9.3 import issue review UI.
- The import panel now renders structured unsupported, skipped, and failed import issues from the latest import response.
- UI workflow smoke and final visual QA cover the import issue review surface without enabling recursive import.

## [2026-06-11] phase-11 | Structured import issues added

- Added Task 11.9.2 structured import issue model.
- `FolderImportSummary.issues` now returns recoverable `ImportIssue` records for the non-recursive import path.
- The desktop import command forwards the issue list in its response data for Task 11.9.3.
- Unsupported files, hidden entries, package directories, symlinks, and entry metadata/read failures are reviewable without blocking library browsing.

## [2026-06-11] phase-11 | Import error policy added

- Added Task 11.9.1 import error and recursive import policy.
- Recursive import remains explicit and defaults off.
- Recoverable issues, unsupported files, symlinks, hidden entries, packages, max depth, and permission errors now have documented review behavior.

## [2026-06-11] phase-11 | Metadata-backed filter added

- Added Task 11.8.2 `has_dimensions` grid filter backed by stored metadata.
- Catalog schema version 5 adds the dimension filter index.
- Camera/lens metadata filters remain disabled until parser and indexed query support exist.

## [2026-06-11] phase-11 | Metadata inspector UI added

- Added Task 11.8.1 Library and Loupe metadata inspector UI.
- The inspector now uses the `get_photo_metadata` command and shows unavailable values honestly when metadata is missing.
- Multi-selection keeps metadata primary-photo-only instead of inventing aggregate metadata.

## [2026-06-11] phase-11 | Metadata query API added

- Added Task 11.7.4 typed metadata query APIs across storage, core, and desktop.
- Metadata fields now serialize explicit `known`, `unknown`, or `unavailable` states for inspector use.
- Query APIs read only catalog state; tests remove originals before query to guard against display-time source-file reads.

## [2026-06-11] phase-11 | Metadata migration and extraction added

- Added Task 11.7.3 metadata migration and import-time extraction.
- Catalog schema version 4 adds nullable width, height, and orientation fields under `photo_metadata`.
- JPEG/JPG imports persist width and height when readable; RAW missing metadata stays unavailable, unsupported files do not get fake metadata rows, and originals remain unchanged.

## [2026-06-11] phase-11 | Metadata backfill policy added

- Added Task 11.7.2 metadata backfill and JPEG-only extraction policy.
- Library open/session restore do not run metadata backfill; existing unknown metadata stays unknown until explicit import or backfill work.
- JPEG/JPG dimensions may use the existing raster path; RAW metadata does not imply RAW decode support.

## [2026-06-11] phase-11 | Metadata schema gate recorded

- Added Task 11.7.1 metadata field and dependency gate.
- No EXIF parser dependency is added yet; camera/lens/orientation/capture metadata remains explicitly unavailable.
- Backfill policy was later completed by Task 11.7.2; migration and import-time JPEG dimension extraction were later completed by Task 11.7.3.

## [2026-06-11] phase-11 | Multi-select grid semantics added

- Added Task 11.6.4 current-page multi-selection semantics.
- The product grid now distinguishes primary selection from multi-selection, supports range/toggle selection, and shows inspector selection counts.
- Batch edit behavior remains out of scope.

## [2026-06-11] phase-11 | Keyboard grid navigation added

- Added Task 11.6.3 current-page roving-focus keyboard navigation.
- Arrow keys, Home, End, PageUp, PageDown, and Enter-to-loupe are wired through the product grid.
- Multi-select was later completed by Task 11.6.4.

## [2026-06-11] phase-11 | Virtualized grid window added

- Added Task 11.6.2 page-local virtualized grid rendering.
- The product grid now renders visible rows plus overscan spacer rows and cleans up grid-owned thumbnail object URLs when rows leave the window.
- Keyboard navigation was later completed by Task 11.6.3; multi-select remains Task 11.6.4.

## [2026-06-11] phase-11 | Page-driven grid UI added

- Added Task 11.6.1 page-driven grid UI states.
- The product grid now shows page metadata, previous/next controls, loading, empty, and error states from `query_library_photos`.
- Virtualized windowing was later completed by Task 11.6.2; keyboard navigation and multi-select remain Task 11.6.3 and Task 11.6.4.

## [2026-06-11] phase-11 | Page-scoped thumbnail hydration added

- Added Task 11.5.5 page-scoped thumbnail hydration for paged grid queries.
- The product grid now calls `query_library_photos` and the UI smoke harness checks the page-scoped thumbnail request marker.
- Full page UI states and pagination controls were later completed by Task 11.6.1.

## [2026-06-11] phase-11 | Desktop paged grid command added

- Added Task 11.5.4 `query_library_photos` desktop command for page-based grid queries.
- The command accepts typed page/sort/filter fields and returns `photoGridPage` metadata.
- At the time, page-scoped thumbnail hydration remained Task 11.5.5.

## [2026-06-11] phase-11 | Storage/core paged query added

- Added Task 11.5.3 read-only paged library query APIs in `silica-storage` and `silica-core`.
- Query responses include bounded rows, total-count metadata, deterministic order fields, and deterministic empty pages.
- At the time, desktop command wiring remained Task 11.5.4.

## [2026-06-11] phase-11 | Paged query indexes added

- Added Task 11.5.2 catalog schema version 3 for paged library queries.
- Storage migration 3 adds normalized `photos.file_type` values and query indexes for accepted sort/filter fields.
- At the time, storage/core paged query execution remained Task 11.5.3.

## [2026-06-11] phase-11 | Paged query contract added

- Added Task 11.5.1 typed `silica-catalog` request and page contracts for bounded offset library queries.
- Whitelisted sort/filter enums and deterministic tie breakers now define what storage may implement.
- At the time, query indexes and storage/core execution remained Task 11.5.2 and Task 11.5.3.

## [2026-06-11] phase-11 | Layout visual QA states added

- Added Task 11.4.3 visual QA states for sidebar collapsed, inspector collapsed, and layout reset.
- The final visual QA runner now captures 11 surfaces across `1280x800`, `1440x900`, and `1728x965`.
- Latest run produced 33 screenshots with no horizontal overflow, toolbar overlap, clipped controls, or layout-state assertion failures.

## [2026-06-11] phase-11 | Layout interactions persisted

- Added Task 11.4.2 desktop commands for recording and resetting app-session layout preferences.
- Wired sidebar, inspector, filmstrip, thumbnail size, sort, and filter controls to app-session layout state without adding catalog query filtering ahead of Task 11.5.
- Dedicated layout screenshot states remain Task 11.4.3.

## [2026-06-11] phase-11 | Layout preference model documented

- Added Task 11.4.1 core helpers for layout defaults and layout reset.
- Documented sidebar, inspector, filmstrip, thumbnail size, sort, and filter defaults plus invalid-value behavior in the UI MVP baseline.
- Desktop layout persistence wiring and responsive QA remain Task 11.4.2 and Task 11.4.3.

## [2026-06-11] phase-11 | Selected photo restore added

- Added Task 11.3.2 selected-photo restore from app-session state.
- User-driven selection and mode changes now record to app-session JSON, and launch restore validates the saved photo id through a read-only catalog probe before applying it.
- Missing selected photos clear selection and resolve mode back to Library without crashing or opening write-side workflows.

## [2026-06-11] phase-11 | Launch restore resolver added

- Added Task 11.3.1 launch restore resolution from app-session state.
- The resolver validates the last library and catalog without opening the normal migrate/repair path, and the UI applies Welcome vs Library state on boot without calling `open_library`.
- Selected-photo restore and true Develop/Export mode restore remain Task 11.3.2.

## [2026-06-11] phase-11 | Welcome recent libraries connected

- Added Task 11.2.2 Welcome recents rendering from real app-session data.
- First launch stays empty, unavailable recent paths are disabled and labeled, and valid recent libraries open through the existing desktop command path.
- Relaunch restore, selected-photo restore, and layout preference persistence remain separate Phase 11 tasks.

## [2026-06-11] phase-11 | Real recent recording added

- Added Task 11.2.1 app-session recent recording after successful library create/open.
- Recent entries dedupe, cap at the documented limit, update last-library state, and remain outside library catalogs and sidecars.
- Welcome recents rendering and unavailable-path UI remain Task 11.2.2.

## [2026-06-11] phase-11 | Desktop app session commands added

- Added Task 11.1.3 desktop app-session command boundary.
- Desktop resolves `app-session.json` under the Tauri app config directory and exposes read, write, reset, and inspect handlers backed by `silica-core`.
- Real recent recording, Welcome recents, and relaunch restore remain separate Phase 11 tasks.

## [2026-06-11] phase-11 | App session core types added

- Added Task 11.1.2 core app-session v1 types and JSON read/write helpers.
- Kept app session state outside `catalog.db`, sidecars, and frontend-only storage; desktop path commands, recents, and restore behavior remain next tasks.
- Verified the targeted `silica-core` app-session tests and full `silica-core` crate tests before moving on.

## [2026-06-11] phase-11 | Plan tightening after agent re-audit

- Re-audited the Phase 11 plan with architecture, storage, preview/export, and release/harness agents.
- Tightened the existing plan without changing the product direction: bounded offset pagination, page-scoped thumbnail hydration, metadata backfill policy, and import-error review before recursive scanning.
- Reaffirmed lean validation: task-specific checks during development and `scripts/harness/check.sh` as the PR completion gate, without broad fallback systems or large test matrices.

## [2026-06-11] phase-11 | Session, library, and metadata design added

- Added the Phase 11 design gate after consulting architecture, storage, frontend, and test agents.
- Split Phase 11 into atomic app-session, recents, relaunch restore, layout preference, paged query, virtual grid, metadata, and recursive import tasks.
- Recorded stop gates for app-session storage boundaries, query safety, truthful metadata, recursive import, and original-file preservation.

## [2026-06-11] phase-10 | Public trust package completed

- Added Task 10.6.2 contribution and security docs with public issue templates and PR public-trust checks.
- Added a static public trust package harness check for required files, README limitation language, security disclosure boundaries, and roadmap completion status.
- Completed Phase 10 evidence, recovery, and public OSS trust gates.

## [2026-06-11] phase-10 | Public trust docs started

- Added Task 10.6.1 public trust docs with the MIT project license, root `LICENSE`, and ADR 0008.
- Recorded allowed and forbidden public claims for the current local alpha, including unsigned developer-preview limits and deferred RAW/color/Metal/MLX/MCP/plugin distribution claims.
- Left contribution/security templates and static trust regression checks for Task 10.6.2.

## [2026-06-11] phase-10 | Restore boundary implementation added

- Added Task 10.5.3 staged restore behavior in `silica-storage`.
- Restore validates backup manifests, rejects newer catalog schema backups before target mutation, restores through staging, and creates rollback copies before replacing existing target catalog and sidecars.
- Verified restored edit states, flags, sidecar status, export records, migration metadata, and original-file preservation; user-facing restore UI and conflict UI remain later work.

## [2026-06-11] phase-10 | Backup boundary implementation added

- Added Task 10.5.2 checkpointed backup artifact creation in `silica-storage`.
- Backup artifacts include `catalog.db`, `sidecars/`, and `backup-manifest.json` under library `backups/` while excluding originals, disposable caches, export outputs, logs, and nested backups.
- Verified latest WAL state is copied through checkpoint-before-copy behavior; restore execution and rollback behavior remain pending.

## [2026-06-11] phase-10 | Backup and restore policy added

- Added the Task 10.5.1 backup/WAL/checkpoint/restore policy topic.
- Recorded checkpoint-before-copy backup behavior, durable and disposable recovery boundaries, restore target rules, and migration failure handling.
- Added a recovery-policy harness guard while keeping backup archive creation, restore execution, conflict UI, and original-file mutation out of scope.

## [2026-06-11] phase-10 | Sidecar rebuild dry-run added

- Added Task 10.4 catalog rebuild dry-run reports from library-local sidecars.
- Resolved portable flags by `sidecar.flags`, then `edit_graph.metadata`, then defaults while reporting malformed/schema-invalid sidecars, photo-id mismatches, flag/metadata disagreements, and catalog reconciliation conflicts.
- Kept applied restore, catalog overwrite, conflict UI, broad rescanning, backup archives, RAW/color proof, and export proof out of scope.

## [2026-06-11] phase-10 | Sidecar v1 foundation added

- Added Task 10.3 sidecar v1 read/write foundation for library-local sidecars.
- Validated sidecar and nested edit graph payloads while keeping `photo_flags` catalog-authoritative during normal app operation.
- Preserved original-file safety and kept automatic sync, rebuild, backup/restore, conflict UI, RAW/color proof, and export proof out of scope.

## [2026-06-11] phase-10 | Evidence and recovery design added

- Added the Phase 10 evidence and recovery design gate for Task 10.3 through Task 10.6.
- Defined sidecar, rebuild dry-run, backup/restore, and public OSS trust boundaries before implementation.
- Recorded RAW, color, export, auto-sync, next-to-original sidecar, Homebrew, auto-update, MLX, plugin, and MCP exclusions.

## [2026-06-11] phase-10 | Golden image tolerance policy added

- Added the Task 10.2 golden image and tolerance policy baseline.
- Separated byte equality, file/profile inspection, pixel or perceptual tolerance, and manual visual review gates.
- Recorded that RAW support and color correctness claims remain blocked until fixture-backed evidence exists.

## [2026-06-11] phase-10 | Fixture manifest contract added

- Added the RAW/color fixture manifest schema and example for Task 10.1.
- Added a deterministic harness check for fixture provenance, license, path, hash, RAW gate, and Color Class F expectations.
- Recorded that manifests are provenance and expectation metadata, not RAW support or color correctness proof.

## [2026-06-11] phase-9 | Local DMG release runbook added

- Added a maintainer runbook for signed local alpha DMG releases and current unsigned developer-preview artifacts.
- Included prerelease checks, tag naming, rollback steps, Gatekeeper assessment commands, and notarization troubleshooting links.
- Updated README distribution notes so contributors can find the current release and preview paths from the repository entry point.

## [2026-06-11] phase-9 | Release notes template added

- Added `.github/release-template.md` for local alpha DMG release notes.
- Included install steps, checksum verification, known issues, privacy, QA evidence, rollback, and unsigned developer-preview boundary language.
- Added a harness check so release notes do not lose required local-distribution safety fields.

## [2026-06-11] phase-9 | Homebrew and auto-update deferral recorded

- Added ADR 0007 to defer Homebrew Cask and auto-update until local DMG alpha trust gates are met.
- Corrected the Phase 9.3 planned ADR filename to avoid the existing ADR 0005 MLX deferral.
- Linked the decision from release docs so future agents do not add updater or Homebrew behavior during local alpha.

## [2026-06-11] phase-8 | Developer-preview artifact runbook added

- Added a maintainer runbook for triggering, watching, downloading, and checksum-verifying unsigned developer-preview DMG artifacts.
- Documented manual workflow dispatch and `developer-preview-*` tag workflows without treating the artifact as user-ready local distribution.
- Extended release workflow guardrails so the runbook and unsigned/notarized boundary stay present.

## [2026-06-11] phase-8 | Unsigned developer-preview workflow added

- Added ADR 0006 to permit unsigned developer-preview DMG artifacts while Developer ID funding is blocked.
- Added a manual/tag-triggered GitHub Actions workflow that builds unsigned macOS DMG artifacts, writes SHA256 checksums, and uploads an unsigned warning note.
- Added release workflow guardrails to keep unsigned preview artifacts separate from the future signed/notarized user-ready release path.

## [2026-06-11] phase-7 | Signing prerequisite audit added

- Checked local code signing identities and GitHub repository secret names for Phase 7.1.
- Recorded that signing/notarization is blocked: the local keychain has an Apple Development identity, no Developer ID Application identity, and no required GitHub signing secrets.
- Added a signing/notarization prep checklist and repeatable preflight script that records secret names only, never secret values.

## [2026-06-11] phase-6 | Local build-Mac DMG smoke recorded

- Built a developer unsigned DMG locally and verified its SHA256, mount behavior, mounted app presence, installed app hash match, installed-app preflight, and GUI launch from `/Applications`.
- Added a repeatable local DMG artifact smoke harness for build-machine artifact checks.
- Recorded that clean-Mac Task 6.3 remains pending because the local smoke ran on the Mac that built the app.

## [2026-06-10] roadmap | Post-alpha product roadmap added

- Added the Post-Alpha Product Roadmap after consulting separate RAW/Metal/color, Library/Develop/UI, and MLX/plugin/MCP planning agents.
- Split post-alpha work into atomic phases for evidence and trust gates, session/library/metadata, Core Image RAW proof, color proof, product Metal viewer, RAW/color/Metal vertical slice, undo/history, Develop expansion, masks, export, preferences, hardening, permissions, MLX, plugins, MCP, public beta, and v1.0.
- Linked the new roadmap from the wiki index, roadmap overview, and local DMG distribution plan so agents can continue after Phase 9 without expanding local-alpha scope.

## [2026-06-10] phase-5.6 | Final visual QA refreshed

- Added a final visual/responsive QA runner for M001, M002, M003, M004, M005, M007, M008-minimal, and M009 surfaces across compact, desktop, and large viewports.
- Fixed final QA findings in Loupe/Develop image scaling, Export preview pixels, Import step progress state, and cache-maintenance status copy.
- Recorded that the Phase 5.6.12 QA notes supersede the static-only Phase 5.5 visual pass for Phase 6 readiness.

## [2026-06-10] phase-5.6 | Connected runtime smoke added

- Added a developer runtime smoke that generates legal fixtures and exercises the desktop command workflow end to end.
- Covered create/open, import, grid, culling, loupe, Develop edit, JPEG sRGB export, cache clear, reopen, and original byte comparisons.
- Documented that this is not clean-Mac DMG install QA, which remains a Phase 6 gate.

## [2026-06-10] phase-5.6 | Legal fixtures and preflight added

- Added a legal synthetic fixture generator for supported JPEG/JPG samples, unsupported files, and optional RAW-blocked placeholders.
- Added an installed-app developer preflight report that records app artifact hash, host/macOS field, fixture path, fixture hash results, and known local-alpha limitations.
- Wired the fixture/preflight check into the project harness and documented the manual preflight checklist.

## [2026-06-09] phase-5.6 | Product alpha runtime completion planned

- Added the Product Alpha Runtime Completion topic.
- Inserted Phase 5.6 before clean-Mac install QA in the local DMG distribution plan.
- Recorded that Phase 5.5 is a screen/command-wiring milestone, while Phase 5.6 is the installed-app readiness pass for real JPEG/JPG pixels, native/selectable paths, persisted edit-state UI readback, cache clear, legal fixtures, and runtime smoke.
- Narrowed the guaranteed installed-alpha visible photo path to JPEG/JPG until additional raster codecs are explicitly implemented and tested.

## [2026-06-09] phase-5.5 | UI MVP baseline started

- Added Task 5.5 as a UI MVP vertical slice before local install QA.
- Added the UI MVP baseline topic with source hierarchy, mockup mapping, and `ui-ux-pro-max` accepted/rejected guidance.
- Started static frontend tokenization with `apps/desktop/static/styles/tokens.css` and `base.css`.

## [2026-06-09] phase-5 | JPEG sRGB export added

- Added command/API-level JPEG sRGB export for already-rendered raster sources.
- Added catalog export record persistence and exported flag updates.
- Added a minimal Tauri export command.
- Kept RAW decoding, Metal rendering, UI export screens, broad color fixture validation, MLX, MCP, plugin behavior, and distribution changes out of scope.

## [2026-06-09] phase-5 | Exposure and contrast edit flow added

- Added default edit graph construction and validated exposure/contrast updates to `silica-edit`.
- Added draft exposure/contrast preview request planning to `silica-render`.
- Added active edit graph read/commit persistence in `silica-storage`.
- Added `silica-core` and minimal Tauri command APIs that keep slider-preview updates out of SQLite and persist only on commit/release.
- Kept product M005 UI screens, pixel rendering, sidecar writing, RAW decoding, Metal viewer work, MLX, MCP, and plugin behavior out of scope.

## [2026-06-09] phase-5 | Edit graph types and validation added

- Added typed Rust edit graph structures to `silica-edit`.
- Added schema-aware validation for schema/version constants, enum shape, closed objects, numeric ranges, mask adjustment numbers, and `mlx_denoise` object/null shape.
- Verified round-trip serialization against `schemas/edit_graph.example.json`.
- Kept edit application, render integration, sidecar persistence, UI, RAW decoding, Metal viewer work, MLX, MCP, and plugin behavior out of scope.

## [2026-06-08] phase-5 | Preview readiness path added

- Added preview decode readiness routing to `silica-decode`.
- Added render-side preview readiness planning to `silica-render`.
- Added catalog preview candidate lookup, core preview session API, and a minimal Tauri preview status command.
- Recorded that `MockupUI/` contains the target UI screens, while M004/M005 implementation remains a later explicit UI task.

## [2026-06-08] phase-4 | Photo flag persistence added

- Added the local alpha photo flags contract to `silica-catalog`.
- Added SQLite `photo_flags` default row creation, read APIs, and write APIs to `silica-storage`.
- Added core and minimal Tauri command boundaries for rating, pick, reject, and color label persistence.
- Verified flags survive local library reopen without sidecar or UI implementation.

## [2026-06-08] phase-4 | Folder import scanner added

- Added the local alpha import candidate contract and supported extension list to `silica-catalog`.
- Added non-recursive folder import scanning to `silica-storage`.
- Verified mixed supported/unsupported fixture import by reference without copying or mutating originals.

## [2026-06-08] phase-4 | Library create/open path added

- Added local library folder create/open helpers to `silica-storage`.
- Added create/open APIs to `silica-core` and minimal Tauri commands in the desktop shell.
- Updated the Catalog wiki topic and local DMG roadmap to mark Task 4.2 complete.

## [2026-06-08] phase-4 | Catalog migration foundation completed

- Added the local alpha catalog schema contract to `silica-catalog`.
- Aligned `silica-storage` migration verification with the catalog contract for required tables, required indexes, schema version, and migration bookkeeping.
- Added the Catalog wiki topic and marked Task 4.1 complete in the local DMG distribution plan.

## [2026-06-08] phase-3 | ADR 0005 deferred MLX from local alpha

- Added ADR 0005 for MLX local-alpha deferral.
- Recorded that `silica-mlx` remains a dependency-free boundary crate.
- Corrected the Phase 3.5 roadmap location to ADR 0005 because ADR 0004 already exists.

## [2026-06-08] phase-3 | Spike 004 recorded SQLite persistence path

- Added the SQLite catalog persistence spike report.
- Selected `rusqlite` with bundled SQLite and embedded SQL migrations.
- Added initial catalog schema and required index migration tests to `silica-storage`.

## [2026-06-08] phase-3 | Spike 003 recorded color-management path

- Added the color-managed preview/export spike report.
- Recorded Core Image/ColorSync-compatible color management first, linear Display P3 working-space recommendation, sRGB default export, and Display P3 export support.
- Recorded that tagged raster and RAW color fixtures are still missing, so color correctness remains unproven.

## [2026-06-08] phase-3 | Spike 002 recorded RAW decoder path

- Added the RAW decoder path spike report.
- Recorded Core Image RAW primary with LibRaw deferred until legal fixtures prove a support gap.
- Added `silica-decode` gate metadata without adding decoder dependencies or RAW decoding behavior.

## [2026-06-08] phase-3 | Spike 001 recorded Path B

- Added the Tauri + native Metal viewer spike report.
- Recorded Path B: Tauri can host a native Metal view, but the product viewer needs a dedicated AppKit/Metal bridge.
- Updated Metal rendering, architecture risk, and open question pages to point at the spike result.
- Added optional macOS native bridge dependencies to the dependency policy.

## [2026-06-08] phase-2 | Desktop shell skeleton started

- Replaced the desktop placeholder boundary with a minimal Tauri v2 shell under `apps/desktop/src-tauri`.
- Added local static shell assets without a frontend dev server.
- Added Phase 2 bundle metadata for app and DMG packaging validation.
- Verified developer-only unsigned/ad-hoc `.app` and `.dmg` generation locally.

## [2026-06-08] governance | Git and PR workflow documented

- Added the contributor-facing Git and PR workflow page.
- Recorded the current branch model as GitHub Flow: protected `main`, short-lived task branches, PRs into `main`, squash merge, and release branches only for packaging preparation.
- Clarified that a long-lived `dev` branch is not part of the project workflow unless a future ADR changes that decision.

## [2026-06-08] phase-1 | CI foundation started

- Added GitHub Actions CI to run the project harness on `main` pushes and pull requests.
- Added the GitHub PR template for local alpha safety and release blocker review.
- Added early-alpha scope guardrails to keep MLX, MCP, plugin runtime, telemetry, analytics, cloud sync, and network upload out of Phase 1.

## [2026-06-08] phase-0 | Repository baseline and alpha decisions

- Initialized the project as a git repository.
- Published the public GitHub repository at https://github.com/raspverry/SilicaRAW.
- Hardened `.gitignore` for Rust outputs, local agent state, release scratch files, secrets, and editor state.
- Added ADR 0002 for the local DMG distribution target.
- Added ADR 0003 for the first Tauri shell and packaging spike path.
- Added ADR 0004 for local alpha scope and license gates.

## [2026-06-08] plan | Local DMG distribution target

- Defined local distribution as a GitHub Release DMG containing a signed and notarized `.app`.
- Added a phase-by-phase local DMG distribution plan.
- Clarified that unsigned DMGs are developer-only artifacts, not the final local distribution target.

## [2026-06-08] scaffold | Initial public and LLM-readable wiki

- Created the initial `docs/wiki/` structure.
- Added overview, decision, topic, source, risk, and question categories.
- Established frontmatter, status, and maintenance conventions.
- Recorded initial source notes for the LLM Wiki pattern, `karpathy/autoresearch`, and `huggingface/ml-intern`.

## Notes for LLM Agents

Use this log to understand recent wiki changes before editing multiple wiki pages.

## [2026-06-20] quality | Source capability contract narrowed

- Completed Q1.1 of the local alpha quality closure plan.
- Defined the installed-alpha source contract as JPEG/JPG-only for preview, Develop, and export readiness.
- Recorded that RAW, PNG, TIFF, HEIC, database, and sidecar-like files may be cataloged by reference as unsupported product rows until later codec-specific proof adds end-to-end evidence.

## [2026-06-20] quality | Export hard-link guard added

- Completed Q1.2 of the local alpha quality closure plan.
- Hardened export overwrite protection so existing output paths that are hard links to the original source are blocked before writes.
- Covered both JPEG export and fixture-backed RAW proof export guard paths.
