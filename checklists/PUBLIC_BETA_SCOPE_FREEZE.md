# Public Beta Scope Freeze

Status: blocked until signed/notarized DMG prerequisites exist.

## Required Scope Decision

- [x] Public beta requires signed and notarized DMG.
- [x] Unsigned developer-preview DMG is internal testing only.
- [x] MLX runtime and bundled models are excluded.
- [x] Plugin runtime is excluded.
- [x] MCP server/runtime is excluded.
- [x] Cloud sync, telemetry, auto-update, Homebrew, and Mac App Store distribution are excluded.
- [x] Broad RAW camera support claims are excluded.
- [x] Broad visual color correctness claims are excluded.

## Required Evidence Before Beta RC

- [ ] `scripts/harness/check.sh` passes on `main`.
- [ ] Dependency/license inventory is current.
- [ ] Fixture and sample asset license review is current.
- [ ] Model license manifests exist if any model ships.
- [ ] Original-file safety QA is current.
- [ ] Color/export QA is current.
- [ ] Clean-Mac downloaded-DMG install QA passes.
- [ ] Gatekeeper accepts installed `.app`.
- [ ] Gatekeeper accepts downloaded `.dmg`.
- [ ] SHA256 checksums are published.

## Current Block

Public beta is blocked by missing Apple Developer Program funding, Developer ID certificate, and notarization credentials.
