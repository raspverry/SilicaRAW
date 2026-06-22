---
title: Local Alpha Quality Closure Plan
status: active
audience: maintainers
updated: 2026-06-20
source_of_truth: docs/wiki/roadmaps/local-dmg-distribution-plan.md
---

# Local Alpha Quality Closure Plan

## Summary

This plan closes the gap between "the harness passes" and "the local macOS alpha feels trustworthy when a person uses it." It supersedes ad hoc blocked-gate UI hardening while Task 27.2 remains blocked by signing, notarization, and clean-Mac downloaded-artifact prerequisites.

The current product is not yet a user-ready local alpha DMG. Until the signed/notarized path is available and verified, the honest label is **unsigned developer-preview alpha**.

## Inputs

- Code review graph context: desktop/static UI and photo pipeline are the active risk areas.
- UI visual review: the current Library and Develop screens can look like a status-board shell instead of a photo-first editor, even when DOM overflow checks pass.
- Release review: default harness checks do not prove installed app or DMG behavior.
- Storage review: source support, export overwrite, and missing-original states need trust hardening before visual polish.
- Architecture review: scope must freeze until the local alpha workflow is sealed.
- Mockup source set: `MockupUI/M001` through `MockupUI/M016`.
- Current automated visual QA result: 28 surfaces, 84 screenshots across `1280x800`, `1440x900`, and `1728x965`.

## Closure Order

Follow this order. Do not polish around broken trust states.

1. Reconcile evidence and routing.
2. Close P0 data trust and source-support contracts.
3. Polish the photo-first UI against the mockups.
4. Verify interaction, keyboard, modal, focus, and responsive behavior.
5. Add the right QA gates without making the default harness excessive.
6. Prove the installed `.app` workflow.
7. Prove the unsigned developer-preview DMG.
8. Resume signed user-ready DMG work only after Developer ID prerequisites exist.

## Product Labels

| Label | Meaning | Allowed Now |
|---|---|---|
| `unsigned developer-preview alpha` | Internal GitHub Actions or local unsigned DMG for development QA. Gatekeeper warnings are expected. | Yes |
| `local alpha candidate` | A signed/notarized DMG candidate that passes installed app workflow QA. | No |
| `user-ready local alpha` | A GitHub Release DMG that a user can download, install, and run with clean-Mac evidence. | No |
| `public beta` | Broader release beyond the local alpha trust gate. | No |

## Non-Goals

- No MLX runtime or model loading.
- No MCP server or runtime.
- No plugin runtime.
- No cloud sync, telemetry, auto-update, Homebrew, or Mac App Store work.
- No broad RAW support claims beyond documented proof paths.
- No custom shortcut remapping.
- No large fallback systems.
- No original photo mutation.

## Phase Q0: Evidence and Routing Reconciliation

### Q0.1: Route Agents to This Closure Plan

- **Location:** `docs/wiki/llm/current-route.md`, `docs/wiki/index.md`
- **Dependencies:** none
- **Work:** Make this page the active route while Task 27.2 is blocked. Keep the blocked public beta UI hardening plan as an input, not the top-level route.
- **Acceptance:** New agents can discover that local alpha quality closure is the next workstream before feature growth.
- **Validation:** `python3 scripts/harness/check-md-links.py`

### Q0.2: Correct Visual QA Inventory Drift

- **Location:** `docs/wiki/topics/ui-visual-responsive-qa.md`, `scripts/harness/run-final-visual-qa.py`
- **Dependencies:** none
- **Work:** Align docs with the runner's current 28-surface, 84-screenshot result set, including `M025-library-unsupported-grid`, `M026-loupe-unsupported`, `M027-loupe-missing`, and `M028-develop-unsupported`.
- **Acceptance:** Docs and runner agree on surface count, viewport count, and artifact location.
- **Validation:** `python3 scripts/harness/run-final-visual-qa.py` when UI files change; docs-only changes may use link checks.

### Q0.3: Normalize Alpha Terminology

- **Location:** release docs, current route, runbooks
- **Dependencies:** Q0.1
- **Work:** Use `unsigned developer-preview alpha` until signing and notarization exist. Reserve `user-ready local alpha` for the signed/notarized DMG gate.
- **Acceptance:** No docs imply that an unsigned DMG is ready for normal users.
- **Validation:** `rg -n "user-ready|public beta|developer-preview|unsigned" docs/wiki`

### Q0.4: Freeze Product Expansion

- **Location:** `docs/wiki/llm/current-route.md`
- **Dependencies:** Q0.1
- **Work:** Restate that product expansion waits until this closure plan finishes or explicitly routes to a new phase.
- **Acceptance:** Agents do not add new Develop controls, new export formats, RAW claims, MLX, MCP, plugin runtime, or preferences expansion while closing alpha quality.
- **Validation:** review diff scope.

## Phase Q1: Data Trust and Source Contract

### Q1.1: Define One Source Capability Contract

- **Location:** core catalog/readiness code, desktop state mapping, UI copy, docs
- **Dependencies:** Q0
- **Work:** Make the contract explicit: JPEG/JPG are the supported local alpha source formats. Unsupported files may be cataloged by reference, but they must not look preview-ready, develop-ready, or export-ready.
- **Acceptance:** PNG, TIFF, HEIC, database files, and sidecar-like files are not shown as supported photo sources. Unsupported rows remain understandable and compact.
- **Validation:** targeted core/storage tests plus `python3 scripts/harness/check-ui-workflow-smoke.py`
- **Status:** Completed on 2026-06-20. Catalog, preview, Develop commit, desktop static UI, fixture generator, and docs now use the JPEG/JPG-only installed-alpha source contract.

### Q1.2: Harden Export Against Same-File and Hard-Link Writes

- **Location:** export write guard
- **Dependencies:** Q1.1
- **Work:** Prevent export destinations that resolve to the original file or the same inode through hard links.
- **Acceptance:** Export cannot overwrite or mutate an original by path, symlink, or hard-link equivalence.
- **Validation:** `cargo test -p silica-export hard_link`
- **Status:** Completed on 2026-06-20. JPEG export and RAW proof export guards now compare existing output destinations by canonical path and Unix file identity before any output write.

### Q1.3: Downgrade Missing Originals from Ready State

- **Location:** preview/open-photo pipeline, desktop state mapping, export gating
- **Dependencies:** Q1.1
- **Work:** If the referenced original is missing, preview, histogram, Develop, and export states must show a missing-original block instead of `Ready`.
- **Acceptance:** A catalog row with a deleted original cannot be developed or exported and does not show stale thumbnail readiness as proof of source availability.
- **Validation:** targeted core/desktop tests plus workflow smoke.
- **Status:** Completed on 2026-06-20. Runtime grid/query mapping, preview, histogram, Develop, export, and desktop command responses now downgrade deleted referenced originals to missing/blocked states and hide stale thumbnail readiness without mutating catalog state.

### Q1.4: Tighten Sidecar Rebuild Dry-Run Behavior

- **Location:** storage sidecar rebuild path
- **Dependencies:** Q1.1
- **Work:** Schema-invalid sidecars should produce issues only during dry-run rebuild, not rebuild entries that imply recoverability.
- **Acceptance:** Dry-run output separates recoverable rebuild work from invalid sidecar issues.
- **Validation:** `python3 scripts/harness/check-sidecar-contract.py`
- **Status:** Completed on 2026-06-20. Storage dry-run now stops schema-invalid sidecars at issue reporting, produces no rebuild entries for them, and documents that rebuild precedence applies only after sidecar schema validation passes.

### Q1.5: Verify Disposable Cache Clear Safety

- **Location:** storage cache-clear path, harness
- **Dependencies:** Q1.1
- **Work:** Add narrow verification for symlinked or adversarial cache paths without creating a broad fallback cleaner.
- **Acceptance:** Cache clear only deletes disposable cache material and never follows paths into originals, sidecars, exports, backups, or logs.
- **Validation:** targeted storage test plus `scripts/harness/check.sh`
- **Status:** Completed on 2026-06-20. Cache clear now uses symlink-aware metadata handling for disposable cache roots, removes symlink cache entries without following protected targets, and verifies nested symlinks into originals are removed without deleting originals.

## Phase Q2: Photo-First UI Product Polish

### Q2.1: Create a Mockup Parity Checklist

- **Location:** `docs/wiki/topics/ui-visual-responsive-qa.md` or a linked report
- **Dependencies:** Q0
- **Work:** Compare actual screenshots against `MockupUI/M003`, `M004`, `M005`, `M007`, `M008`, `M009`, and `M010`. Record only actionable deltas.
- **Acceptance:** The checklist prioritizes visual hierarchy, spacing, typography, panel density, image state, and copy problems. It does not ask for new product features.
- **Validation:** manual screenshot review plus visual QA artifacts.
- **Status:** Completed on 2026-06-20. Added the linked UI Mockup Parity Checklist for the then-current 25-surface visual QA artifacts, prioritizing photo-first hierarchy, viewer dominance, thumbnail/card density, honest preview states, inspector rhythm, and disabled/future-state demotion without adding product features.

### Q2.2: Restore Photo-First Library Hierarchy

- **Location:** desktop static Library UI and style tokens
- **Dependencies:** Q1.1, Q2.1
- **Work:** Reduce maintenance/status-board dominance, keep import and cache controls secondary, and make the photo grid the primary surface when photos exist.
- **Acceptance:** The populated Library screen reads as a photo browser, not a diagnostic page.
- **Validation:** `python3 scripts/harness/check-static-ui.py`, visual QA, manual screenshot review.
- **Status:** Completed on 2026-06-20. Populated Library now uses explicit `grid`, `loupe`, and `ai-review` view state, places the photo grid before disposable cache maintenance when photos exist, demotes cache clear to a bottom utility strip, and hides Library maintenance chrome in Loupe so the viewer reads as the primary surface.

### Q2.3: Fix Thumbnail Card State Density

- **Location:** Library grid card markup/styles
- **Dependencies:** Q1.1, Q2.2
- **Work:** Prevent metadata, filename, badges, and unsupported labels from overlapping. Keep unsupported state compact and non-repetitive.
- **Acceptance:** Grid cards remain readable at standard and compact desktop widths. Unsupported files are clear without filling every card with noisy warnings.
- **Validation:** visual QA including `M025-library-unsupported-grid`.
- **Status:** Completed on 2026-06-20. Unsupported and missing thumbnail cards now keep one textual state badge, remove duplicated file-type and empty-rating noise, and keep supported-card ratings on a fixed-width footer slot for compact and standard grid widths.

### Q2.4: Make Image Preview States Honest

- **Location:** Loupe and Develop preview surfaces
- **Dependencies:** Q1.3
- **Work:** Supported JPEG/JPG photos should render as normal photos. Unsupported or missing originals should use a deliberate neutral blocked state, not tinted mockup imagery or fake preview content.
- **Acceptance:** Users can tell whether they are seeing a real selected photo, a cached thumbnail, or an unavailable source state.
- **Validation:** workflow smoke plus manual screenshot review with a real local sample folder.
- **Status:** Completed on 2026-06-20. Supported visual QA JPEG fixtures now render as deterministic photo-like reference images instead of gradient/checker mockups, unavailable rows cannot leak stale thumbnail bytes through the desktop command boundary, and Loupe/Develop explicitly distinguish ready, unsupported, and missing-original states with no preview image for unavailable sources.

### Q2.5: Unify Inspector Density and Panel Rhythm

- **Location:** right inspector panels across Library, Loupe, Develop, Export, Preferences
- **Dependencies:** Q2.1
- **Work:** Align headings, row spacing, labels, button sizes, status copy, and disabled states across panels.
- **Acceptance:** Inspector content does not shift between unrelated component styles from screen to screen.
- **Validation:** static UI checks, visual QA, manual review.
- **Status:** Completed on 2026-06-20. Right inspector sections now use a tighter shared section rhythm, label/value rows share the same control typography, Export summary and recent-export headings no longer overpower inspector headings, disabled controls are demoted in scoped panel contexts, and batch export failures separate file name from block reason. The ready Develop visual QA state now also asserts the mask support copy stays `Manual`, not `Unsupported`.

### Q2.6: Resolve Known Visual Contradictions

- **Location:** Export workflow, Preferences Advanced, Develop History, AI Review surfaces
- **Dependencies:** Q2.1
- **Work:** Fix contradictory export states, clipped advanced preferences content, unreadable history density, and over-prominent AI review states while keeping disabled future paths honest.
- **Acceptance:** Blocked or future features are clear but not visually louder than the editor workflow.
- **Validation:** visual QA surfaces `M017`, `M021`, `M022`, `M023`, and `M024`.
- **Status:** Completed on 2026-06-20. Export workflow Display P3 state now keeps the read-only field, summary, and safety copy consistent; Advanced Preferences compresses permission-contract readback so the plugin review is not clipped; Develop History opens directly on the readable history panel; and AI Review/approval states use quieter status, selected-card, and action emphasis while preserving explicit approval gates.

## Phase Q3: Interaction, Keyboard, Accessibility, and Resize Gate

### Q3.1: Verify Escape Dismiss Priority

- **Location:** desktop UI global key handling
- **Dependencies:** Q2
- **Work:** Ensure `Escape` closes exactly one topmost dismissible surface in the documented order.
- **Acceptance:** Dialogs, import issue review, import panel, Loupe, AI Review, and grid multi-select do not fight each other.
- **Validation:** `python3 scripts/harness/check-ui-workflow-smoke.py`
- **Status:** Completed on 2026-06-22. The global dismiss handler now follows the documented order from the public beta UI hardening plan, and Library grid multi-selection is the lowest-priority dismissible state so `Escape` clears it even when grid focus has moved elsewhere.

### Q3.2: Verify Read-Only Shortcuts Discovery

- **Location:** Welcome, Preferences, shortcuts dialog
- **Dependencies:** Q3.1
- **Work:** Keep shortcuts discoverable from UI and `?`, but do not implement custom shortcut binding.
- **Acceptance:** Users can find active shortcuts without a settings system that does not exist.
- **Validation:** workflow smoke plus manual keyboard check.
- **Status:** Completed on 2026-06-22. The shortcuts dialog is reachable from the Welcome footer, Preferences > Shortcuts, and the `?` key, describes those access paths, lists only active local-alpha shortcuts, and keeps custom remapping disabled.

### Q3.3: Verify Focus Return and Keyboard-Only Flow

- **Location:** dialog openers, grid, toolbar, inspector controls
- **Dependencies:** Q3.1
- **Work:** Closing a dialog returns focus to a sensible opener or work surface. Keyboard-only operation can import, select, rate, open preview, edit supported controls, and export where supported.
- **Acceptance:** No modal trap, lost focus, or invisible focus state in the local alpha path.
- **Validation:** targeted smoke checks plus manual keyboard pass.

### Q3.4: Test Realistic Resize Boundaries

- **Location:** responsive CSS and visual QA runner
- **Dependencies:** Q2
- **Work:** Cover `1180px` default app width, `1280px`, `1440px`, `1728px`, and narrow rail behavior near `1024px` or explicit collapsed sidebar state.
- **Acceptance:** Standard desktop keeps readable sidebar text. Narrow rail behavior is intentional, reversible, and not triggered at `1279px`.
- **Validation:** visual QA runner or manual screenshot evidence for each breakpoint.

### Q3.5: Check Modal Scroll and Text Fit

- **Location:** Preferences, Export, shortcuts, import issue review
- **Dependencies:** Q3.4
- **Work:** Verify long copy, disabled states, and buttons do not clip or overlap when dialogs are smaller than ideal.
- **Acceptance:** Text fits within containers without crowding, and scroll areas are obvious where needed.
- **Validation:** visual QA plus manual resize screenshots.

## Phase Q4: Harness and Evidence Gate

### Q4.1: Define When Final Visual QA Must Run

- **Location:** docs, PR workflow, optional CI path filter
- **Dependencies:** Q2, Q3
- **Work:** Keep `scripts/harness/check.sh` reasonably small, but require `python3 scripts/harness/run-final-visual-qa.py` for UI-affecting changes.
- **Acceptance:** UI regressions are not hidden behind a passing default harness, and non-UI PRs are not slowed unnecessarily.
- **Validation:** docs link check and PR template/workflow review if touched.

### Q4.2: Add Drift Detection for Visual QA Docs

- **Location:** harness or lightweight docs check
- **Dependencies:** Q0.2
- **Work:** Detect runner/docs mismatch for surface count or known surface IDs without making screenshot generation mandatory.
- **Acceptance:** A future `M029` cannot be added to the runner while the wiki still claims `M028`.
- **Validation:** targeted script check or documented manual gate.

### Q4.3: Separate Static UI QA from Installed App QA

- **Location:** wiki QA docs, release runbooks
- **Dependencies:** Q4.1
- **Work:** State clearly that browser/static screenshots prove layout only. Installed app QA proves desktop command, app bundle, native shell, and local persistence behavior.
- **Acceptance:** Agents stop treating static UI screenshots as installed product readiness.
- **Validation:** docs link check.

### Q4.4: Maintain a Closure Evidence Index

- **Location:** docs/wiki or release evidence docs
- **Dependencies:** Q4.3
- **Work:** Record the exact command, artifact path, screenshot set, app build, and DMG file used for each gate.
- **Acceptance:** A maintainer can reproduce why a candidate passed or failed without rereading chat history.
- **Validation:** evidence index review.

## Phase Q5: Installed App Workflow Seal

### Q5.1: Launch the Built `.app`, Not Only Static HTML

- **Location:** desktop build/run scripts, QA runbook
- **Dependencies:** Q1, Q2, Q3
- **Work:** Run the app bundle path that local users will launch. Static HTML checks are not enough for this phase.
- **Acceptance:** The tested path matches the app bundle behavior, not only browser simulation.
- **Validation:** installed app smoke log.

### Q5.2: Prove Library and Import by Reference

- **Location:** installed app QA runbook and harness where practical
- **Dependencies:** Q5.1
- **Work:** Create or open a local library, import a folder by reference, and verify catalog rows point at originals without copying or modifying them.
- **Acceptance:** Original files are unchanged before and after import.
- **Validation:** original hash manifest before/after.

### Q5.3: Prove Review and Edit Persistence

- **Location:** installed app QA runbook and command smoke
- **Dependencies:** Q5.2
- **Work:** Rate, pick, reject, open preview, apply exposure/contrast, restart the app, and verify state persists.
- **Acceptance:** Local alpha workflow state survives restart through documented storage paths.
- **Validation:** installed app smoke plus catalog/edit state inspection.

### Q5.4: Prove JPEG sRGB Export

- **Location:** export workflow
- **Dependencies:** Q5.3
- **Work:** Export a supported JPEG/JPG photo to JPEG sRGB and verify output exists, opens, and does not overwrite any original.
- **Acceptance:** Export creates a new artifact with evidence and leaves originals unchanged.
- **Validation:** output inspection plus original hash manifest before/after.

### Q5.5: Prove Unsupported, Missing, and Cache-Clear States

- **Location:** installed app QA runbook
- **Dependencies:** Q5.4
- **Work:** Check unsupported source rows, deleted original rows, and disposable cache clear behavior in the installed app.
- **Acceptance:** Trust states from Q1 are visible and accurate in the real app.
- **Validation:** screenshot and action-log evidence.

## Phase Q6: Unsigned Developer-Preview DMG Gate

### Q6.1: Build and Inspect the Unsigned DMG

- **Location:** developer preview artifact workflow and runbook
- **Dependencies:** Q5
- **Work:** Build the unsigned DMG, mount it, inspect bundle metadata, and verify expected app contents.
- **Acceptance:** The artifact is internally testable and clearly labeled as unsigned developer preview.
- **Validation:** checksum, mount log, bundle inspection.

### Q6.2: Install to `/Applications` and Launch

- **Location:** local macOS QA runbook
- **Dependencies:** Q6.1
- **Work:** Copy the app from the DMG to `/Applications`, launch it, and rerun the minimal installed workflow.
- **Acceptance:** The app does not depend on the repository checkout or dev server.
- **Validation:** installed app workflow evidence from `/Applications`.

### Q6.3: Verify Offline Behavior

- **Location:** local macOS QA runbook
- **Dependencies:** Q6.2
- **Work:** Launch and run the local alpha workflow without network-dependent features.
- **Acceptance:** Core local editor flow works offline. Disabled MLX, MCP, plugin, cloud, and update paths remain unavailable.
- **Validation:** manual/offline QA note.

### Q6.4: Prepare Developer Preview Release Notes

- **Location:** release notes or runbook
- **Dependencies:** Q6.3
- **Work:** Document unsigned warning expectations, supported source contract, known blocked gates, and local-only workflow.
- **Acceptance:** No release note claims signed/notarized user-ready status.
- **Validation:** release note review.

## Phase Q7: Signed User-Ready Local DMG Gate

### Q7.1: Unblock Signing Prerequisites

- **Location:** release runbook, secrets documentation
- **Dependencies:** external funding and Developer ID access
- **Work:** Acquire Developer ID certificate, signing identity, notarization credentials, and CI secret setup.
- **Acceptance:** Maintainer can sign and notarize without manual secret leakage.
- **Validation:** dry-run signing checklist.

### Q7.2: Sign, Notarize, and Staple

- **Location:** release workflow
- **Dependencies:** Q7.1
- **Work:** Sign the app bundle, notarize the DMG, staple tickets, and verify Gatekeeper acceptance.
- **Acceptance:** Downloaded artifact launches on a clean Mac without unsigned-app warnings.
- **Validation:** `spctl`, notarization log, clean-Mac launch evidence.

### Q7.3: Clean-Mac Downloaded Artifact QA

- **Location:** release evidence index
- **Dependencies:** Q7.2
- **Work:** Download the GitHub Release artifact on a clean Mac and run the local alpha workflow.
- **Acceptance:** The GitHub-hosted DMG works outside the developer machine.
- **Validation:** clean-Mac evidence log and original hash manifest.

### Q7.4: Publish User-Ready Local Alpha

- **Location:** GitHub Release
- **Dependencies:** Q7.3
- **Work:** Publish the signed/notarized DMG, checksum, release notes, scope, and known limitations.
- **Acceptance:** The release can be described as a user-ready local alpha.
- **Validation:** post-release download and launch check.

## Execution Rules

- Execute phases in order unless a task explicitly says it is docs-only and unblocks routing.
- Keep each task atomic and committable.
- Prefer one PR per phase, or smaller PRs for Q1 data trust work.
- Run `scripts/harness/check.sh` before claiming a phase complete.
- Run `python3 scripts/harness/run-final-visual-qa.py` for UI-affecting PRs.
- Do not add dependencies without updating [Dependencies Policy](../../DEPENDENCIES.md).
- Do not commit user-supplied private sample images.

## Links

- [Local DMG Distribution Plan](local-dmg-distribution-plan.md)
- [Developer Preview Artifact Runbook](developer-preview-artifact-runbook.md)
- [Local DMG Release Runbook](local-dmg-release-runbook.md)
- [Blocked Public Beta UI Hardening Plan](blocked-public-beta-ui-hardening-plan.md)
- [UI Visual and Responsive QA](../topics/ui-visual-responsive-qa.md)
- [Data Safety](../topics/data-safety.md)
- [Current LLM Route](../llm/current-route.md)

## Notes for LLM Agents

Read this plan before starting more product feature work while Task 27.2 is blocked. If you find a visual issue, first decide whether it is a Q1 trust problem, Q2 product hierarchy problem, Q3 interaction problem, or Q4 evidence problem. Do not treat screenshots as implementation evidence for installed app behavior.
