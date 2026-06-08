---
title: Data Safety
status: active
audience: all
updated: 2026-06-08
source_of_truth: docs/10_Data_Model_and_Storage_Specification.md
---

# Data Safety

## Summary

Data safety is a core trust requirement. Originals are sacred, catalog state must be recoverable, edits are versioned, and caches are disposable.

## Current Stance

- Original photo files must never be modified by SilicaRAW.
- Catalog state lives in SQLite.
- Sidecars provide portable recovery state.
- Caches may be deleted without losing originals, edits, ratings, collections, presets, or sidecars.

## Early Required Tests

- Original hash protection.
- SQLite migration safety.
- Edit graph serialization.
- Sidecar read/write.
- Cache clear safety.

## Links

- [Data Model and Storage Specification](../../10_Data_Model_and_Storage_Specification.md)
- [Testing and QA Plan](../../15_Testing_QA_Plan.md)
- [Schema Reference](../../19_Schema_Reference.md)
- [Agent Rules](../../../codex/AGENT_RULES.md)

## Notes for LLM Agents

Any task that touches files, catalog state, sidecars, exports, or caches must preserve original-file safety. Do not add convenience file operations that can mutate or delete originals.

