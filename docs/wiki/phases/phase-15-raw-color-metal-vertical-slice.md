---
title: Phase 15 RAW Color Metal Vertical Slice Brief
status: active
audience: all
updated: 2026-06-12
source_of_truth: docs/wiki/roadmaps/post-alpha-master-execution-plan.md
---

# Phase 15 RAW Color Metal Vertical Slice Brief

## Summary

Phase 15 starts the first fixture-backed RAW, color, and Metal product vertical slice.

This phase must turn the Phase 12 RAW proof, Phase 13 color proof, and Phase 14 native viewer bridge into one narrow product path:

```txt
fixture-backed RAW decode
-> bounded disposable preview artifact
-> native viewer preview texture identity
-> exposure/contrast draft path
-> committed edit graph
-> RAW-derived JPEG sRGB export with ICC evidence
-> manual color QA record
```

The phase is evidence-limited. It must not claim broad RAW support, broad color correctness, camera coverage, or production-grade Metal rendering beyond the fixture-backed path that is actually proven.

## Required Read Set

For all Phase 15 tasks, read:

- [Post-Alpha Master Execution Plan](../roadmaps/post-alpha-master-execution-plan.md)
- [Post-Alpha Product Roadmap](../roadmaps/post-alpha-product-roadmap.md)
- [RAW Decoding](../topics/raw-decoding.md)
- [Color Management](../topics/color-management.md)
- [Metal Rendering](../topics/metal-rendering.md)
- [Data Safety](../topics/data-safety.md)
- [Phase 12 RAW Proof Brief](phase-12-raw-proof.md)
- [Phase 13 Color Pipeline Proof Brief](phase-13-color-pipeline-proof.md)
- [Phase 14 Product Metal Viewer Bridge Brief](phase-14-product-metal-viewer-bridge.md)
- The matching task card under [Task Cards](../tasks/index.md)

When a task adds or changes dependencies, also read:

- [Dependencies Policy](../../DEPENDENCIES.md)

When a task changes edit graph fields or persisted catalog/export state, also read:

- [Edit Graph Schema](../../../schemas/edit_graph.schema.json)
- [Schema Reference](../../19_Schema_Reference.md)

## Task Order

0. [Task 15.0: Vertical Slice Evidence Gate](../tasks/15.0-vertical-slice-evidence-gate.md)
1. [Task 15.1: Decoded Image Handoff Contract](../tasks/15.1-decoded-image-handoff-contract.md)
2. [Task 15.2: RAW Decode to Preview Artifact](../tasks/15.2-raw-decode-preview-artifact.md)
3. [Task 15.3: Metal Preview Display](../tasks/15.3-metal-preview-display.md)
4. [Task 15.4: Exposure/Contrast Metal Draft Path](../tasks/15.4-exposure-contrast-metal-draft-path.md)
5. [Task 15.5: RAW-Derived JPEG sRGB Export](../tasks/15.5-raw-derived-jpeg-srgb-export.md)
6. [Task 15.6: RAW Export Manual Color QA](../tasks/15.6-raw-export-manual-color-qa.md)

## Scope

- Use fixture-backed RAW evidence only.
- Keep RAW preview artifacts disposable and under library cache paths.
- Keep original RAW files unchanged.
- Keep preview cache and full-resolution export paths separate.
- Keep draft preview interaction read-only with respect to catalog/edit history.
- Embed and record sRGB ICC evidence for RAW-derived JPEG export.
- Record manual Preview.app or Photos color QA before broadening claims.

## Non-Goals

- No broad RAW camera support claim.
- No RAW support inferred from file extension alone.
- No silent profile fallback for missing or unsupported color data.
- No Display P3 default export change.
- No MLX, MCP, plugin runtime, cloud sync, telemetry, auto-update, Homebrew, or Mac App Store scope.
- No original-file mutation.

## Required Fixtures

- RAW Class A minimum.
- RAW Class C or D recommended for higher-risk path coverage.
- Color Class F for ICC/profile regression.

If fixture evidence is unavailable, Phase 15 implementation must stop at the evidence gate rather than creating fake product support.

Task 15.0 completed the evidence gate on 2026-06-12. The allowed scope is recorded in [Phase 15 Vertical Slice Evidence Gate](../../../checklists/PHASE_15_VERTICAL_SLICE_EVIDENCE.md): RAW classes A-D are fixture-backed local Core Image proof inputs, RAW class E remains blocked, and Color Class F is required for ICC/profile regression only.

## Validation Strategy

- Task 15.0: evidence gate docs and fixture manifest routing. Completed on 2026-06-12.
- Task 15.1: `cargo test -p silica-decode -p silica-render -p silica-core`. Completed on 2026-06-12.
- Task 15.2: fixture-backed RAW preview artifact tests and original hash checks. Completed on 2026-06-12.
- Task 15.3: feature-gated desktop/native viewer tests. Completed on 2026-06-12.
- Task 15.4: no-draft-write tests and edit graph commit tests. Completed on 2026-06-12.
- Task 15.5: export inspection, ICC evidence, output/source hash checks.
- Task 15.6: manual Preview.app or Photos QA record.
- Before completion: `scripts/harness/check.sh`.

## Stop Gates

Stop if:

- RAW support is inferred from extension alone.
- Required RAW or color fixture evidence is missing.
- A preview artifact can write outside disposable library cache paths.
- Missing or unsupported profile data silently exports.
- Export can overwrite an original.
- Viewer texture cache becomes the export source of truth.
- Visual color correctness language exceeds recorded tolerance and manual review evidence.

## Notes for LLM Agents

Read this brief and the selected task card before implementation. Do not re-plan the whole phase unless a stop gate fires or a current platform/dependency document contradicts the master execution plan.
