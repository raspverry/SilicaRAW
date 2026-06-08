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
