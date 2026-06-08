# 16 — SilicaRAW Release & Distribution Plan

Status: GO WITH CONDITIONS

## Principle

Never trade user trust for release speed.

## Channels

- Nightly: developer testing, may be unstable, original safe mandatory
- Alpha: selected testers, basic workflows work
- Beta: public testing, no data-loss bugs, signed/notarized preferred
- Stable: signed/notarized/stapled, GitHub Releases + DMG + Homebrew Cask

## Versioning

Semantic versioning:

- 0.1.0-alpha.1
- 0.3.0-beta.2
- 1.0.0-rc.1
- 1.0.0

## macOS Target

Apple Silicon first. Minimum macOS version decided after Tauri/Metal/MLX spikes.

## Artifacts

- DMG
- optional app ZIP
- SHA256 checksums
- release notes
- dependency/license list later

## Signing/Notarization

Public beta/stable should use Developer ID signing and notarization. Gatekeeper verification required before stable.

## Distribution

- GitHub Releases
- DMG
- Homebrew Cask after stable
- Website download later
- Auto-update only after signing/update safety mature

## Privacy

No telemetry by default. No crash uploads by default. Photos/metadata stay local.

## Final Verdict

GO WITH CONDITIONS.

Need exact signing scripts, minimum OS decision, Apple Developer account setup, entitlements review, Homebrew cask draft, updater decision.
