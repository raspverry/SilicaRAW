# Phase 15 RAW Export Manual Color QA

Status: complete for Task 15.6 on 2026-06-12.

This record covers one fixture-backed RAW-derived JPEG sRGB export. It proves that the generated export has inspectable sRGB ICC evidence and opens in Preview.app on the maintainer Mac. It does not prove broad RAW camera support or broad visual color correctness.

## Review Record

```txt
reviewer: Codex on maintainer Mac
date: 2026-06-12
git commit: 7becb0cdfc31bfd85185b0268b1de7be9f8295e0
fixture manifest path: /Users/hansol/dev/personal/SilicaRAW/.tmp/legal-raw-fixtures/raw-fixtures.json
source fixture id: raw_pixls_canon_eos_7d_cr2_raw_3_2
source fixture class: A
export target: srgb
export command or app artifact: SILICARAW_RAW_EXPORT_QA_DIR=/Users/hansol/dev/personal/SilicaRAW/.tmp/phase-15-raw-export-manual-qa cargo test -p silica-core --features core-image-raw-probe raw_derived_jpeg_srgb_export_from_fixture_records_evidence_without_preview_cache -- --ignored
output path: /Users/hansol/dev/personal/SilicaRAW/.tmp/phase-15-raw-export-manual-qa/raw_pixls_canon_eos_7d_cr2_raw_3_2-adjusted-srgb.jpg
output SHA-256: 226ff79e1c63f3a4376928ddf100efdd2cea07de09dcf88a0feecff94928bfaa
embedded ICC profile: sRGB IEC61966-2.1
embedded ICC SHA-256: 2b3aa1645779a9e634744faf9b01e9102b0c9b88fd6deced7934df86b949af7e
original source SHA-256 before: b5e47c5fcf7332ac03e0134926f17a338a42e68c1fd7f83e16f45f4b767544e8
original source SHA-256 after: b5e47c5fcf7332ac03e0134926f17a338a42e68c1fd7f83e16f45f4b767544e8
macOS version: 26.4 (25E246)
viewer: Preview.app
display model or display profile: Built-in Liquid Retina XDR Display, 3024 x 1964 Retina, sRGB export profile
observed issue notes: Output opened in Preview.app. Additional artifact inspection showed a nonblank 5184 x 3456 image with no gross decode corruption, no visible profile warning, and no missing ICC evidence. Warm/orange scene rendering remains fixture/RAW-renderer dependent and is not a color-correctness claim.
pass/fail: pass for Task 15.6 evidence-limited Preview.app open and ICC/profile gate
```

## Local QA Artifact

The exported JPEG and machine-readable evidence are local ignored files:

```txt
.tmp/phase-15-raw-export-manual-qa/raw_pixls_canon_eos_7d_cr2_raw_3_2-adjusted-srgb.jpg
.tmp/phase-15-raw-export-manual-qa/raw-export-qa-evidence.json
```

These files must not be committed because they derive from local ignored fixture media.

## Known Limitations

- One Class A RAW fixture was manually reviewed for this gate.
- This record does not prove camera-wide RAW support, Preview.app parity, Photos parity, Lightroom parity, Capture One parity, or vendor-renderer parity.
- This record does not set a perceptual tolerance threshold.
- Visual color correctness remains blocked pending broader tolerance and review evidence.
