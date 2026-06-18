---
title: Public Beta Evidence Index
status: active
audience: maintainers
updated: 2026-06-18
source_of_truth: docs/wiki/roadmaps/post-alpha-master-execution-plan.md
---

# Public Beta Evidence Index

## Verdict

Public beta is **blocked** until SilicaRAW has a signed and notarized DMG, published checksums, and clean-Mac downloaded-artifact QA.

Unsigned developer-preview DMGs may be used for internal testing only. They must not be called public beta releases.

## Frozen Beta Scope

Included in the public beta scope:

- Local macOS app installed from a GitHub Release DMG.
- Local library create/open.
- Import folder by reference without mutating original files.
- Library grid, bounded query, rating, pick, reject, and color label state.
- Stored metadata status display from catalog-owned data.
- Preview/develop workflow for supported local raster paths and fixture-backed proof paths only.
- Develop history, undo/redo, reset, before/after presentation state, supported P0/P1 controls, manual masks, and scoped edit clipboard sync.
- JPEG, PNG, and TIFF local export paths with original-overwrite protection.
- JPEG sRGB default, explicit JPEG Display P3, export metadata policy, batch export progress, and recent export evidence.
- Local preferences, disposable cache controls, backup/recovery evidence, and public trust docs.

Excluded from public beta scope:

- Broad RAW camera support claims.
- Broad visual color correctness claims.
- Product MLX inference runtime, bundled models, or required AI.
- Plugin runtime or arbitrary executable plugins.
- MCP server, background listener, HTTP transport, mutating MCP tools, or permission self-escalation.
- Cloud sync, telemetry, analytics, auto-update, Homebrew distribution, and Mac App Store distribution.
- Unsigned developer-preview DMG as a public beta artifact.

## Extension Status

| Area | Public beta state | Evidence |
| --- | --- | --- |
| MLX | Runtime absent; no bundled models; manifests/results are optional local contracts only. | [MLX](../topics/mlx.md), [Model Manifest Schema](../../../schemas/model_manifest.schema.json) |
| Plugins | Runtime absent; manifests and data-only preset approval exist, plugins remain disabled by default. | [Plugins and MCP](../topics/plugins-and-mcp.md), [Plugin Manifest Schema](../../../schemas/plugin_manifest.schema.json) |
| MCP | Server absent and off by default; read-only manifest validation and internal Core API adapter only. | [ADR 0010](../decisions/adr-0010-mcp-transport-session.md), [26.3 Task Card](../tasks/26.3-read-only-mcp-adapter-core-apis.md) |

## Evidence Matrix

| Gate | Current state | Evidence source | Beta action |
| --- | --- | --- | --- |
| Data trust matrix | Present for action trust, edit history, sidecars, extension evidence, and original-file safety. | [Action Trust](../topics/action-trust.md), [Data Safety](../topics/data-safety.md) | Re-audit in Task 27.1. |
| Original-hash safety | Local workflow and export tests preserve originals; manual QA checklist exists. | [QA Checklist](../../../checklists/QA_CHECKLIST.md), [Photographer Workflow QA](../../../checklists/PHOTOGRAPHER_WORKFLOW_QA.md) | Re-run before beta RC. |
| Dependency/license inventory | Current dependency inventory exists and harness checks it. | [Dependencies Policy](../../DEPENDENCIES.md), `scripts/harness/check.sh` | Re-check before beta RC. |
| Fixture/sample asset licenses | Fixture manifest contract exists; committed sample asset policy remains conservative. | [Fixture Manifest Schema](../../../schemas/fixture_manifest.schema.json), [Installed App Preflight](../../../checklists/INSTALLED_APP_PREFLIGHT.md) | Confirm no unlicensed public sample media ships. |
| Model licenses | No models ship in public beta scope. Any shipped model needs manifest license/source/hash. | [Model Manifest Schema](../../../schemas/model_manifest.schema.json), [MLX](../topics/mlx.md) | Keep models absent unless license manifest exists. |
| Color/export evidence | ICC/export evidence exists; broad visual color correctness remains limited. | [Color Export Manual QA](../../../checklists/COLOR_EXPORT_MANUAL_QA.md), [Golden Image Tolerance Policy](../../../checklists/GOLDEN_IMAGE_TOLERANCE_POLICY.md) | Repeat color/export QA before beta RC. |
| Clean-Mac install QA | Checklist exists; downloaded public beta artifact QA not complete. | [Local DMG Install Checklist](../../../checklists/LOCAL_DMG_INSTALL_CHECKLIST.md), [Local DMG Release Runbook](local-dmg-release-runbook.md) | Must pass on signed/notarized DMG. |
| Signed/notarized artifact | Blocked by Apple Developer Program funding, Developer ID certificate, and notarization credentials. | [Local DMG Release Runbook](local-dmg-release-runbook.md) | Public beta cannot ship until unblocked. |

## Release Language

Allowed public beta wording:

```txt
SilicaRAW is a local-first macOS photo editor beta for non-destructive library, develop, and export workflows.
```

Forbidden public beta wording:

```txt
Broad RAW camera support
Color-correct Lightroom replacement
AI-powered editor
Plugin platform
MCP server
Signed public beta available
```

## Next Step

Task 27.1 should audit this index, fill any missing evidence rows, and decide whether the release path is still blocked or ready for a signed/notarized beta RC.
