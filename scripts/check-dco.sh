#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

BASE="${DCO_BASE:-main}"
if ! git rev-parse --verify "${BASE}" >/dev/null 2>&1; then
  echo "DCO: base ref '${BASE}' not found; skipping range check"
  [ -n "${CI:-}" ] && exit 1
  exit 0
fi

RANGE="${BASE}..HEAD"
if ! git rev-list --count "${RANGE}" | grep -qv '^0$'; then
  echo "DCO: no commits in ${RANGE}"
  exit 0
fi

while IFS= read -r sha; do
  author_name=$(git show -s --format='%an' "$sha")
  author_email=$(git show -s --format='%ae' "$sha")
  pattern="Signed-off-by: ${author_name} <${author_email}>"
  if ! git show -s --format='%B' "$sha" | grep -Fq "$pattern"; then
    echo "DCO: missing Signed-off-by for commit $sha (expected: $pattern)"
    exit 1
  fi
done < <(git rev-list --no-merges "${RANGE}")

echo "DCO: OK (${RANGE})"
