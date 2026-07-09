# Local DMG Install Smoke Checklist

Use this checklist to verify that a SilicaRAW DMG installs and runs the local alpha workflow from `/Applications`, not from the build tree or mounted image.

This checklist applies to developer-only unsigned DMGs and later signed/notarized release candidates. Record which kind of artifact is being tested.

## Test Record

- Tester:
- Date:
- Git commit:
- App version:
- Artifact type: `developer unsigned DMG` / `signed notarized DMG`
- DMG path or release URL:
- DMG SHA256:
- macOS version:
- Machine model:
- Chip:
- Memory:
- Network state during offline launch check:
- Result: `pass` / `fail`

## Prerequisites

- [ ] Test machine is Apple Silicon.
- [ ] Test account can write to `/Applications`.
- [ ] DMG exists and matches the recorded commit or release.
- [ ] SHA256 is recorded before mounting.
- [ ] Expected signing state is known:
  - [ ] Developer unsigned/ad-hoc artifact: Gatekeeper warning or manual override is expected and recorded.
  - [ ] Signed/notarized artifact: Gatekeeper must not require command-line quarantine removal.
- [ ] Legal local sample folder is available outside the SilicaRAW library folder.
- [ ] Sample folder contains at least one raster candidate that can export through the current local alpha path.
- [ ] Test output folder is separate from the sample originals folder.
- [ ] Test library folder is empty or disposable.

## Install

- [ ] Mount the DMG.
- [ ] Confirm the mounted image contains `SilicaRAW.app`.
- [ ] Drag `SilicaRAW.app` from the mounted DMG to `/Applications`.
- [ ] Eject the mounted DMG.
- [ ] Launch `/Applications/SilicaRAW.app`.
- [ ] Confirm the app is not running from the mounted image.
- [ ] If the artifact is signed/notarized, confirm Gatekeeper opens the app without command-line quarantine removal.
- [ ] If the artifact is unsigned/ad-hoc, record the exact warning or override used.

## Offline Launch Check

- [ ] Quit SilicaRAW.
- [ ] Disable network access or disconnect from the network.
- [ ] Launch `/Applications/SilicaRAW.app` while offline.
- [ ] Confirm the app reaches the welcome or library-open screen.
- [ ] Confirm no cloud login, telemetry consent, network permission, or remote service is required to start the local workflow.
- [ ] Re-enable network only after the offline launch result is recorded.

## Local Alpha Workflow

- [ ] Create a new local library in the disposable test library folder, or open an existing disposable test library.
- [ ] Confirm the library opens from the installed app.
- [ ] Import the sample folder by reference.
- [ ] Confirm the UI states that original files stay in place or are not modified.
- [ ] Confirm the library grid shows imported catalog rows.
- [ ] Select a photo in the library grid.
- [ ] Apply a rating.
- [ ] Mark a photo as Pick.
- [ ] Mark a photo as Reject.
- [ ] Open the selected photo in the preview/loupe surface.
- [ ] Confirm RAW candidates show a blocked decode state instead of implying RAW decoding is implemented.
- [ ] Switch to Develop.
- [ ] Adjust exposure.
- [ ] Adjust contrast.
- [ ] Commit the Develop edit.
- [ ] Switch to Export.
- [ ] Set an output path in the test output folder.
- [ ] Confirm the output path differs from the referenced original source path.
- [ ] Export JPEG sRGB.
- [ ] Confirm the exported JPEG exists at the output path.
- [ ] Confirm the original source file still exists at its original path.

## Restart Persistence

- [ ] Quit SilicaRAW.
- [ ] Relaunch `/Applications/SilicaRAW.app`.
- [ ] Reopen the same local library.
- [ ] Confirm imported catalog rows are still listed.
- [ ] Confirm rating, Pick, and Reject state persists.
- [ ] Confirm the committed exposure/contrast edit state persists.
- [ ] Confirm the export record or exported state is visible if available in the current UI.

## Quick Original Safety Spot Check

Task 6.2 performs the full original safety QA. For this install smoke test, record a quick spot check:

- [ ] Record a SHA256 hash for at least one source original before import.
- [ ] Record the SHA256 hash for the same original after import, edit, export, restart, and reopen.
- [ ] Confirm the before/after hashes match.
- [ ] Confirm no exported JPEG was written over an original source path.

## Cleanup

- [ ] Eject any mounted DMG.
- [ ] Quit SilicaRAW.
- [ ] Remove or archive the disposable test library folder.
- [ ] Remove or archive the test output folder.
- [ ] Leave `/Applications/SilicaRAW.app` installed only if needed for the next QA pass.

## Failure Notes

Record failures with enough detail to reproduce:

```txt
Step:
Expected:
Actual:
Screenshot or log path:
Blocker severity:
Follow-up issue or PR:
```

## Build-Mac Smoke Record

This is not the clean-Mac result for Task 6.3. It records whether the DMG can be verified, mounted, installed, and launched on the Mac that built it.

Latest local build-machine smoke:

- Date: 2026-07-08
- Git commit: `e0ddfa8919c86930cfc3297d02967214b0a6e5e2`
- Tag: not tagged; local current-branch Q6.2 installed workflow evidence
- Artifact type: `developer unsigned DMG`
- Workflow run: not run for this local inspection
- Artifact: local build output
- Artifact API digest: not applicable
- DMG path: `target/release/bundle/dmg/SilicaRAW_0.1.0_aarch64.dmg`
- DMG SHA256: `cc75a60e35a09410dc681f66ce0b23890ca62d7a4243a5b4b0d9a9829986a266`
- App path launched: `/Applications/SilicaRAW.app`
- macOS version: `26.4`
- Machine: `arm64`
- DMG verification: pass
- DMG mount and app presence: pass
- Mounted app tree hash: `680510cd60a972acd495acea53b15c0c50a0ce51df41ea9e8a7239e9e96b00a1`
- Installed app tree hash matches mounted app: pass
- Installed app tree hash: `680510cd60a972acd495acea53b15c0c50a0ce51df41ea9e8a7239e9e96b00a1`
- Installed-app preflight: launch sub-proof pass; installed executable workflow pass
- GUI launch command: `python3 scripts/harness/installed-app-launch-smoke.py --app /Applications/SilicaRAW.app --output .tmp/q6-installed-app-launch/installed-app-launch-smoke.json`
- Checksum verification: pass
- Unsigned warning file: not generated for this local build-machine inspection
- Smoke reports: `.tmp/q6-installed-workflow/local-dmg-artifact-smoke.json`, `.tmp/q6-installed-workflow/installed-app-launch-smoke.json`, `.tmp/q6-installed-workflow/installed-app-workflow-evidence.json`
- Full installed workflow result: pass for installed executable workflow; WebView click and native path picker automation not covered
- Clean-Mac Task 6.3 result: still pending

## Related References

- [Local DMG Distribution Plan](../docs/wiki/roadmaps/local-dmg-distribution-plan.md)
- [UI MVP Baseline](../docs/wiki/topics/ui-mvp-baseline.md)
- [Data Safety](../docs/wiki/topics/data-safety.md)
