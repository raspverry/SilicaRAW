---
title: RAW Decoding
status: active
audience: all
updated: 2026-06-08
source_of_truth: docs/20_v1_1_Architecture_Patch.md
---

# RAW Decoding

## Summary

RAW decoding is one of SilicaRAW's highest-risk technical areas. The project has not selected a final decoder path yet.

## Current Stance

- RAW decoding must be abstracted behind `silica-decode`.
- The initial MVP recommendation is Core Image RAW primary plus a LibRaw spike.
- Full decoder-dependent features must wait for Spike 002.

## Blocked Work

- Camera profile assumptions.
- Broad camera support claims.
- Fuji RAF support commitments.
- Lens metadata and correction behavior.
- Final color pipeline assumptions tied to decoder output.

## Possible Outcomes

- Core Image RAW primary.
- LibRaw primary.
- Hybrid Core Image and LibRaw path.

## Links

- [Architecture Patch](../../20_v1_1_Architecture_Patch.md)
- [RAW Editing Feature Specification](../../07_RAW_Editing_Feature_Specification.md)
- [System Architecture](../../03_System_Architecture.md)
- [Dependencies Policy](../../DEPENDENCIES.md)
- [Open Questions](../questions/open-questions.md)

## Notes for LLM Agents

Do not implement RAW decoding until the decoder spike is explicitly requested. Do not add LibRaw or a Rust binding without updating `docs/DEPENDENCIES.md`.

