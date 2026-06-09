#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../.."

echo "==> Checking Markdown local links"
python3 scripts/harness/check-md-links.py

echo "==> Checking Cargo dependency documentation"
python3 scripts/harness/check-cargo-deps.py

echo "==> Checking early-alpha scope guardrails"
scripts/harness/check-scope-guardrails.sh

echo "==> Checking static UI contract"
python3 scripts/harness/check-static-ui.py

echo "==> Checking UI workflow smoke path"
python3 scripts/harness/check-ui-workflow-smoke.py

echo "==> Checking Rust formatting"
cargo fmt --all --check

echo "==> Building Rust workspace"
cargo build --workspace

echo "==> Testing Rust workspace"
cargo test --workspace

echo "==> Harness checks passed"
