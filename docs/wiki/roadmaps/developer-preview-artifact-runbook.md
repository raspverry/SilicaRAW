---
title: Developer Preview Artifact Runbook
status: active
audience: maintainers
updated: 2026-06-22
source_of_truth: docs/wiki/decisions/adr-0006-unsigned-developer-preview-dmg.md
---

# Developer Preview Artifact Runbook

## Summary

This runbook explains how to build, download, and verify the unsigned SilicaRAW developer-preview DMG from GitHub Actions while Apple Developer Program funding is blocked.

This is not the signed and notarized local distribution path. The developer-preview DMG is not signed, not notarized, and not user-ready. Developer-preview artifacts are for maintainer and contributor testing only. Gatekeeper warnings are expected after download.

## Preconditions

- `main` contains the intended code and documentation.
- CI is passing on `main`.
- The developer-preview workflow exists at `.github/workflows/developer-preview-macos.yml`.
- The artifact is allowed by [ADR 0006](../decisions/adr-0006-unsigned-developer-preview-dmg.md).
- The artifact must be labeled unsigned or developer-preview wherever it is referenced.

Do not use this runbook to publish a user-ready GitHub Release. Signed and notarized release work remains blocked by Phase 7 until Developer ID funding and credentials exist.

## Trigger Options

### Manual Workflow Dispatch

Use this when testing the current `main` branch without creating a tag:

```bash
gh workflow run "Developer Preview macOS DMG" --ref main
```

Then find the run:

```bash
gh run list --workflow "Developer Preview macOS DMG" --limit 5
```

### Developer Preview Tag

Use this when a preview needs a stable ref:

```bash
git switch main
git pull --ff-only
git tag developer-preview-YYYYMMDD.N
git push origin developer-preview-YYYYMMDD.N
```

Use a monotonically increasing suffix for repeated previews on the same day, for example `developer-preview-20260611.1`.

## Watch the Run

```bash
gh run watch RUN_ID --exit-status
```

The run must complete with conclusion `success`:

```bash
gh run view RUN_ID --json conclusion,status,url,workflowName,headBranch,headSha
```

The workflow may emit GitHub-hosted action runtime warnings. Record them if they affect future maintenance, but do not treat a warning-only run as failed unless the job conclusion is not `success`.

## Download Artifact

Download into an ignored scratch directory:

```bash
gh run download RUN_ID --dir .tmp/developer-preview-RUN_ID
```

Expected artifact directory:

```txt
.tmp/developer-preview-RUN_ID/
  silicaraw-unsigned-developer-preview-macos/
    SilicaRAW_VERSION_ARCH.dmg
    SHA256SUMS.txt
    UNSIGNED-DEVELOPER-PREVIEW.txt
```

The artifact must include `UNSIGNED-DEVELOPER-PREVIEW.txt`. If that file is missing, do not share the artifact.

## Verify Checksum

The generated `SHA256SUMS.txt` records the path used in the GitHub Actions workspace. After downloading, compare the checksum value rather than relying on `shasum -c` path matching:

```bash
cat .tmp/developer-preview-RUN_ID/silicaraw-unsigned-developer-preview-macos/SHA256SUMS.txt
shasum -a 256 .tmp/developer-preview-RUN_ID/silicaraw-unsigned-developer-preview-macos/SilicaRAW_*.dmg
```

The two checksum values must match.

## Optional Build-Mac Install Smoke

This smoke test can find packaging mistakes on the build maintainer's Mac. It does not close the clean-Mac QA gate.

```bash
hdiutil verify .tmp/developer-preview-RUN_ID/silicaraw-unsigned-developer-preview-macos/SilicaRAW_*.dmg
mkdir -p /tmp/SilicaRAWDeveloperPreview
hdiutil attach -nobrowse -mountpoint /tmp/SilicaRAWDeveloperPreview .tmp/developer-preview-RUN_ID/silicaraw-unsigned-developer-preview-macos/SilicaRAW_*.dmg
ls /tmp/SilicaRAWDeveloperPreview
hdiutil detach /tmp/SilicaRAWDeveloperPreview
```

For installed-app workflow testing, use [Local DMG Install Smoke Checklist](../../../checklists/LOCAL_DMG_INSTALL_CHECKLIST.md) and record that the artifact is unsigned.

## Evidence Boundary

This runbook proves developer-preview artifact mechanics: workflow execution, artifact download, checksum verification, DMG verification, mount behavior, and optional build-Mac install smoke.

It does not prove user-ready local distribution, signed or notarized behavior, clean-Mac behavior, or normal Gatekeeper acceptance. Browser/static visual QA also cannot be used as a substitute for this runbook because screenshots only prove layout and seeded UI state. Installed app workflow evidence must come from launching the artifact's `SilicaRAW.app`, preferably from `/Applications`, and completing the relevant [Local DMG Install Smoke Checklist](../../../checklists/LOCAL_DMG_INSTALL_CHECKLIST.md) steps against that installed app.

## Required Record

For each useful developer preview, record:

- workflow run URL
- git ref and commit SHA
- artifact name
- artifact digest from the GitHub Actions API, if available
- DMG SHA256
- macOS version and machine model used for any install smoke
- whether Gatekeeper warning behavior was observed
- link to any completed checklist or preflight report

## Do Not

- Do not attach this unsigned artifact to a user-ready GitHub Release.
- Do not call it signed, notarized, Gatekeeper-accepted, or production-ready.
- Do not remove quarantine attributes and then count that as normal user install success.
- Do not add auto-update, Homebrew, Mac App Store, telemetry, cloud sync, MLX, MCP, or plugin runtime work under this runbook.

## Links

- [ADR 0006: Unsigned Developer Preview DMG](../decisions/adr-0006-unsigned-developer-preview-dmg.md)
- [Local DMG Distribution Plan](local-dmg-distribution-plan.md)
- [Signing and Notarization Prep](../../../checklists/SIGNING_NOTARIZATION_PREP.md)
- [Local DMG Install Smoke Checklist](../../../checklists/LOCAL_DMG_INSTALL_CHECKLIST.md)
- [Connected Developer Runtime Smoke](../../../checklists/CONNECTED_RUNTIME_SMOKE.md)

## Notes for LLM Agents

Use this runbook only for the unpaid developer-preview path. If a task asks for user-ready local distribution, first check Phase 7 signing and notarization status.
