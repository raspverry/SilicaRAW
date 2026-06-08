---
title: Wiki Log
status: active
audience: all
updated: 2026-06-08
source_of_truth: none
---

# Wiki Log

## Summary

This append-only log records meaningful changes to the SilicaRAW wiki.

## Entries

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
