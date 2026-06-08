---
title: "ADR 0002: Local DMG Distribution Target"
status: accepted
audience: all
updated: 2026-06-08
source_of_truth: docs/wiki/roadmaps/local-dmg-distribution-plan.md
---

# ADR 0002: Local DMG Distribution Target

## Context

SilicaRAW's first meaningful delivery target is local macOS distribution. The intended user experience is downloading a DMG from GitHub, installing `SilicaRAW.app`, and completing a minimal local editor workflow.

On macOS, `.app` is the executable application bundle. `.dmg` is the distribution container that carries the app for installation.

## Decision

The local alpha distribution target is:

```txt
GitHub Release
  -> signed and notarized SilicaRAW.dmg
    -> signed SilicaRAW.app
```

Unsigned DMGs may be produced for developer-only testing, but they must not be described as user-ready.

## Consequences

- Release planning must include app bundle metadata, signing, notarization, stapling, checksums, and release notes.
- A DMG is not enough by itself; the installed app must complete the local alpha workflow.
- Gatekeeper acceptance is required before calling a GitHub DMG user-ready.

## Alternatives Considered

- Raw `.app` ZIP: useful as an optional artifact later, but weaker as the primary install experience.
- Homebrew Cask: deferred until after local alpha.
- Mac App Store: out of scope for local alpha.
- Unsigned DMG only: acceptable for developer testing, not for user-ready local distribution.

## Links

- [Local DMG Distribution Plan](../roadmaps/local-dmg-distribution-plan.md)
- [Release and Distribution Plan](../../16_Release_Distribution_Plan.md)
- Tauri distribution docs: https://v2.tauri.app/distribute
- Apple Developer ID: https://developer.apple.com/developer-id/
- Apple notarization docs: https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution

## Notes for LLM Agents

Do not treat an unsigned developer DMG as the final local distribution target. The user-ready target requires a signed/notarized app inside a signed/notarized DMG or equivalent Gatekeeper-accepted artifact.

