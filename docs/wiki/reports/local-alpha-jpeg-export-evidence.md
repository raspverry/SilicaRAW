---
title: Local Alpha JPEG sRGB Export Evidence
status: active
audience: maintainers
updated: 2026-07-08
source_of_truth: docs/wiki/roadmaps/local-alpha-quality-closure-plan.md
---

# Local Alpha JPEG sRGB Export Evidence

## Scope

This report records Q5.4 evidence that the local desktop runtime exports a selected supported raster photo to a separate JPEG sRGB artifact and preserves the original source file.

It does not prove the WebView GUI export dialog, `/Applications` install, DMG behavior, Gatekeeper acceptance, signed/notarized behavior, offline behavior, or clean-Mac downloaded-artifact behavior.

## Test Record

| Field | Value |
| --- | --- |
| Date | 2026-07-08 |
| Source commit at evidence run | `8226e99e3127c72590209df853fd6bf5d849e190` |
| App artifact path | `target/release/bundle/macos/SilicaRAW.app` |
| Evidence command | `scripts/harness/check-library-import-reference-evidence.py` |
| Evidence report | `.tmp/q5-jpeg-export-evidence/library-import-reference-evidence.json` |
| Evidence report SHA256 | `53cfa0e8c039dde52b098e7e1ff77e9ab12b9315a9e80ae589e28f3d09b083bd` |
| Catalog DB | `.tmp/q5-jpeg-export-evidence/run/SilicaRAW Library/catalog.db` |
| Catalog DB SHA256 | `f4408404ec077fd36c59355cf516b18c32bf65226a56630b081bd1190feb9633` |
| Export output | `.tmp/q5-jpeg-export-evidence/run/Exports/reference-urban-export.jpg` |
| Export output SHA256 | `f8d6036a6c2f76ebbaad6ad4573b414f40af4b69a13890a22d17a243c6e16f3d` |
| Result | `pass` |

## Commands

```bash
python3 scripts/harness/check-library-import-reference-evidence.py \
  --app target/release/bundle/macos/SilicaRAW.app \
  --scratch .tmp/q5-jpeg-export-evidence \
  --output .tmp/q5-jpeg-export-evidence/library-import-reference-evidence.json

sips -g format -g pixelWidth -g pixelHeight -g space \
  .tmp/q5-jpeg-export-evidence/run/Exports/reference-urban-export.jpg
```

## Export Evidence

| Field | Value |
| --- | --- |
| Source file | `.tmp/q5-jpeg-export-evidence/run/Import Originals/reference-urban.jpg` |
| Output file | `.tmp/q5-jpeg-export-evidence/run/Exports/reference-urban-export.jpg` |
| Output exists | true |
| Output bytes | `75680` |
| Output format | `jpeg` |
| Output dimensions | `720 x 480` |
| `sips` color space | `RGB` |
| Export settings format | `jpeg` |
| Export settings color profile | `srgb` |
| Export quality | `90` |
| ICC profile embedded | true |
| ICC profile SHA256 | `2b3aa1645779a9e634744faf9b01e9102b0c9b88fd6deced7934df86b949af7e` |
| Source SHA256 before export | `b7e64b5967716ad780751bf738bb957f5abbd54a4ea0d52c956da731963c2614` |
| Source SHA256 after export | `b7e64b5967716ad780751bf738bb957f5abbd54a4ea0d52c956da731963c2614` |
| Source original hash unchanged | true |

## Evidence Boundary

This pass proves:

- the developer desktop runtime writes a separate JPEG export artifact
- the output opens through macOS `sips` as JPEG image data
- export settings record JPEG sRGB quality 90
- export settings record embedded ICC evidence
- source and output paths differ
- the source SHA-256 before and after export is unchanged

This pass does not prove manual GUI export dialog behavior, `/Applications` launch, DMG install, Gatekeeper, notarization, offline behavior, or clean-Mac behavior.

## Next Gate

Q5.5 should record unsupported, missing-original, and cache-clear trust-state evidence from the local runtime.
