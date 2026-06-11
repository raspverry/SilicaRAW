# SilicaRAW VERSION

## Release Type

- [ ] Signed and notarized local alpha DMG
- [ ] Unsigned developer-preview artifact

Do not publish an unsigned developer-preview artifact as user-ready local distribution. If this release is unsigned, mark it as a pre-release and state that Gatekeeper warnings are expected.

## Downloads

- `SilicaRAW_VERSION_ARCH.dmg`
- `SHA256SUMS.txt`

## Install

1. Download the DMG and checksum file.
2. Verify the checksum before opening the DMG.
3. Open the DMG.
4. Drag `SilicaRAW.app` to `/Applications`.
5. Launch `SilicaRAW.app` from `/Applications`.

Unsigned developer-preview builds may require right-click Open or approval in System Settings. Signed and notarized local alpha releases should launch without command-line quarantine removal.

## Verify Checksum

```bash
shasum -a 256 SilicaRAW_VERSION_ARCH.dmg
cat SHA256SUMS.txt
```

The checksum values must match.

## Local Alpha Workflow Covered

- Create or open a local library.
- Import a folder by reference.
- Show the library grid.
- Rate, pick, or reject photos.
- Open a preview.
- Apply exposure and contrast.
- Persist edit state.
- Export JPEG sRGB.
- Verify original files are unchanged.

## Known Issues

- RAW decoding is not implemented in the local alpha path.
- The product Metal viewer is not implemented yet.
- MLX, MCP, plugin runtime, cloud sync, telemetry, auto-update, Homebrew, and Mac App Store distribution are not included.
- Unsigned developer-preview artifacts can trigger Gatekeeper warnings.

## Privacy

SilicaRAW local alpha stores libraries, caches, edit state, and exports on the user's Mac. The app must not upload photos, metadata, edits, analytics, telemetry, or crash data. Imported photos are referenced in place and original photo files must not be modified.

## Compatibility

- macOS: VERSION_OR_RANGE
- CPU: Apple Silicon
- Artifact status: signed/notarized or unsigned developer-preview

## QA Evidence

- CI run:
- Release workflow run:
- DMG SHA256:
- Local DMG install checklist:
- Original-file safety evidence:
- Clean-Mac QA record:

## Rollback

If the release artifact is broken, mark the release as draft or remove the affected asset. Do not replace a signed/notarized release with an unsigned artifact without clearly changing release status and notes.
