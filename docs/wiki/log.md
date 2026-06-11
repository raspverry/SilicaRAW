---
title: Wiki Log
status: active
audience: all
updated: 2026-06-11
source_of_truth: none
---

# Wiki Log

## Summary

This append-only log records meaningful changes to the SilicaRAW wiki.

## Entries

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
