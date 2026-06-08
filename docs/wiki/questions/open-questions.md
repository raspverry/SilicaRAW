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

- Which Spike 001 path will be recorded for Tauri + native Metal viewer integration?
- If Tauri is unsuitable, when will the project switch planning to SwiftUI/AppKit shell plus Rust Core?
- What architecture guardrail checks should be added after the monorepo foundation?

## RAW and Color

- Which RAW decoder path will be selected after Spike 002?
- What working color space will be selected after the color spike?
- What fixture set will be legally usable for RAW and color testing?
- What tolerance policy will be used for golden image tests?

## Storage and Data Safety

- Which SQLite binding will be selected, and why?
- What migration framework shape will be used?
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

## Links

- [Final Master Plan](../../18_Final_Master_Plan.md)
- [Development Roadmap](../../13_Development_Roadmap.md)
- [Architecture Risks](../risks/architecture-risks.md)
- [Dependencies Policy](../../DEPENDENCIES.md)

## Notes for LLM Agents

If an implementation task depends on one of these questions, treat it as blocked unless the user explicitly scopes the task as a spike or decision record.

