---
title: Public Trust
status: active
audience: all
updated: 2026-06-11
source_of_truth: README.md
---

# Public Trust

## Summary

This page records what SilicaRAW can publicly claim today and what remains unproven. It is written for contributors, users evaluating an early alpha, and LLM agents that need accurate project boundaries.

## Current Trust Package

- Project source code and documentation are licensed under the [MIT License](../../../LICENSE).
- Third-party dependency and license status is tracked in [Dependencies Policy](../../DEPENDENCIES.md).
- Data-safety boundaries are tracked in [Data Safety](data-safety.md) and [Backup and Restore](backup-restore.md).
- Distribution status is tracked in the [Local DMG Distribution Plan](../roadmaps/local-dmg-distribution-plan.md), [Local DMG Release Runbook](../roadmaps/local-dmg-release-runbook.md), and [Developer Preview Artifact Runbook](../roadmaps/developer-preview-artifact-runbook.md).

## Allowed Public Claims

- SilicaRAW is an early-stage, open-source Apple Silicon-first photo editor project.
- The local alpha target is a local macOS app installed from a DMG.
- Current local-alpha workflow support is limited to fixture-backed JPEG/JPG import, culling flags, preview/develop state, exposure/contrast persistence, JPEG sRGB export, cache cleanup, backup artifacts, and staged restore boundaries.
- Original photo files are referenced by path and must not be modified by SilicaRAW.
- The app has no default telemetry, cloud sync, network upload, auto-update, plugin runtime, MCP server, or MLX runtime in the local alpha scope.

## Claims Not Yet Allowed

- Do not describe SilicaRAW as production-ready.
- Do not claim broad RAW camera support.
- Do not claim color correctness until tagged fixtures and tolerance checks exist.
- Do not claim the product Metal viewer is implemented.
- Do not claim MLX, MCP, plugins, Homebrew, auto-update, Mac App Store distribution, or public beta readiness.
- Do not treat unsigned developer-preview DMGs as user-ready releases.

## Release Status

The intended user-ready local distribution path is a signed and notarized GitHub Release DMG containing `SilicaRAW.app`.

That path is blocked until Apple Developer Program funding, a Developer ID Application certificate, and notarization credentials are available.

Unsigned developer-preview DMGs may be built for internal testing only. Gatekeeper warnings are expected, and those artifacts are not user-ready local distribution.

## Dependency and Asset Status

- `docs/DEPENDENCIES.md` is the current dependency and third-party license inventory.
- No model weights are bundled.
- No redistributable RAW/color fixture assets are committed as product fixtures.
- Future model weights, sample assets, binary tools, or bundled runtime components need explicit source, license, rights, and hash records before they are committed or shipped.

## Remaining Task 10.6 Work

Task 10.6.2 still needs the public contribution guide, security policy, issue templates, PR template updates, and static checks that keep the trust package from regressing.

## Notes for LLM Agents

Use this page to keep public statements honest. If a feature is only planned, deferred, blocked, or spike-only, describe it that way.
