---
title: Phase 18 Summary
status: active
audience: all
updated: 2026-06-13
source_of_truth: docs/wiki/phases/phase-18-professional-editing-baseline.md
---

# Phase 18 Summary

## Summary

Phase 18 is complete. It added the P1 Develop baseline on top of Phase 17 without weakening edit graph validation, history semantics, preview/export parity, or original-file safety.

Use this page for context before Phase 19. Read the full [Phase 18 Professional Editing Baseline Brief](phase-18-professional-editing-baseline.md) only when changing Phase 18 behavior.

## Delivered Behavior

- Tone curve: schema-owned RGB point curve mutation, JPEG/JPG preview, undoable commit, JPEG export parity, compact Develop UI, and visual QA.
- HSL color mixer: schema-owned per-channel hue, saturation, and luminance mutation, supported JPEG/JPG preview/export parity, compact Develop UI, and visual QA.
- Detail baseline: schema-owned sharpening and non-MLX noise-reduction values, with explicit unsupported preview/commit/export/UI boundaries. No Detail pixel effect is claimed.
- Lens and geometry: schema-owned lens and geometry values, supported rectangular crop, quarter-turn rotate, and flip preview/export behavior, plus UI that keeps lens correction and arbitrary transform disabled.
- Edit clipboard: graph-only payloads, copy, paste-to-primary, selected-page batch sync, all-or-none history commits, explicit subset choice, disabled unsupported subsets, and JPEG/JPG Develop target gating.

## Durable Boundaries

- Draft preview remains render-only and does not write catalog state, sidecars, exports, action-log rows, caches outside the render path, or originals.
- Committed edits write one undoable catalog checkpoint per affected photo.
- Batch sync writes only after target preflight succeeds; blocked targets produce no partial catalog writes.
- RAW rows are not Develop clipboard targets in this alpha, even when catalog import keeps them as supported photo candidates.
- Detail, lens correction, arbitrary transform, angled crop, MLX denoise, plugin runtime, MCP runtime, cloud sync, telemetry, auto-update, broad RAW support, and camera/lens profile database behavior remain out of scope.
- Original photo files and original metadata must remain unchanged.

## Evidence

- Phase 18 task cards `18.1.1` through `18.5.3` are complete.
- Final Phase 18 merge: `c000ee2 feat(ui): add edit clipboard sync panel (#72)`.
- PR #72 GitHub CI `Harness` completed successfully on 2026-06-13.
- Local verification after merge: `scripts/harness/check.sh` passed on `main`.
- Code-review graph was rebuilt after the final merge.

## Phase 19 Dependency

Phase 19 may rely on validated edit graph mutation, undoable history, explicit unsupported-state UI patterns, and original-file safety from Phase 18. It must still create its own mask task cards before implementation.

## Notes for LLM Agents

Do not reopen Phase 18 planning by default. Use the task cards only when modifying a specific completed Phase 18 slice or investigating a regression in that slice.
