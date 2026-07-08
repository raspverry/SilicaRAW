---
title: Local Alpha Review and Edit Persistence Evidence
status: active
audience: maintainers
updated: 2026-07-08
source_of_truth: docs/wiki/roadmaps/local-alpha-quality-closure-plan.md
---

# Local Alpha Review and Edit Persistence Evidence

## Scope

This report records Q5.3 evidence that review flags and a basic Develop edit persist through the local desktop runtime storage path.

It uses the same developer desktop command runtime evidence boundary as Q5.2. It does not automate the WebView GUI, native path picker, app menu, `/Applications` install, DMG behavior, Gatekeeper acceptance, signed/notarized behavior, offline behavior, or clean-Mac downloaded-artifact behavior.

## Test Record

| Field | Value |
| --- | --- |
| Date | 2026-07-08 |
| Source commit at evidence run | `7bedb18fdd0a8f26ef7b8432d8b87ab6cbbac96a` |
| App artifact path | `target/release/bundle/macos/SilicaRAW.app` |
| App tree SHA256 | `89ab925b523bc2b7943fcfa8ad3318e3266b542edc5d8f7967addf8ff60da95d` |
| Evidence command | `scripts/harness/check-library-import-reference-evidence.py` |
| Evidence report | `.tmp/q5-review-edit-persistence/library-import-reference-evidence.json` |
| Evidence report SHA256 | `9c463e2c0b25e1bc0546c9f9e8b9db8f6d4a82e56210519d5015b9abbbb97bd2` |
| Catalog DB | `.tmp/q5-review-edit-persistence/run/SilicaRAW Library/catalog.db` |
| Catalog DB SHA256 | `1b15bc20000d83759704c848858cabd2f499b6dee74b7dcd402baf709f6ec64b` |
| App session | `.tmp/q5-review-edit-persistence/run/AppConfig/app-session.json` |
| App session SHA256 | `eb559f319f3c044073d42e78bd32c608f27ecdc693bf04f272956aeecb53ce3e` |
| Result | `pass` |

## Command

```bash
python3 scripts/harness/check-library-import-reference-evidence.py \
  --app target/release/bundle/macos/SilicaRAW.app \
  --scratch .tmp/q5-review-edit-persistence \
  --output .tmp/q5-review-edit-persistence/library-import-reference-evidence.json
```

## Persistence Evidence

The selected persisted photo is `reference-urban.jpg`.

| Field | Value |
| --- | --- |
| Final rating | `4` |
| Final picked | `true` |
| Final rejected | `false` |
| Final color label | `green` |
| Edited flag | `true` |
| Exported flag | `true` |
| Active edit exposure | `0.4` |
| Active edit contrast | `12.0` |
| Active edit source SHA256 | `b7e64b5967716ad780751bf738bb957f5abbd54a4ea0d52c956da731963c2614` |
| App-session last mode | `develop` |
| App-session selected photo | `reference-urban.jpg` catalog id |

## Undoable History

| Sequence | Kind | State | Persisted Result |
| --- | --- | --- | --- |
| `1` | `flag_change` | `applied` | rating `5`, picked `true`, rejected `false` |
| `2` | `flag_change` | `applied` | rating `3`, picked `false`, rejected `true` |
| `3` | `flag_change` | `applied` | rating `4`, picked `true`, rejected `false`, color label `green` |
| `4` | `edit_commit` | `applied` | exposure `0.4`, contrast `12.0` |

## Evidence Summary

The connected runtime smoke verifies:

- the library is created and reopened
- the app session records the recent library
- culling flags are written and read back
- Loupe preview opens for the selected JPEG
- exposure/contrast preview produces disposable preview bytes
- exposure/contrast commit persists active edit state
- launch restore resolves the saved selected photo and requested Develop mode
- reopening the library reads back the final flags and edit state
- original source hashes remain unchanged

## Evidence Boundary

This pass proves review/edit persistence through the local desktop command runtime and catalog/session files.

This pass does not prove manual GUI interaction, `/Applications` launch, DMG install, Gatekeeper, notarization, offline behavior, or clean-Mac behavior. Those remain Q5.4 through Q7 gates.

## Next Gate

Q5.4 should record JPEG sRGB export evidence with output inspection and original-source overwrite safety.
