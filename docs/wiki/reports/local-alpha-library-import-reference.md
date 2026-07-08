---
title: Local Alpha Library Import Reference Evidence
status: active
audience: maintainers
updated: 2026-07-08
source_of_truth: docs/wiki/roadmaps/local-alpha-quality-closure-plan.md
---

# Local Alpha Library Import Reference Evidence

## Scope

This report records Q5.2 evidence that the local alpha library/import path stores catalog references to original files and preserves original file bytes.

This is developer runtime evidence anchored to the current built `.app` artifact. It does not automate the WebView GUI, native path picker, menu commands, drag-to-`/Applications` install, DMG mount behavior, Gatekeeper acceptance, signed/notarized behavior, offline behavior, or clean-Mac downloaded-artifact behavior.

## Test Record

| Field | Value |
| --- | --- |
| Date | 2026-07-08 |
| Source commit at evidence run | `0b4ed88ad5fe5b21cace524d7293b2c175f43ba4` |
| App artifact path | `target/release/bundle/macos/SilicaRAW.app` |
| App tree SHA256 | `89ab925b523bc2b7943fcfa8ad3318e3266b542edc5d8f7967addf8ff60da95d` |
| Host | `macOS 26.4`, `arm64` |
| Evidence report | `.tmp/q5-library-import-reference/library-import-reference-evidence.json` |
| Evidence report SHA256 | `de61829201df8a361ee119fd45a275e359ed79f25015417d7362bcd10e598369` |
| Fixture manifest | `.tmp/q5-library-import-reference/fixtures/fixture-manifest.json` |
| Fixture manifest SHA256 | `41b0b94feb737b3834addf6742500b0b45856a9e0e90baaea478b2c50652b312` |
| Result | `pass` |

## Command

```bash
python3 scripts/harness/check-library-import-reference-evidence.py \
  --app target/release/bundle/macos/SilicaRAW.app \
  --scratch .tmp/q5-library-import-reference \
  --output .tmp/q5-library-import-reference/library-import-reference-evidence.json
```

The runner creates legal fixtures, records the `.app` artifact hash, runs the exact `silica-desktop` connected runtime smoke, inspects `catalog.db`, and records original SHA-256 values before and after the workflow.

## Evidence Summary

| Check | Result |
| --- | --- |
| Schema version | `12` |
| Library row points to test library root | pass |
| Folder row points to import folder | pass |
| Final catalog photo rows | `6` |
| Catalog paths reference the import folder | pass |
| Catalog paths stay outside the library root | pass |
| Catalog fingerprints match source files | pass |
| Hidden file and package child are not cataloged | pass |
| Photo flags exist for all catalog rows | pass |
| Import action-log payloads match non-recursive and recursive imports | pass |
| Active edit-state source fingerprint matches catalog photo | pass |
| Export settings preserve source path and source SHA-256 | pass |
| Original source hashes unchanged | `8 / 8` pass |

## Catalog Rows

The final catalog rows point at `.tmp/q5-library-import-reference/run/Import Originals`, not copied files inside `.tmp/q5-library-import-reference/run/SilicaRAW Library`.

| File | Type | Unsupported | Reference check |
| --- | --- | --- | --- |
| `reference-urban.jpg` | `jpeg` | false | import-root path, outside library root |
| `reference-still-life.jpeg` | `jpeg` | false | import-root path, outside library root |
| `recursive-child.jpg` | `jpeg` | false | import-root path, outside library root |
| `blocked-raw.DNG` | `unsupported` | true | import-root path, outside library root |
| `notes.txt` | `unsupported` | true | import-root path, outside library root |
| `recursive-notes.txt` | `unsupported` | true | import-root path, outside library root |

The hidden JPEG and `.photoslibrary` package child are tracked for hash safety but remain absent from catalog rows.

## Evidence Boundary

This pass proves:

- `create_library`, `open_library`, and `import_folder` preserve original file bytes in the developer desktop command runtime path.
- `photos.path` stores references to original import-folder files, not library-local copies.
- `photos.full_hash` matches independent SHA-256 checks on the original files.
- Import action-log rows record `catalog_reference` side effects for non-recursive and recursive import.
- Edit and export records preserve source path and source SHA-256 evidence for the selected JPEG.

This pass does not prove:

- manual GUI path picker behavior
- launch from `/Applications`
- install from DMG
- Gatekeeper behavior
- signed or notarized behavior
- offline behavior
- clean-Mac downloaded-artifact behavior

## Next Gate

Q5.3 is recorded in [Local Alpha Review and Edit Persistence Evidence](local-alpha-review-edit-persistence.md), Q5.4 is recorded in [Local Alpha JPEG sRGB Export Evidence](local-alpha-jpeg-export-evidence.md), Q5.5 is recorded in [Local Alpha Trust-State Evidence](local-alpha-trust-state-evidence.md), and Q6.1 is recorded in [Local Alpha Unsigned DMG Inspection](local-alpha-unsigned-dmg-inspection.md). The next gate is Q6.2 install to `/Applications` and launch.
