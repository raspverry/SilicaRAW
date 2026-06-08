---
title: Risk Register
status: active
audience: all
updated: 2026-06-08
source_of_truth: docs/18_Final_Master_Plan.md
---

# Risk Register

## Summary

This section tracks project risks that affect sequencing, architecture, implementation safety, and release trust.

## Current Risk Pages

- [Architecture Risks](architecture-risks.md)

## Related Topic Pages

- [Catalog](../topics/catalog.md)

## Highest-Level Risks

- Tauri + Metal viewer integration.
- RAW decoder quality and support.
- Color management correctness.
- Data migration and original-file safety.
- Large catalog performance.
- MLX runtime and model safety, deferred from local alpha by ADR 0005.
- Plugin and MCP permission safety.

## Links

- [Final Master Plan](../../18_Final_Master_Plan.md)
- [Development Roadmap](../../13_Development_Roadmap.md)
- [Testing and QA Plan](../../15_Testing_QA_Plan.md)
- [Open Questions](../questions/open-questions.md)

## Notes for LLM Agents

Risks are not blockers for all work. They are sequencing constraints. Do foundation tasks first, and do not build broad product features on top of unresolved spike risks.
