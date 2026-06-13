---
title: Post-Alpha Master Execution Plan
status: active
audience: all
updated: 2026-06-13
source_of_truth: docs/wiki/roadmaps/post-alpha-product-roadmap.md
---

# Post-Alpha Master Execution Plan

## Summary

This page is the execution router for Phase 14 through v1.0.

It exists so maintainers and agents do not re-plan every phase from scratch. The [Post-Alpha Product Roadmap](post-alpha-product-roadmap.md) remains the scope source of truth. This master plan controls sequence, dependency order, stop gates, and known task splits that must be applied when creating future task cards.

## Operating Rule

- Use this page before choosing any Phase 14 or later task.
- Use the active phase plan, phase brief, and task card for the actual implementation details.
- If a future phase does not yet have task cards, create those cards from this master plan in a docs-only task, then implement the first card.
- Do not create a new phase-wide planning document unless this page marks an unresolved design gate.
- Update this page only when a gate, dependency, or task split changes.

Allowed reasons to create a new phase-specific plan:

- A current official API, platform rule, or dependency document contradicts the existing plan.
- A schema or crate boundary change needs a durable design decision before code.
- A release/signing/security requirement changes the delivery path.
- MLX, plugin, or MCP work is being started and needs a runtime ADR.

## Current Position

As of 2026-06-13:

- Phase 12 RAW proof is complete for fixture-backed Core Image support mapping.
- Phase 13 color proof is complete for ICC/profile evidence, but visual color correctness remains blocked pending tolerance and manual review.
- Phase 14 is complete.
- Task 14.0 through Task 14.8 are complete.
- Phase 15 and Phase 16 are complete.
- Task 15.0 evidence gate is complete.
- Task 15.1 decoded image handoff contract is complete.
- Task 15.2 RAW decode to preview artifact is complete.
- Task 15.3 Metal preview display is complete.
- Task 15.4 Exposure/contrast Metal draft path is complete.
- Task 15.5 RAW-derived JPEG sRGB export is complete.
- Task 15.6 RAW export manual color QA is complete.
- Phase 15 is complete.
- Task 16.0 Phase 16 design gate is complete.
- Task 16.1 Undo, History, and Action Semantics Contract is complete.
- Task 16.2 Edit History Persistence is complete.
- Task 16.3 Undo and Redo Core Commands is complete.
- Task 16.4 Develop History Panel Contract is complete.
- Task 16.5 Append-Only Action Log is complete.
- Task 16.6 Sidecar Sync Status After History Commits is complete.
- Phase 16 is complete.
- Tasks 17.1.1 through 17.5 are complete.
- Phase 17 is complete.
- Phase 18 task cards exist. Tasks 18.1.1 through 18.5.1 are complete. Current next task is [Task 18.5.2 Batch Sync History](../tasks/18.5.2-batch-sync-history.md).

## Wave Map

| Wave | Scope | Entry Gate | Exit Gate |
| --- | --- | --- | --- |
| A | Phase 14: Native Viewer Bridge | Spike 001 Path B, Phase 12/13 proof context | Feature-gated AppKit/Metal bridge, reserved viewer rect, lifecycle/input/QA evidence |
| B | Phase 15: First RAW/Color/Metal Product Vertical | Phase 14 complete, RAW fixture manifest, color fixture manifest | Fixture-backed RAW preview, exposure/contrast Metal draft, JPEG sRGB export with ICC, original hashes unchanged |
| C | Phase 16: Non-Destructive Trust Substrate | Phase 15 complete | Undo/history/action log semantics, transaction-safe undo/redo, sidecar sync status |
| D | Phases 17, 18, 20.1, 20.4, 21.1-21.4: Editor Core Expansion | Phase 16 complete | Develop P0/P1 controls, export settings, batch export, preferences, preview/export parity evidence |
| E | Phase 19: Manual Masks | Phase 18 core edit behavior stable | Manual mask schema, preview/export mask parity, durable mask data vs disposable cache split |
| F | Phase 20.2, 20.3, 20.5: Export Format and Metadata Expansion | Export settings model, color proof, relevant editor controls | PNG/TIFF, metadata policy, explicit P3 expansion, no silent color/default changes |
| G | Phase 22: Continuous Hardening | Embedded after each relevant surface | Visual QA, scale reports, migration/backup failure tests, performance evidence, manual photographer QA |
| H | Phase 23 plus Task 21.5: Permission Foundation | Phase 16 action log | Default-deny permission policy, prompt contract, action log integration, advanced access UI off by default |
| I | Phases 24, 25, 26: Extension Lab | Phase 23 complete, editor core trustworthy | Optional/off-by-default MLX, plugin, and MCP paths without direct mutation or raw SQL |
| J | Phase 27: Public Beta Gate | Beta scope cut and evidence package complete | Signed/notarized beta RC only when Developer ID funding and secrets exist |
| K | Phase 28: v1.0 Gate | Public beta feedback and v1.0 scope freeze | Stability matrix and signed/notarized v1.0 release |

## Dependency Graph

```txt
Phase 14
  -> Phase 15
  -> Phase 16
  -> Phase 17
  -> Phase 18
  -> Phase 19
  -> Phase 27 / Phase 28

Phase 17
  -> Task 20.1 / Task 20.4
  -> Task 21.1 through Task 21.4

Task 20.1
  -> Task 20.2 / Task 20.3
  -> Task 22.3
  -> Task 21.4

Task 16.5 + Phase 23
  -> Task 21.5
  -> Phase 24 / Phase 25 / Phase 26

Task 19.4 + Task 24.4
  -> Task 24.5

Phase 22
  -> embedded across Waves D through K, not saved for the end
```

## Locked Task Refinements

These refinements come from architecture, render, RAW/color/export, storage, and release reviews. They supersede coarse grouping in the roadmap when future task cards are created.

### Phase 14: Native Viewer Bridge

Phase 14 already has task cards 14.0 through 14.8. Keep that sequence.

Do not merge Phase 14 with Phase 15. Phase 14 proves bridge ownership and viewer safety only. RAW pixels, color correctness, and exposure/contrast Metal rendering start after the bridge is proven.

Hard gates:

- `metal_host_spike.rs` stays proof code.
- Product viewer code is feature-gated behind `native-metal-viewer`.
- The native view consumes a reserved viewer rectangle, not arbitrary web overlay space.
- Native input cannot steal clicks, drag, scroll, or magnify events outside the viewer.
- Product bridge code must build with and without the feature.

### Phase 15: RAW, Color, and Metal Vertical Slice

Use this finer split instead of the current coarse roadmap grouping:

| Task | Name | Key Gate |
| --- | --- | --- |
| 15.0 | Vertical Slice Evidence Gate | RAW Classes A-D fixture evidence, Color Class F evidence, no broad RAW/color-correctness claim |
| 15.1 | Decoded Image Handoff Contract | dimensions, orientation, source SHA, decoder backend, input profile, working space, cache identity, pixel format, blocked states |
| 15.2 | RAW Decode to Preview Artifact | Core Image creates bounded disposable preview artifacts under library cache; unsupported RAW stays blocked |
| 15.3 | Metal Preview Display | preview artifact becomes native viewer texture; latest request wins; web controls remain usable |
| 15.4 | Exposure/Contrast Metal Draft Path | slider drafts update preview without DB/history writes; commit writes one validated edit graph |
| 15.5 | RAW-Derived JPEG sRGB Export | full-res export path separate from preview cache; sRGB ICC embedded; output/source/ICC hashes recorded |
| 15.6 | RAW Export Manual Color QA | Preview.app or Photos review record exists; release language remains evidence-limited |

Required fixtures:

- RAW Class A minimum.
- RAW Class C or D recommended for higher-risk path coverage.
- Color Class F for ICC/profile regression.

Stop if:

- RAW support is inferred from extension alone.
- Missing or unsupported profile data silently exports.
- Export path can overwrite an original.
- Visual color correctness language exceeds recorded tolerance and manual review evidence.

### Phase 16: Undo, History, and Action Trust

Treat Phase 16 as a hard trust gate before more Develop breadth.

Required split:

- 16.0: Design gate for action classes, schema boundary, transaction policy, and sidecar policy.
- 16.1: Undo, history, and action semantics contract.
- 16.2: Edit history persistence with migration and idempotence tests.
- 16.3: Undo/redo core commands as catalog transactions.
- 16.4: History panel backed by real checkpoints only.
- 16.5: Append-only action log through Core APIs.
- 16.6: Sidecar sync status after history commits.

Stop if undo/redo can bypass catalog transactions, if history rows can reference invalid edit graphs, or if extensions can write sensitive actions outside Core APIs.

### Phase 17: Develop P0 Expansion

Do not bundle all controls into one implementation task. Split by control family so preview/export parity can fail narrowly.

Required control slices:

- White balance, temperature, and tint.
- Highlights, shadows, whites, and blacks.
- Vibrance and saturation.
- Preview/export parity for each enabled family.
- Histogram, reset, before/after, and basic presets only after the mutator/render/export behavior is clear.

Each control family must validate schema ranges, render preview where supported, commit one undoable checkpoint, and export through the same committed semantics.

### Phase 18: Professional Editing Baseline

Keep Phase 18 as vertical slices:

- Tone curve.
- HSL and color mixer.
- Detail baseline.
- Lens, geometry, crop, and rotate.
- Copy/paste edits and batch sync.

Every slice must specify edit graph validation, preview support, export support, undo behavior, and unsupported states. Batch sync must record history per affected photo and preserve original hashes.

### Phase 19: Manual Masks

Phase 19 remains manual-first. AI masks wait until permissions, history, and mask provenance are ready.

Hard gate:

- Durable mask data and disposable mask/render caches must be separate.
- Mask preview and export must agree within documented tolerance before mask claims broaden.
- AI-generated mask provenance must be separate from manual mask provenance.

### Phase 20: Export and Delivery Expansion

Before adding PNG, TIFF, metadata controls, or additional color-space behavior, create an export matrix:

```txt
format: JPEG / PNG / TIFF
color space: sRGB / Display P3
metadata: preserve / remove GPS / remove all
source: JPEG/JPG / fixture-backed RAW
support state: supported / blocked / deferred
```

Unsupported combinations must be blocked, not guessed.

Display P3 note:

- Phase 13.8 already enabled explicit ICC-backed Display P3 JPEG export proof.
- Phase 20.5 is therefore export expansion and verification, not first enablement.
- Display P3 must never become an accidental default.

### Phase 21: Preferences and App Settings

Task 21.1 through 21.4 can proceed with editor/export settings. Task 21.5 belongs with Wave H because advanced agent access depends on Phase 23 permission policy.

Cache preferences must remain limited to disposable cache directories and must never expose original-file operations.

### Phase 22: Continuous Hardening

Phase 22 is not a late cleanup phase. Embed it across the editor waves.

Required placement:

- 22.1: run after each new UI surface or major UI state.
- 22.2: eligible after Task 11.5; rerun after large catalog or grid changes. 50k reports are evidence artifacts, not normal CI.
- 22.3: run before beta after history, export settings, and migration-sensitive state exist.
- 22.4: start after Task 15.3 for decode/render/slider timing evidence; rerun before performance claims.
- 22.5: required before public beta.

Evidence artifacts must include machine metadata, dataset shape, known limitations, and whether results are automated or manual.

### Phase 23: Permission and Audit Foundation

Phase 23 is policy and audit first, runtime later.

Required rules:

- Default deny.
- No raw SQL permission.
- No plugin or MCP runtime before permission policy and action logging exist.
- Permissioned actors go through Core APIs.
- Mutating tool permission is out of scope unless a future ADR approves it.

Task 21.5 advanced agent access preferences belongs after this policy is in place and must not start a server or runtime.

### Phases 24, 25, and 26: Extension Lab

Keep MLX, plugin, and MCP as optional, disabled-by-default extension tracks.

Phase 24:

- Runtime spike first.
- Model manifest validation before model use.
- AI results stored separately from edit graph and catalog flags.
- First feature is non-mutating review.
- Mutating AI approval waits for masks, history, and permissions.

Phase 25:

- Declarative plugin manifest first.
- Data-only preset plugins before executable plugin models.
- Plugin applies require explicit approval and action logging.

Phase 26:

- MCP transport/session ADR first.
- Read-only tool manifests before adapters.
- Read-only adapter calls go through Core APIs.
- No mutating MCP tools in the first MCP phase.

### Phase 27: Public Beta Gate

Add Task 27.0 before the existing audit:

- Freeze beta scope.
- List included and excluded features.
- Confirm whether MLX, plugins, or MCP are disabled, hidden, or absent.

Task 27.1 must produce a public beta readiness evidence index, not only an audit note.

Evidence index must cover:

- Data trust matrix.
- Original-hash safety results.
- Dependency and license inventory.
- Fixture and sample asset license records.
- Model license records if models ship.
- Color/export evidence and known limitations.
- Clean-Mac install QA.
- Signed/notarized artifact readiness or explicit funding block.

Public beta cannot use an unsigned developer-preview DMG.

### Phase 28: v1.0 Gate

Phase 28 starts after beta feedback and a v1.0 scope freeze.

Stop if any unresolved S0/P0 data-loss, edit-loss, migration, restore, color/export, viewer, or install bug remains.

v1.0 release requires:

- Stability matrix.
- Clean install verification.
- Signed, notarized, stapled DMG.
- Checksums.
- Release notes with supported formats, known issues, privacy posture, and limitations.

## Stop-Gate Register

| Gate | Blocks | Trigger |
| --- | --- | --- |
| Original file safety | All phases | Any task can edit, delete, move, or overwrite originals without explicit approved scope |
| RAW evidence | Phase 15+ RAW claims | RAW support inferred from extension, missing manifest, fixture hash mismatch, unsupported RAW showing pixels |
| Color evidence | Export and color claims | ICC/profile proof is confused with visual correctness, tolerance/manual review missing |
| Native viewer safety | Phase 15 | Overlay covers web UI, input leaks, unsafe AppKit/Metal lifecycle, no fallback path |
| History trust | Phase 17+ | Undo/redo bypasses transactions, invalid edit graph history, draft writes persist |
| Export safety | Phase 20+ | Output path can overwrite source, unsupported format/color/metadata combination guessed |
| Extension permission | Phase 24-26 | Runtime starts before permission/action log, raw SQL access, mutation without explicit approval |
| Release trust | Phase 27-28 | Unsigned artifact called public beta or v1.0, missing checksums, Gatekeeper fail, clean install QA missing |

## Validation Matrix

Use the smallest useful verification for the active task. Avoid broad slow checks when a narrow check proves the change. Always run the full harness before claiming a normal task complete unless the task documents a narrower reason.

Default:

```bash
scripts/harness/check.sh
```

Docs:

```bash
python3 scripts/harness/check-md-links.py
```

RAW and color fixtures:

```bash
SILICARAW_RAW_FIXTURE_MANIFEST=/path/to/legal-raw-fixtures.json scripts/harness/check-raw-probe-fixtures.py
SILICARAW_COLOR_FIXTURE_MANIFEST=/path/to/legal-color-fixtures.json scripts/harness/check-color-probe-fixtures.py
```

Native viewer:

```bash
cargo check -p silica-desktop
cargo check -p silica-desktop --features native-metal-viewer
cargo test -p silica-desktop --features native-metal-viewer
```

Core editor/export paths:

```bash
cargo test -p silica-edit -p silica-render -p silica-export -p silica-core
```

Storage trust:

```bash
cargo test -p silica-storage backup
cargo test -p silica-storage restore
cargo test -p silica-storage -p silica-core
```

Release evidence, when signing is funded and secrets exist:

```bash
python3 scripts/harness/check-signing-prereqs.py
spctl --assess --type open --verbose SilicaRAW.dmg
spctl --assess --type execute --verbose /Applications/SilicaRAW.app
shasum -a 256 SilicaRAW.dmg
```

## Current Next Tasks

1. Start [Task 18.5.2 Batch Sync History](../tasks/18.5.2-batch-sync-history.md).
2. Continue Phase 18 in task-card order after each slice passes focused checks and the full harness.

## Links

- [Post-Alpha Product Roadmap](post-alpha-product-roadmap.md)
- [Phase 14 Product Metal Viewer Bridge Plan](phase-14-metal-viewer-bridge-plan.md)
- [Phase 14 Product Metal Viewer Bridge Brief](../phases/phase-14-product-metal-viewer-bridge.md)
- [Phase 15 RAW Color Metal Vertical Slice Brief](../phases/phase-15-raw-color-metal-vertical-slice.md)
- [Task Cards](../tasks/index.md)
- [Roadmap Overview](../overview/roadmap.md)
- [RAW Decoding](../topics/raw-decoding.md)
- [Color Management](../topics/color-management.md)
- [Metal Rendering](../topics/metal-rendering.md)
- [Edit Graph](../topics/edit-graph.md)
- [Data Safety](../topics/data-safety.md)
- [Public Trust](../topics/public-trust.md)
- [Plugins and MCP](../topics/plugins-and-mcp.md)
- [Edit Graph Schema](../../../schemas/edit_graph.schema.json)
- [Development Roadmap](../../13_Development_Roadmap.md)

## Notes for LLM Agents

This page is the durable plan for Phase 14 through v1.0. Read it once when choosing work, then switch to the smallest active task card and topic pages. Do not re-open broad planning unless a stop gate above is hit.
