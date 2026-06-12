---
title: Synthetic Local Color Fixture Source Review
status: active
audience: all
updated: 2026-06-12
source_of_truth: docs/wiki/roadmaps/phase-13-color-pipeline-proof-plan.md
---

# Synthetic Local Color Fixture Source Review

## Summary

Task 13.1 accepts a local-only synthetic source path for Color Class F fixture work.

The project should not start Phase 13 with external photos. Synthetic fixtures avoid model-release, privacy, GPS, and third-party photo-license risk. The fixture media must remain ignored until redistribution permission for embedded profiles and generated outputs is explicitly reviewed.

## Accepted Source Family

```txt
Source family: SilicaRAW synthetic local color fixtures
Fixture classes: F / srgb_jpeg, display_p3_jpeg, untagged_jpeg
Fixture media policy: local ignored corpus only
Commit permission: blocked until profile redistribution is reviewed
Privacy status: no people, no GPS, no user photos
```

## Candidate Fixtures

| Fixture subclass | Pixel source | Profile source | Accepted for Task 13.2 | Commit permission |
| --- | --- | --- | --- | --- |
| `srgb_jpeg` | synthetic generated pixels | local macOS `/System/Library/ColorSync/Profiles/sRGB Profile.icc` | yes, local-only | blocked |
| `display_p3_jpeg` | synthetic generated pixels | local macOS `/System/Library/ColorSync/Profiles/Display P3.icc` | yes, local-only | blocked |
| `untagged_jpeg` | synthetic generated pixels | no embedded profile after color metadata removal | yes, local-only | blocked until generated asset policy is written |

## Local Profile Evidence

Observed on the maintainer Mac used for Phase 13 planning:

```txt
/System/Library/ColorSync/Profiles/sRGB Profile.icc
sha256: 2b3aa1645779a9e634744faf9b01e9102b0c9b88fd6deced7934df86b949af7e

/System/Library/ColorSync/Profiles/Display P3.icc
sha256: 0ff6958f98684c61f6bbdce1368ddeaf3873baf84545baba482e920d92a914c0
```

These hashes identify the local profile files used for local fixture generation. They are not a redistribution license.

## Planned Generation Policy

Task 13.2 may generate ignored local fixtures with macOS system tools such as `sips`:

```txt
sRGB fixture -> embed local sRGB ICC profile
Display P3 fixture -> match/embed local Display P3 ICC profile
untagged fixture -> remove color-management properties
```

The exact commands, generated file hashes, and local fixture manifest belong to Task 13.2.

## Rejected Alternatives

- External photos with unclear licenses: rejected for Phase 13 start.
- User photos: rejected unless the user explicitly approves a local-only private fixture.
- Committed JPEGs embedding Apple system profiles: blocked until profile redistribution permission is reviewed.
- File labels without ICC/profile evidence: rejected as color proof.

## Decision

Accepted for Task 13.2:

```txt
Use synthetic generated local fixtures in an ignored local corpus.
Use local macOS ColorSync profiles only for local proof.
Do not commit generated fixture media yet.
Do not claim color correctness from source review alone.
```

## Links

- [Phase 13 Color Pipeline Proof Plan](../roadmaps/phase-13-color-pipeline-proof-plan.md)
- [Color Management](../topics/color-management.md)
- [Golden Image and Tolerance Policy](../../../checklists/GOLDEN_IMAGE_TOLERANCE_POLICY.md)

## Notes for LLM Agents

Source review only proves that the planned fixture source is legally safer than external/user photos. It does not prove profile parsing, transform correctness, ICC embedding, export correctness, or visual color correctness.
