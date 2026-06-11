---
title: Wiki Log
status: active
audience: all
updated: 2026-06-10
source_of_truth: none
---

# Wiki Log

## Summary

This append-only log records meaningful changes to the SilicaRAW wiki.

## Entries

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
