#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../.."

echo "==> Checking Markdown local links"
python3 scripts/harness/check-md-links.py

echo "==> Checking Cargo dependency documentation"
python3 scripts/harness/check-cargo-deps.py

echo "==> Checking early-alpha scope guardrails"
scripts/harness/check-scope-guardrails.sh

echo "==> Checking release workflow guardrails"
python3 scripts/harness/check-release-workflows.py

echo "==> Checking release runbook guardrails"
python3 scripts/harness/check-release-runbook.py

echo "==> Checking release template guardrails"
python3 scripts/harness/check-release-template.py

echo "==> Checking legal QA fixtures and installed-app preflight"
python3 scripts/harness/check-qa-fixtures.py

echo "==> Checking static UI contract"
python3 scripts/harness/check-static-ui.py

echo "==> Checking UI workflow smoke path"
python3 scripts/harness/check-ui-workflow-smoke.py

echo "==> Checking connected desktop runtime smoke"
python3 scripts/harness/check-connected-runtime-smoke.py

echo "==> Checking Rust formatting"
cargo fmt --all --check

echo "==> Building Rust workspace"
cargo build --workspace

echo "==> Testing Rust workspace"
cargo test --workspace

echo "==> Harness checks passed"
