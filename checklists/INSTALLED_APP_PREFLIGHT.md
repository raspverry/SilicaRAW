# Installed-App Developer Preflight

Use this before clean-Mac DMG QA to record local evidence from a developer-built app artifact.

This checklist does not replace clean-Mac install QA. It creates legal synthetic fixtures, verifies their hashes, and records the app artifact and host details in an ignored JSON report.

## Generate Fixtures

```bash
python3 scripts/harness/generate-legal-fixtures.py \
  --output .tmp/legal-qa-fixtures \
  --include-raw-placeholders
```

Expected output:

- `fixture-manifest.json`
- `supported/*.jpg`
- `supported/*.jpeg`
- `unsupported/*`
- `raw-blocked/*` when `--include-raw-placeholders` is used

## Record Preflight

Replace the app path with the local `.app` artifact being checked.

```bash
python3 scripts/harness/installed-app-preflight.py \
  --app apps/desktop/src-tauri/target/debug/bundle/macos/SilicaRAW.app \
  --fixtures .tmp/legal-qa-fixtures \
  --output .tmp/installed-app-preflight/installed-app-preflight.json
```

The JSON report records:

- app artifact path, kind, size, file count, and SHA-256 digest
- host platform and macOS version field
- fixture path and manifest summary
- expected versus actual fixture hash results
- known local-alpha limitations for RAW decode, Metal viewer output, AI tools, and clean-Mac QA

## Manual Sign-Off

- Fixture manifest path:
- App artifact path:
- Preflight report path:
- macOS version:
- Hash results all pass:
- Known limitations reviewed:
- Follow-up needed before clean-Mac QA:

## Guardrails

- Do not commit generated fixture output or preflight reports.
- Do not use user photos as fixture inputs for this generator.
- RAW-blocked placeholders are text fixtures, not decodable camera RAW files.
- Keep this developer preflight separate from the later DMG install checklist.
