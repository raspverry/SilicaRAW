---
title: "ADR 0008: Project License"
status: accepted
audience: all
updated: 2026-06-11
source_of_truth: LICENSE
---

# ADR 0008: Project License

## Context

SilicaRAW needs a final project license before broad public contribution and public beta work. ADR 0004 kept license selection as a maintainer decision and blocked public beta until the final project license and dependency inventory were ready.

The repository is a mixed Rust/Tauri desktop application with project documentation, schemas, harness scripts, and future optional extension surfaces. The local alpha does not bundle model weights or redistributable RAW/color fixture assets.

## Decision

SilicaRAW source code and project documentation are licensed under the MIT License.

The repository root `LICENSE` file is the authoritative project license text.

Third-party dependencies keep their own licenses and are tracked in `docs/DEPENDENCIES.md`.

Sample assets, model weights, generated release artifacts, and third-party binaries are not automatically covered by the project license. They require explicit source, license, rights, and integrity records before being committed or shipped.

## Consequences

- Contributors and downstream users have a simple permissive license for project code and docs.
- The public beta license gate from ADR 0004 is no longer open for project source code and documentation.
- Dependency, model, and sample-asset license gates remain separate release checks.
- Public docs must not imply that unsupported RAW decoding, color correctness, MLX, MCP, plugins, Homebrew, auto-update, or Mac App Store distribution are available.

## Alternatives Considered

- Keep the project license pending: rejected because Task 10.6 exists to finish the public open-source trust package.
- Apache-2.0: viable, but heavier than needed for the current small desktop application and docs repository.
- MIT OR Apache-2.0 dual license: common in Rust, but more policy surface than this repository currently needs.
- GPL or AGPL: rejected because the project goal is broad desktop-app adoption and low contributor friction.

## Links

- [Project License](../../../LICENSE)
- [Dependencies Policy](../../DEPENDENCIES.md)
- [ADR 0004: Local Alpha Scope and License Gates](adr-0004-local-alpha-scope-and-license-gates.md)
- [Public Trust](../topics/public-trust.md)
