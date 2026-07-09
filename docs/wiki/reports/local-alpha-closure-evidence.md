---
title: Local Alpha Closure Evidence
status: active
audience: maintainers
updated: 2026-07-08
source_of_truth: docs/wiki/roadmaps/local-alpha-quality-closure-plan.md
---

# Local Alpha Closure Evidence

## Scope

This index records the evidence used by the [Local Alpha Quality Closure Plan](../roadmaps/local-alpha-quality-closure-plan.md).

It is not a release approval. It separates source, static UI, installed app, unsigned developer-preview DMG, and signed user-ready DMG evidence so maintainers do not treat one gate as proof for another.

## Latest Recorded Source Harness Baseline

| Field | Value |
| --- | --- |
| Branch | `qa/q6-installed-workflow-evidence` |
| Commit | `e0ddfa8919c86930cfc3297d02967214b0a6e5e2` |
| Recent PRs | [#144 Record Q5.5 trust-state evidence](https://github.com/raspverry/SilicaRAW/pull/144), [#145 Record Q6.1 unsigned DMG inspection](https://github.com/raspverry/SilicaRAW/pull/145), [#146 Record Q6.2 installed app launch](https://github.com/raspverry/SilicaRAW/pull/146) |
| CI | Pending for this branch; local harness validation required before merge |
| Final Visual QA | Pending for this branch; no UI surface changes expected |
| Scope | Import/export source hash evidence, closure evidence routing, source/static UI harness gates, built `.app` launch evidence, library import reference evidence, review/edit persistence evidence, JPEG sRGB export evidence, trust-state evidence, unsigned DMG inspection evidence, installed app launch sub-proof, installed executable workflow evidence |
| Artifact status | Current local build-machine unsigned DMG, `/Applications` launch sub-proof, and installed executable workflow evidence recorded; offline and clean-Mac gates still pending |

## Gate Evidence Matrix

| Gate | Status | Proves | Does Not Prove | Command or Record | Artifact Path |
| --- | --- | --- | --- | --- | --- |
| Default source harness | Recorded | Repo docs, guardrails, Rust build/tests, static UI smoke, connected developer runtime smoke | Installed app, DMG install, Gatekeeper, clean-Mac behavior | `scripts/harness/check.sh` | Command output only |
| Visual QA | Recorded by runner contract | Browser/static seeded UI layout, responsive behavior, modal text fit, visual surface drift | Tauri command boundary, native shell, app bundle, install, filesystem writes | `python3 scripts/harness/run-final-visual-qa.py` | `.tmp/final-visual-responsive-qa/screenshots`, `.tmp/final-visual-responsive-qa/visual-qa-results.json` |
| Visual QA docs drift | Recorded | Runner/wiki surface count, viewport count, screenshot count, surface IDs | Screenshot correctness | `python3 scripts/harness/check-visual-qa-docs.py` | Command output only |
| Built `.app` launch | Recorded for current local build | Generated `.app` bundle launches as a GUI app and process path is the app executable | `/Applications` install, full workflow, DMG, Gatekeeper, clean-Mac behavior | [Local Alpha Built App Launch](local-alpha-built-app-launch.md) | `target/release/bundle/macos/SilicaRAW.app`, `.tmp/q5-built-app-launch/installed-app-preflight.json` |
| Library import by reference | Recorded for current local build | Developer desktop runtime creates/opens a library, imports by reference, records catalog paths outside the library root, and preserves source hashes | Manual GUI path picker, `/Applications`, DMG, Gatekeeper, clean-Mac behavior | [Local Alpha Library Import Reference Evidence](local-alpha-library-import-reference.md) | `.tmp/q5-library-import-reference/library-import-reference-evidence.json` |
| Review/edit persistence | Recorded for current local build | Developer desktop runtime persists review flags, undoable edit history, active exposure/contrast edit state, and app-session selected-photo restore state | Manual GUI controls, `/Applications`, DMG, Gatekeeper, clean-Mac behavior | [Local Alpha Review and Edit Persistence Evidence](local-alpha-review-edit-persistence.md) | `.tmp/q5-review-edit-persistence/library-import-reference-evidence.json`, `.tmp/q5-review-edit-persistence/run/AppConfig/app-session.json` |
| JPEG sRGB export | Recorded for current local build | Developer desktop runtime writes a separate JPEG sRGB artifact, records export settings/source SHA evidence, and preserves original source hashes | Manual GUI export dialog, `/Applications`, DMG, Gatekeeper, clean-Mac behavior | [Local Alpha JPEG sRGB Export Evidence](local-alpha-jpeg-export-evidence.md) | `.tmp/q5-jpeg-export-evidence/library-import-reference-evidence.json`, `.tmp/q5-jpeg-export-evidence/run/Exports/reference-urban-export.jpg` |
| Trust states | Recorded for current local build | Developer desktop runtime proves supported PNG readiness, RAW/text unsupported state, deleted-original downgrade, blocked write paths, and disposable cache clear scope | Manual GUI interaction, `/Applications`, DMG, Gatekeeper, clean-Mac behavior | [Local Alpha Trust-State Evidence](local-alpha-trust-state-evidence.md) | `.tmp/q5-trust-states/trust-state-evidence.json` |
| Unsigned DMG inspection | Recorded for current local build | Local build produces unsigned DMG, checksum, `hdiutil verify`, mounted app presence, bundle metadata, and ad-hoc signing state | `/Applications` install, GitHub Release artifact, Gatekeeper, clean-Mac behavior | [Local Alpha Unsigned DMG Inspection](local-alpha-unsigned-dmg-inspection.md) | `target/release/bundle/dmg/SilicaRAW_0.1.0_aarch64.dmg`, `.tmp/q6-unsigned-dmg-inspection/local-dmg-artifact-smoke.json` |
| Installed app launch from `/Applications` | Partial recorded for current local build | Installed app matches the mounted DMG app and launches as `/Applications/SilicaRAW.app/Contents/MacOS/silica-desktop`, not the repository checkout | Full import/edit/export/restart workflow, offline behavior, Gatekeeper, clean-Mac behavior | [Local Alpha Installed App Launch](local-alpha-installed-app-launch.md) | `/Applications/SilicaRAW.app`, `.tmp/q6-installed-app-launch/local-dmg-artifact-smoke.json`, `.tmp/q6-installed-app-launch/installed-app-launch-smoke.json` |
| Installed app workflow from `/Applications` | Recorded for current local build | Installed executable workflow creates/opens a library, imports by reference, populates grid, blocks unsupported RAW placeholder preview, persists rating/Pick/Reject/edit state, exports JPEG sRGB, and preserves original file hashes | WebView click automation, native path picker, offline behavior, Gatekeeper, clean-Mac behavior | [Local Alpha Installed App Workflow](local-alpha-installed-app-workflow.md) | `/Applications/SilicaRAW.app`, `.tmp/q6-installed-workflow/installed-app-workflow-evidence.json` |
| Developer-preview GitHub artifact | Recorded for older preview | Unsigned DMG build, checksum, mount, mounted app presence | Current `main`, user-ready release, signed/notarized behavior, clean-Mac behavior | [Developer Preview Artifacts](developer-preview-artifacts.md) | `.tmp/developer-preview-28434695717/silicaraw-unsigned-developer-preview-macos/SilicaRAW_0.1.0_aarch64.dmg` |
| Offline installed workflow | Pending for current `main` | Local workflow does not require network | Gatekeeper acceptance or clean-Mac behavior by itself | [Local DMG Install Smoke Checklist](../../../checklists/LOCAL_DMG_INSTALL_CHECKLIST.md) | Evidence TBD |
| Signed user-ready DMG | Blocked | Gatekeeper-accepted local alpha candidate | Not applicable until produced | [Local DMG Release Runbook](../roadmaps/local-dmg-release-runbook.md) | Blocked by Developer ID and notarization prerequisites |
| Clean-Mac downloaded artifact | Blocked | GitHub-hosted downloaded artifact works outside the developer machine | Not applicable until signed/notarized artifact exists | [Local DMG Install Smoke Checklist](../../../checklists/LOCAL_DMG_INSTALL_CHECKLIST.md) | Blocked by signed/notarized release artifact |

## Latest Developer-Preview Artifact

| Field | Value |
| --- | --- |
| Tag | `developer-preview-20260630.1` |
| Commit | `d8305260c24b5f6625334176339bd5bd3d922f95` |
| Workflow | `Developer Preview macOS DMG` |
| Run | https://github.com/raspverry/SilicaRAW/actions/runs/28434695717 |
| Artifact | `silicaraw-unsigned-developer-preview-macos` |
| DMG | `SilicaRAW_0.1.0_aarch64.dmg` |
| DMG SHA256 | `665f1998cc7d7d148eecb458cafa0af508d39e33d9fe1f4170221de3f0de4aac` |
| Smoke status | DMG verification and mounted app presence passed |
| Current-main status | Stale relative to `a2f66bfec44306bff290172ef4be10954d16463a`; use [Local Alpha Unsigned DMG Inspection](local-alpha-unsigned-dmg-inspection.md) for current local build-machine DMG evidence |

## Required Record for Remaining Installed App Evidence

Q5.1 through Q5.5, Q6.1, the Q6.2 installed launch sub-proof, and the Q6.2 installed executable workflow now have developer-local evidence records. Remaining installed/offline evidence should add or link records with:

- tested commit and app version
- app path launched, normally `/Applications/SilicaRAW.app`
- source sample folder path class, without committing private files
- original SHA-256 before and after workflow
- library path and export output path class
- command, script, or manual checklist used
- screenshots or logs for unsupported, missing-original, and cache-clear trust states
- pass/fail result and follow-up PR or issue for any failure

## Required Record for Q6 Developer-Preview DMG Evidence

When Q6 starts, update this index and [Developer Preview Artifacts](developer-preview-artifacts.md) with:

- workflow run URL and commit SHA
- artifact name and artifact digest
- DMG file name and SHA-256
- local artifact path under `.tmp/`
- app bundle metadata inspection result
- mounted app tree hash
- installed app tree hash when copied to `/Applications`
- installed workflow result
- unsigned Gatekeeper warning or override note

## Blocked User-Ready Gates

Task 27.2 and Phase 28 cannot be completed from unsigned artifacts. Required missing evidence:

- Apple Developer Program funding
- Developer ID Application certificate
- notarization credentials
- signed, notarized, and stapled DMG
- checksum published with release artifact
- Gatekeeper acceptance for app and DMG
- clean-Mac downloaded-artifact QA

Until those exist, the honest label remains `unsigned developer-preview alpha`.
