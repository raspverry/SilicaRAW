---
title: raw.pixls.us Source Review
status: active
audience: all
updated: 2026-06-12
source_of_truth: none
---

# raw.pixls.us Source Review

## Summary

raw.pixls.us is accepted as the first external source for local ignored RAW probe fixtures.

This review approves selected files for local-only download and probing. It does not permit committing RAW media to the repository, and it does not prove product RAW support.

## Source Evidence

- raw.pixls.us upload flow asks uploaders to declare full rights and release the file under Creative Commons Zero into the public domain.
- raw.pixls.us provides a non-CC0 view for samples that are not under CC0.
- The repository JSON endpoint records per-sample license links and SHA-256 checksums.
- Creative Commons describes CC0 as public-domain dedication with permission to copy, modify, distribute, and perform the work without asking permission.
- raw.pixls.us excludes photographs of people in its upload guidance for legal reasons.

## Accepted Source Policy

Use only samples that meet all rules:

- License field links to `https://creativecommons.org/publicdomain/zero/1.0/`.
- Candidate is absent from `https://raw.pixls.us/json/getrepository.php?set=noncc0`.
- SHA-256 is recorded before use.
- Media is downloaded only into an ignored local fixture directory.
- Local manifest uses `source_policy.policy = "local_ignored_corpus"`.
- No RAW media is staged or committed.

Reject samples that meet any rule:

- License is unclear, missing, non-CC0, non-commercial, share-alike, or imported from rawsamples.ch under non-CC0 terms.
- File is a color target unless a later color-target task explicitly needs it.
- File contains people or other obvious privacy risk.
- Source hash cannot be verified after download.

## Accepted Candidates

These candidates are approved for Task 12.5.2 local-only download and Task 12.5.3 manifest creation.

| Fixture class | Candidate id | Source | Format | License | SHA-256 | Size | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| A | `raw_pixls_canon_eos_7d_cr2_raw_3_2` | `https://raw.pixls.us/getfile.php/131/nice/Canon - EOS 7D - RAW (3:2).CR2` | CR2 | CC0 1.0 Universal | `b5e47c5fcf7332ac03e0134926f17a338a42e68c1fd7f83e16f45f4b767544e8` | 23.99 MB | Ordinary Core Image candidate RAW. Avoid same-camera color-target row `getfile.php/1183`. |
| B | `raw_pixls_canon_eos_r6_mark_iii_cr3_full_frame` | `https://raw.pixls.us/getfile.php/8961/nice/Canon - EOS R6 Mark III - RAW (3:2).CR3` | CR3 | CC0 1.0 Universal | `e491e4bb960961b5fa299361bf698310a80cbe7b15d30d8dad3bb21bc5457dab` | 33.01 MB | Newer high-risk/edge-case RAW candidate. |
| C | `raw_pixls_fujifilm_x_t30_iii_raf_compressed` | `https://raw.pixls.us/getfile.php/8769/nice/Fujifilm - X-T30 III - 14bit compressed (3:2).RAF` | RAF | CC0 1.0 Universal | `49f77d6162abfa5c94d2d8b90e4e926b7386c42bcf7e84a152c9ffe1ebd584da` | 28.66 MB | Fuji RAF candidate. |
| D | `raw_pixls_apple_iphone_12_pro_dng` | `https://raw.pixls.us/getfile.php/4264/nice/Apple - iPhone 12 Pro - 8bit (4:3).DNG` | DNG | CC0 1.0 Universal | `e91e77a4533ed7cce551d83330676ea5c47dd5e55fb38adda7819366afdbdfc2` | 27.84 MB | Apple ProRAW-style DNG candidate. raw.pixls mirror path is `Apple/iPhone 12 Pro/IMG_1361.DNG`. |

## Deferred Candidate

Fixture class E still needs a separate source decision.

Class E should represent a legal RAW-like file expected to remain unsupported or blocked. Do not use fake RAW files, unclear-license files, or the existing `.tmp` blocked placeholders as fixture evidence.

## Evidence Commands

These commands were used on 2026-06-12 to extract candidate rows:

```bash
curl -sS 'https://raw.pixls.us/json/getrepository.php?set=all' | jq -r '.data[] | select(.[0]=="Apple" and .[1]=="iPhone 12 Pro") | @tsv'
curl -sS 'https://raw.pixls.us/json/getrepository.php?set=all' | jq -r '.data[] | select(.[0]=="Fujifilm" and .[1]=="X-T30 III") | @tsv'
curl -sS 'https://raw.pixls.us/json/getrepository.php?set=all' | jq -r '.data[] | select(.[0]=="Canon" and .[1]=="EOS 7D" and .[2]=="RAW (3:2)") | @tsv'
curl -sS 'https://raw.pixls.us/json/getrepository.php?set=all' | jq -r '.data[] | select(.[0]=="Canon" and .[1]=="EOS R6 Mark III" and .[2]=="RAW (3:2)" and .[4]=="Full-frame") | @tsv'
curl -sS 'https://raw.pixls.us/json/getrepository.php?set=noncc0' | jq -r '.data[] | @tsv' | rg -i 'Canon\tEOS 7D|Canon\tEOS R6 Mark III|Fujifilm\tX-T30 III|Apple\tiPhone 12 Pro'
```

The final non-CC0 filter produced no matching rows for the accepted candidates.

## Links

- raw.pixls.us: https://raw.pixls.us/
- Repository JSON: https://raw.pixls.us/json/getrepository.php?set=all
- Non-CC0 JSON: https://raw.pixls.us/json/getrepository.php?set=noncc0
- Data mirror: https://raw.pixls.us/data/
- CC0 deed: https://creativecommons.org/publicdomain/zero/1.0/
- Task card: [Task 12.5 Legal RAW Fixture Evidence](../tasks/12.5-legal-raw-fixture-evidence.md)
- RAW topic: [RAW Decoding](../topics/raw-decoding.md)

## Notes for LLM Agents

This page only approves external source candidates. Next work must download files into an ignored local path, verify SHA-256, create a local fixture manifest, run the probe harness, and update the support matrix from actual probe output.
