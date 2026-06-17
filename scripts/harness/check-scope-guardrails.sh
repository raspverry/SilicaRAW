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
  local cargo="crates/$crate/Cargo.toml"
  local src_dir="crates/$crate/src"

  if [ ! -f "$cargo" ]; then
    fail "missing deferred crate manifest: $cargo"
    return
  fi

  if find "$src_dir" -type f ! -name 'lib.rs' | grep -q .; then
    fail "$crate must remain boundary-only for local alpha; unexpected source files found"
  fi

  if awk '
    /^\[dependencies\]/ { in_deps = 1; next }
    /^\[/ { in_deps = 0 }
    in_deps && $0 !~ /^[[:space:]]*($|#)/ { found = 1 }
    END { exit found ? 0 : 1 }
  ' "$cargo"; then
    fail "$crate must not add runtime dependencies during local alpha"
  fi
}

check_deferred_crate_boundary "silica-mlx"
check_deferred_crate_boundary "silica-mcp"
check_deferred_crate_boundary "silica-plugin"

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
