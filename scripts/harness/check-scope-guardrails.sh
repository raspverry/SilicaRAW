#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../.."

failures=0

fail() {
  echo "scope guard: $*" >&2
  failures=$((failures + 1))
}

check_deferred_crate_boundary() {
  local crate="$1"
  local allowed_deps="${2:-}"
  local cargo="crates/$crate/Cargo.toml"
  local src_dir="crates/$crate/src"

  if [ ! -f "$cargo" ]; then
    fail "missing deferred crate manifest: $cargo"
    return
  fi

  if find "$src_dir" -type f ! -name 'lib.rs' | grep -q .; then
    fail "$crate must remain boundary-only for local alpha; unexpected source files found"
  fi

  local dependency_names
  dependency_names=$(awk '
    /^\[dependencies\]/ { in_deps = 1; next }
    /^\[/ { in_deps = 0 }
    in_deps && $0 !~ /^[[:space:]]*($|#)/ {
      name = $0
      sub(/=.*/, "", name)
      gsub(/[[:space:]]/, "", name)
      print name
    }
  ' "$cargo")

  if [ -n "$dependency_names" ]; then
    if [ -z "$allowed_deps" ]; then
      fail "$crate must not add runtime dependencies during local alpha"
    elif printf '%s\n' "$dependency_names" | grep -Ev "^($allowed_deps)$"; then
      fail "$crate has dependency outside the approved manifest-validation allowlist"
    fi
  fi
}

check_deferred_crate_boundary "silica-mlx" "serde_json|sha2"
check_deferred_crate_boundary "silica-mcp" "serde_json|silica-core"
check_deferred_crate_boundary "silica-plugin" "serde_json"

if rg -n "silica_storage|rusqlite|open_catalog" crates/silica-mcp -g '*.rs' -g '*.toml'; then
  fail "silica-mcp must not access storage, rusqlite, or catalog handles directly"
fi

if rg -n \
  "posthog|sentry|telemetry|analytics|firebase|amplitude|segment\.io|segmentio|@segment|Segment Analytics|segment analytics|cloud sync|cloud-sync|upload_original|uploadPhoto|delete_original|overwrite_original" \
  apps crates scripts \
  -g '*.rs' -g '*.toml' -g '*.ts' -g '*.tsx' -g '*.js' -g '*.jsx' -g '*.sh' -g '*.py' \
  -g '!scripts/harness/check-scope-guardrails.sh'; then
  fail "prohibited early-alpha scope keyword found in product or harness code"
fi

if rg -n "https?://" apps crates -g '*.rs' -g '*.toml' -g '*.ts' -g '*.tsx' -g '*.js' -g '*.jsx'; then
  fail "network URL found in product code; local alpha should stay local-first unless explicitly scoped"
fi

if [ "$failures" -gt 0 ]; then
  exit 1
fi

echo "scope guardrails ok"
