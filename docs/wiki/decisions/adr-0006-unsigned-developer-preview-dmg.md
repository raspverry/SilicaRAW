---
title: "ADR 0006: Unsigned Developer Preview DMG"
status: accepted
audience: all
updated: 2026-06-11
source_of_truth: docs/wiki/roadmaps/local-dmg-distribution-plan.md
---

# ADR 0006: Unsigned Developer Preview DMG

## Context

SilicaRAW's intended local distribution target remains a signed and notarized GitHub Release DMG. That target needs Apple Developer Program access, a Developer ID Application certificate, and notarization credentials.

The project does not currently have funding for Apple Developer Program membership. Phase 7.1 therefore cannot complete now. Blocking all release pipeline work on paid credentials would delay useful local-alpha testing.

## Decision

SilicaRAW will use an unsigned developer-preview DMG path until Developer ID funding is available.

For this temporary path:

- GitHub Actions may build an unsigned `.app` and `.dmg`.
- Artifacts must be labeled as developer preview or unsigned.
- Release notes and checklists must state that Gatekeeper warnings are expected.
- The artifact must not be described as user-ready, signed, notarized, or Gatekeeper-accepted.
- The signed/notarized target remains the final local distribution goal.

## Consequences

- Contributors can test the DMG workflow without Apple Developer Program cost.
- Users may need right-click Open or Privacy & Security approval to run downloaded builds.
- Clean-Mac install QA can record unsigned behavior, but it does not satisfy the signed/notarized Gatekeeper acceptance gate.
- Phase 7 resumes when a Developer ID Application certificate and required GitHub secrets are available.

## Alternatives Considered

- Buy Apple Developer Program membership now: rejected because funding is not available.
- Stop release pipeline work: rejected because unsigned developer-preview artifacts still help validate packaging.
- Claim unsigned DMGs as local distribution complete: rejected because Gatekeeper can warn or block downloaded unsigned apps.

## Links

- [Local DMG Distribution Plan](../roadmaps/local-dmg-distribution-plan.md)
- [Signing and Notarization Prep](../../../checklists/SIGNING_NOTARIZATION_PREP.md)
- [Local DMG Install Checklist](../../../checklists/LOCAL_DMG_INSTALL_CHECKLIST.md)
- Tauri macOS signing: https://v2.tauri.app/distribute/sign/macos/
- Apple Developer ID certificates: https://developer.apple.com/help/account/certificates/create-developer-id-certificates/

## Notes for LLM Agents

Do not add signing, notarization, updater, Homebrew, or Mac App Store behavior under this ADR. This ADR permits only unsigned developer-preview artifacts and honest documentation while the paid Developer ID path is blocked.
