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
| Branch | `main` |
| Commit | `7bedb18fdd0a8f26ef7b8432d8b87ab6cbbac96a` |
| Recent PRs | [#139 Add local alpha closure evidence index](https://github.com/raspverry/SilicaRAW/pull/139), [#140 Record built app launch evidence](https://github.com/raspverry/SilicaRAW/pull/140), [#141 Record library import reference evidence](https://github.com/raspverry/SilicaRAW/pull/141) |
| CI | [Harness run 28949153961](https://github.com/raspverry/SilicaRAW/actions/runs/28949153961), `success` |
| Final Visual QA | [Final Visual QA run 28949153952](https://github.com/raspverry/SilicaRAW/actions/runs/28949153952), `success` |
| Scope | Import/export source hash evidence, closure evidence routing, source/static UI harness gates, built `.app` launch evidence, library import reference evidence |
| Artifact status | No current-main DMG recorded in this index yet |

## Gate Evidence Matrix

| Gate | Status | Proves | Does Not Prove | Command or Record | Artifact Path |
| --- | --- | --- | --- | --- | --- |
| Default source harness | Recorded | Repo docs, guardrails, Rust build/tests, static UI smoke, connected developer runtime smoke | Installed app, DMG install, Gatekeeper, clean-Mac behavior | `scripts/harness/check.sh` | Command output only |
| Visual QA | Recorded by runner contract | Browser/static seeded UI layout, responsive behavior, modal text fit, visual surface drift | Tauri command boundary, native shell, app bundle, install, filesystem writes | `python3 scripts/harness/run-final-visual-qa.py` | `.tmp/final-visual-responsive-qa/screenshots`, `.tmp/final-visual-responsive-qa/visual-qa-results.json` |
| Visual QA docs drift | Recorded | Runner/wiki surface count, viewport count, screenshot count, surface IDs | Screenshot correctness | `python3 scripts/harness/check-visual-qa-docs.py` | Command output only |
| Built `.app` launch | Recorded for current local build | Generated `.app` bundle launches as a GUI app and process path is the app executable | `/Applications` install, full workflow, DMG, Gatekeeper, clean-Mac behavior | [Local Alpha Built App Launch](local-alpha-built-app-launch.md) | `target/release/bundle/macos/SilicaRAW.app`, `.tmp/q5-built-app-launch/installed-app-preflight.json` |
| Library import by reference | Recorded for current local build | Developer desktop runtime creates/opens a library, imports by reference, records catalog paths outside the library root, and preserves source hashes | Manual GUI path picker, `/Applications`, DMG, Gatekeeper, clean-Mac behavior | [Local Alpha Library Import Reference Evidence](local-alpha-library-import-reference.md) | `.tmp/q5-library-import-reference/library-import-reference-evidence.json` |
| Review/edit persistence | Recorded for current local build | Developer desktop runtime persists review flags, undoable edit history, active exposure/contrast edit state, and app-session selected-photo restore state | Manual GUI controls, `/Applications`, DMG, Gatekeeper, clean-Mac behavior | [Local Alpha Review and Edit Persistence Evidence](local-alpha-review-edit-persistence.md) | `.tmp/q5-review-edit-persistence/library-import-reference-evidence.json`, `.tmp/q5-review-edit-persistence/run/AppConfig/app-session.json` |
| Developer-preview DMG artifact | Recorded for older preview | Unsigned DMG build, checksum, mount, mounted app presence | Current `main`, user-ready release, signed/notarized behavior, clean-Mac behavior | [Developer Preview Artifacts](developer-preview-artifacts.md) | `.tmp/developer-preview-28434695717/silicaraw-unsigned-developer-preview-macos/SilicaRAW_0.1.0_aarch64.dmg` |
| Installed app workflow from `/Applications` | Pending for current `main` | App bundle launch, local persistence, export, original safety from installed app | User-ready signed distribution unless signed/notarized artifact is used | [Local DMG Install Smoke Checklist](../../../checklists/LOCAL_DMG_INSTALL_CHECKLIST.md) | `/Applications/SilicaRAW.app`, output evidence TBD |
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
| Current-main status | Stale relative to `7bedb18fdd0a8f26ef7b8432d8b87ab6cbbac96a`; rebuild required before using it as current evidence |

## Required Record for Remaining Installed App Evidence

Q5.1 through Q5.3 now have developer-local evidence records. Remaining installed-app evidence should add or link records with:

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
