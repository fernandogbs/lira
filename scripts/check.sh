#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "== UI install (frozen) =="
pnpm --dir app/ui install --frozen-lockfile

echo "== UI build =="
pnpm --dir app/ui build

echo "== UI i18n keys =="
pnpm --dir app/ui test:i18n

echo "== cargo fmt =="
cargo fmt --all --check

echo "== cargo clippy =="
cargo clippy --workspace --all-targets -- -D warnings

echo "== cargo test =="
cargo test --workspace

echo "== cargo build =="
cargo build --workspace

if [[ "${SKIP_DCO:-}" != "1" ]]; then
  echo "== DCO =="
  "${ROOT}/scripts/check-dco.sh"
else
  echo "== DCO skipped (SKIP_DCO=1) =="
fi

echo "All checks passed."
