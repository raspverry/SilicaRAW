---
title: Open Questions
status: active
audience: all
updated: 2026-06-08
source_of_truth: docs/18_Final_Master_Plan.md
---

# Open Questions

## Summary

This page tracks questions that should be answered by maintainers, spikes, tests, or explicit decisions.

## Architecture

- What exact bridge boundary should implement Spike 001 Path B for the native AppKit/Metal viewer?
- Which physical mouse and trackpad checklist will graduate the viewer bridge from Path B to implementation-ready?
- What architecture guardrail checks should be added after the monorepo foundation?

## RAW and Color

- What fixture set will be legally usable for RAW and color testing?
- What tolerance policy will be used for golden image tests?

## Storage and Data Safety

- How will sidecar conflicts be surfaced to users?
- What cache size and cleanup policy will be used?

## License and Distribution

- What provisional project license will be selected before Gate A?
- What final project license will be selected before public beta?
- How will sample asset licenses be tracked?
- How will model licenses be tracked if models are included?

## Wiki

- Should the wiki later export an `llms.txt` or `llms-full.txt` style artifact?
- Should source ingestion remain manual, or should the project add simple lint/search tooling later?
- What review process should promote wiki pages from `draft` to `active`?

## Answered

- Spike 001 recorded Path B for Tauri + native Metal viewer integration on 2026-06-08.
- Tauri is not rejected by Spike 001, so no SwiftUI/AppKit shell switch is planned yet.
- Spike 002 selected Core Image RAW primary with LibRaw deferred on 2026-06-08.
- Spike 003 selected a linear Display P3-compatible working-space recommendation, sRGB default export, and Display P3 export support on 2026-06-08.
- Spike 004 selected `rusqlite` with bundled SQLite and embedded SQL migrations on 2026-06-08.

## Links

- [Final Master Plan](../../18_Final_Master_Plan.md)
- [Development Roadmap](../../13_Development_Roadmap.md)
- [Architecture Risks](../risks/architecture-risks.md)
- [Spike 001: Tauri + Native Metal Viewer](../../spikes/001-tauri-metal-viewer.md)
- [Spike 002: RAW Decoder Path](../../spikes/002-raw-decoder.md)
- [Spike 003: Color-Managed Preview and Export](../../spikes/003-color-managed-preview-export.md)
- [Spike 004: SQLite Catalog Persistence](../../spikes/004-sqlite-persistence.md)
- [Dependencies Policy](../../DEPENDENCIES.md)

## Notes for LLM Agents

If an implementation task depends on one of these questions, treat it as blocked unless the user explicitly scopes the task as a spike or decision record.
