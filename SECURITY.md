# Security Policy

SilicaRAW is pre-alpha. The current supported surface is the latest `main` branch and the latest published developer-preview or local-alpha release notes, when they exist.

## Reporting a Vulnerability

Preferred private route: use GitHub Private Vulnerability Reporting or a Private GitHub Security Advisory for this repository when that feature is enabled.

If private reporting is unavailable, do not include exploit details, private file paths, sample photos, credentials, or proof-of-concept payloads in a public issue. Open a minimal public issue titled `Security contact request` and state only the affected area and that private follow-up is needed.

## Sensitive Areas

Please treat these as security-sensitive:

- Any mutation or deletion of original photo files.
- Catalog corruption, edit loss, sidecar corruption, backup corruption, or unsafe restore behavior.
- Export path vulnerabilities, especially paths that could overwrite originals.
- Unexpected network upload, telemetry, analytics, or cloud sync.
- Signing, notarization, release artifact, checksum, or installer integrity issues.
- Unsafe parsing of untrusted image files, sidecars, edit graphs, manifests, or backup manifests.

## Deferred Surfaces

The current local alpha does not include default telemetry, cloud sync, auto-update, Homebrew distribution, Mac App Store distribution, MLX runtime, MCP server, or plugin runtime.

Reports against those deferred surfaces should describe the planned risk or documentation gap, not an active shipped vulnerability, unless a future PR explicitly implements the surface.

## Disclosure Expectations

- Do not publish exploit details until a maintainer has had a reasonable chance to respond.
- Do not include user photos or private libraries in public issues.
- Use synthetic fixtures when possible.
- Keep reproduction steps minimal and focused on local files.

## Security Principles

- Original files are sacred.
- Local state stays local by default.
- Imported JSON, sidecars, manifests, backup manifests, image files, and paths are untrusted input.
- Release artifacts must not be described as signed, notarized, or user-ready unless the documented signing/notarization checks pass.
