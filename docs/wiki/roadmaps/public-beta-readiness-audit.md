---
title: Public Beta Readiness Audit
status: active
audience: maintainers
updated: 2026-06-18
source_of_truth: docs/wiki/roadmaps/post-alpha-product-roadmap.md
---

# Public Beta Readiness Audit

## Verdict

SilicaRAW is **not ready for public beta**.

The product can continue internal developer-preview testing, but Task 27.2 public beta release-candidate work is blocked until the project can produce a signed and notarized DMG with checksums and clean-Mac downloaded-artifact QA.

## Audit Inputs

- [Public Beta Evidence Index](public-beta-evidence-index.md)
- [Public Beta Scope Freeze Checklist](../../../checklists/PUBLIC_BETA_SCOPE_FREEZE.md)
- [Local DMG Release Runbook](local-dmg-release-runbook.md)
- [Dependencies Policy](../../DEPENDENCIES.md)
- [README](../../../README.md)
- [Release Template](../../../.github/release-template.md)

## Audit Result Matrix

| Gate | Result | Evidence | Notes |
| --- | --- | --- | --- |
| P0/P1 stability | Conditional pass | `scripts/harness/check.sh` passes locally on 2026-06-18. | Still needs clean-Mac downloaded-DMG QA before public beta. |
| No known data-loss bugs | Conditional pass | Harness original-file safety checks, data-safety docs, action trust docs. | No known data-loss bug is recorded in current local evidence. |
| Final license selected | Pass | [MIT License](../../../LICENSE), [Public Trust](../topics/public-trust.md). | Project source/docs use MIT unless a file states otherwise. |
| Dependency license inventory | Pass | [Dependencies Policy](../../DEPENDENCIES.md), harness dependency documentation check. | Inventory is current for committed dependencies. |
| Sample asset license manifest | Pass by absence | [Installed App Preflight](../../../checklists/INSTALLED_APP_PREFLIGHT.md), fixture manifest contract. | No redistributable product sample media ships. Any future sample assets need license/source/hash records. |
| Model license manifest | Not applicable | [Model Manifest Schema](../../../schemas/model_manifest.schema.json). | No models ship in public beta scope. |
| README limitation honesty | Pass | [README](../../../README.md). | README states local alpha status, blocked signed DMG path, and deferred RAW/Metal/MLX/MCP/plugin claims. |
| Release notes honesty | Conditional pass | [Release Template](../../../.github/release-template.md). | Template is honest for local alpha/developer preview; public beta final notes still need filled artifact-specific QA fields. |
| Color/export evidence | Conditional pass | [Color Export Manual QA](../../../checklists/COLOR_EXPORT_MANUAL_QA.md), [Golden Image Tolerance Policy](../../../checklists/GOLDEN_IMAGE_TOLERANCE_POLICY.md). | ICC/export evidence exists; broad visual color correctness remains explicitly unclaimed. |
| Clean-Mac install QA | Fail | [Local DMG Install Checklist](../../../checklists/LOCAL_DMG_INSTALL_CHECKLIST.md). | No signed/notarized downloaded public beta DMG exists to test. |
| Signing/notarization | Fail | [Local DMG Release Runbook](local-dmg-release-runbook.md). | Blocked by Apple Developer Program funding, Developer ID certificate, and notarization credentials. |
| Checksums | Fail | Release template and runbook. | No public beta DMG exists, so no beta checksum can be published. |

## Blocking Items

Task 27.2 cannot start until these are resolved:

```txt
Apple Developer Program funding available
Developer ID Application certificate available
Notarization credentials available
Signed and notarized DMG produced
SHA256SUMS.txt produced for the DMG
Clean-Mac downloaded-artifact install QA passes
Gatekeeper accepts the downloaded DMG and installed app
```

## Allowed Work While Blocked

- Continue internal unsigned developer-preview testing.
- Improve docs, release notes, and QA runbooks.
- Fix bugs found by harness/manual QA.
- Prepare signing/notarization automation only after credentials and funding exist.

## Disallowed Claims While Blocked

- Public beta available.
- Signed/notarized release available.
- Gatekeeper-ready download.
- Broad RAW camera support.
- Broad visual color correctness.
- MLX-powered editor.
- Plugin platform.
- MCP server.

## Decision

Do not produce a public beta release candidate yet. Keep the public beta route blocked and revisit after signing/notarization prerequisites exist.
