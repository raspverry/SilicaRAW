# SilicaRAW Harness

This directory contains the lightweight project harness for SilicaRAW.

The harness is intentionally small. It checks repository health without trying to replace real feature tests, release QA, or the local DMG install checklist.

## Default Check

Run from the repository root:

```bash
scripts/harness/check.sh
```

The default check runs:

1. Markdown local link validation.
2. Cargo dependency documentation check.
3. `cargo fmt --all --check`.
4. `cargo build --workspace`.
5. `cargo test --workspace`.

## Local Alpha Smoke

Use [local-alpha-smoke.md](local-alpha-smoke.md) when the app can be packaged as a DMG and installed into `/Applications`.

## Scope

The harness should stay focused on checks that are cheap, reliable, and relevant to the local DMG alpha.

Do not add broad browser automation, large fixture downloads, cloud checks, telemetry checks, MLX checks, MCP checks, or release notarization automation here unless the project explicitly reaches that phase.

