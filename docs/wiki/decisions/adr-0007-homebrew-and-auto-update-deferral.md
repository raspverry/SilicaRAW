---
title: "ADR 0007: Defer Homebrew and Auto-Update"
status: accepted
audience: all
updated: 2026-06-11
source_of_truth: docs/wiki/roadmaps/local-dmg-distribution-plan.md
---

# ADR 0007: Defer Homebrew and Auto-Update

## Context

SilicaRAW's current delivery target is a local macOS alpha installed from a GitHub-hosted DMG.

The signed and notarized user-ready DMG path is already blocked until Apple Developer Program funding, a Developer ID Application certificate, and notarization credentials are available. While that gate is blocked, the project uses unsigned developer-preview artifacts only.

Homebrew Cask distribution and auto-update both add release operations that are broader than the current local alpha target:

- Homebrew requires stable release assets, checksums, versioning discipline, and maintenance of a public cask update flow.
- Auto-update requires signed update artifacts, update-channel policy, rollback behavior, user trust decisions, and a security review.
- Both paths can make a broken or unsigned local alpha look more user-ready than it is.

## Decision

SilicaRAW will defer Homebrew Cask distribution and auto-update until after the local DMG alpha path is proven.

For the current local alpha:

- The primary distribution target remains a GitHub Release DMG.
- Unsigned developer-preview DMGs remain maintainer/contributor artifacts only.
- No Homebrew Cask files, tap setup, updater runtime, updater server, update feed, or updater dependency should be added.
- Release documentation may mention Homebrew and auto-update only as deferred future work.

## Revisit Criteria

Revisit Homebrew Cask only after:

- A signed and notarized GitHub Release DMG exists.
- Release checksums and release notes are published consistently.
- Clean-Mac install QA has passed from a downloaded release artifact.
- The project has a stable version/tag policy and rollback process.
- A maintainer is ready to maintain the cask update path.

Revisit auto-update only after:

- Signed and notarized release artifacts are repeatable.
- The app has a clear update-channel policy.
- Update checks can be implemented without telemetry or unexpected network behavior.
- Failure, rollback, and downgrade behavior are specified.
- The selected updater mechanism has a security review and documented dependency policy.

## Consequences

- Local alpha release work stays focused on the simplest installable artifact: a DMG.
- Maintainers avoid shipping update infrastructure before the app is ready for ordinary users.
- Contributors have a clear rule: do not add updater or Homebrew behavior while local alpha release gates are still open.
- Future distribution work has explicit prerequisites instead of being reopened ad hoc.

## Alternatives Considered

- Add Homebrew Cask now: rejected because unsigned or unstable alpha artifacts should not be made easier to install as if they were user-ready.
- Add auto-update now: rejected because update security and rollback behavior require signed, repeatable releases first.
- Ignore Homebrew and auto-update entirely: rejected because both may be reasonable after local DMG alpha, but only after release trust gates are met.

## Links

- [Local DMG Distribution Plan](../roadmaps/local-dmg-distribution-plan.md)
- [ADR 0002: Local DMG Distribution](adr-0002-local-dmg-distribution.md)
- [ADR 0006: Unsigned Developer Preview DMG](adr-0006-unsigned-developer-preview-dmg.md)
- [Release Checklist](../../../checklists/RELEASE_CHECKLIST.md)

## Notes for LLM Agents

Do not implement Homebrew, auto-update, updater feeds, updater dependencies, or background network update checks under local alpha tasks. If a future task asks for any of these, first verify that the revisit criteria in this ADR are satisfied or that a newer ADR supersedes this one.
