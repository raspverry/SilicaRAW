---
title: Local DMG Release Runbook
status: active
audience: maintainers
updated: 2026-06-11
source_of_truth: docs/wiki/roadmaps/local-dmg-distribution-plan.md
---

# Local DMG Release Runbook

## Summary

This runbook documents the release procedure for SilicaRAW local macOS DMG builds.

The user-ready target is a signed and notarized GitHub Release DMG containing `SilicaRAW.app`. That path is currently blocked until Apple Developer Program funding, a Developer ID Application certificate, and notarization credentials are available.

While that gate is blocked, maintainers may use the unsigned developer-preview artifact path for internal testing only.

Public beta readiness is tracked in the [Public Beta Evidence Index](public-beta-evidence-index.md). Unsigned developer-preview artifacts must not be described as public beta releases.

## Release Types

### User-Ready Local Alpha

- Tag pattern: `v0.1.0-alpha.N`
- Artifact: signed and notarized DMG
- Release location: GitHub Releases
- Gatekeeper behavior: app launches from `/Applications` without command-line quarantine removal
- Status: blocked until Phase 7 signing and notarization is complete

### Unsigned Developer Preview

- Tag pattern: `developer-preview-YYYYMMDD.N`
- Artifact: unsigned DMG from GitHub Actions artifacts
- Release location: workflow artifact only
- Gatekeeper behavior: warnings are expected
- Status: allowed by [ADR 0006](../decisions/adr-0006-unsigned-developer-preview-dmg.md)

Do not attach unsigned developer-preview artifacts to a user-ready GitHub Release.

## Prerelease Checklist

Before creating any local DMG release candidate:

- `main` contains the intended code.
- `scripts/harness/check.sh` passes locally.
- CI passes on `main`.
- [Release Checklist](../../../checklists/RELEASE_CHECKLIST.md) is reviewed.
- [Local DMG Install Smoke Checklist](../../../checklists/LOCAL_DMG_INSTALL_CHECKLIST.md) is ready for execution.
- [QA Checklist](../../../checklists/QA_CHECKLIST.md) has current original-file safety evidence.
- [Release Notes Template](../../../.github/release-template.md) is filled in.
- Known issues and blocked features are listed honestly.
- No original photo files are modified by the tested workflow.

For signed user-ready releases, also require:

- Developer ID Application certificate exists.
- GitHub signing and notarization secrets exist.
- Hardened runtime and entitlements are reviewed.
- Notarization succeeds.
- Stapling succeeds.
- Clean-Mac downloaded-artifact QA passes.

## User-Ready Release Flow

This flow is blocked until Phase 7 and Phase 8 signed release pipeline work is complete.

1. Start from clean `main`:

   ```bash
   git switch main
   git pull --ff-only
   scripts/harness/check.sh
   ```

2. Confirm signing prerequisites:

   ```bash
   python3 scripts/harness/check-signing-prereqs.py
   ```

3. Create a version tag only after the release candidate is approved:

   ```bash
   git tag v0.1.0-alpha.N
   git push origin v0.1.0-alpha.N
   ```

4. Run the signed macOS release workflow when available.

5. Verify signing, notarization, and stapling evidence:

   ```bash
   spctl --assess --type execute --verbose SilicaRAW.app
   spctl --assess --type open --verbose SilicaRAW.dmg
   ```

6. Create a draft GitHub Release using `.github/release-template.md`.

7. Attach:

   - signed and notarized DMG
   - `SHA256SUMS.txt`
   - any required QA evidence links

8. Download the release asset from GitHub and verify checksum:

   ```bash
   shasum -a 256 SilicaRAW_VERSION_ARCH.dmg
   cat SHA256SUMS.txt
   ```

9. Execute the clean-Mac install checklist from the downloaded DMG.

10. Publish only after the downloaded-artifact checklist passes.

## Current Developer Preview Flow

Use this flow while the signed user-ready release path is blocked:

1. Follow [Developer Preview Artifact Runbook](developer-preview-artifact-runbook.md).
2. Verify the artifact checksum.
3. Record the workflow URL, commit SHA, artifact name, DMG SHA256, and known Gatekeeper behavior.
4. Do not publish the artifact as user-ready local distribution.

## Rollback

If a release artifact is bad:

- Move the GitHub Release back to draft, or remove the broken asset.
- Leave a maintainer note with the affected version, artifact name, SHA256, and failure reason.
- Do not reuse the same version tag for different bytes.
- Create a new patch or alpha tag after the fix.
- If a signed release fails Gatekeeper, block the release until signing, notarization, and stapling are fixed.
- If original-file safety fails, block the release until the mutation path is fixed and covered by QA.

## Notarization Troubleshooting Links

- Apple notarization overview: https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution
- Apple common notarization issues: https://developer.apple.com/documentation/security/resolving-common-notarization-issues
- Tauri macOS code signing and notarization: https://v2.tauri.app/distribute/sign/macos/
- GitHub release management: https://docs.github.com/en/repositories/releasing-projects-on-github/managing-releases-in-a-repository

## Notes for LLM Agents

Do not treat this runbook as permission to implement signing, notarization, Homebrew, auto-update, telemetry, cloud sync, MLX, MCP, or plugin runtime. Follow the phase gates in [Local DMG Distribution Plan](local-dmg-distribution-plan.md).
