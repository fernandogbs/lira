#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIR="${LIRA_BENCH_DIR:-$HOME/.cache/lira-bench}"
RESULTS="$DIR/results.jsonl"

"$ROOT/scripts/fetch-bench-data.sh"

cargo build --release -p lira-bench
BENCH_BIN="$ROOT/target/release/lira-bench"

VULKAN_BIN=""
if [[ "${BENCH_VULKAN:-0}" == "1" ]]; then
  CARGO_TARGET_DIR="$ROOT/target/bench-vulkan" cargo build --release -p lira-bench --features vulkan
  VULKAN_BIN="$ROOT/target/bench-vulkan/release/lira-bench"
fi

: >"$RESULTS"

run_bench() {
  local bin="$1"
  local label="$2"
  local model="$3"
  local lang="$4"
  local gpu="$5"
  local manifest="$6"
  local gpu_flag=()
  if [[ "$gpu" == "1" ]]; then
    gpu_flag=(--gpu)
  fi
  "$bin" \
    --manifest "$manifest" \
    --model "$model" \
    --lang "$lang" \
    --label "$label" \
    "${gpu_flag[@]}" >>"$RESULTS"
}

PT_MANIFEST="$DIR/fleurs/pt_br/manifest.tsv"
EN_MANIFEST="$DIR/fleurs/en_us/manifest.tsv"

run_bench "$BENCH_BIN" "A-small-cpu" small pt 0 "$PT_MANIFEST"
run_bench "$BENCH_BIN" "A-small-cpu" small en 0 "$EN_MANIFEST"
run_bench "$BENCH_BIN" "B-turbo-cpu" turbo pt 0 "$PT_MANIFEST"
run_bench "$BENCH_BIN" "B-turbo-cpu" turbo en 0 "$EN_MANIFEST"

if [[ -n "$VULKAN_BIN" ]]; then
  run_bench "$VULKAN_BIN" "C-turbo-vulkan" turbo pt 1 "$PT_MANIFEST"
  run_bench "$VULKAN_BIN" "C-turbo-vulkan" turbo en 1 "$EN_MANIFEST"
fi

python3 - "$RESULTS" <<'PY'
import json
import sys
from pathlib import Path

results_path = Path(sys.argv[1])
rows = [json.loads(line) for line in results_path.read_text().splitlines() if line.strip()]

print("| Config | Lang | WER % | s / 10s audio | Peak RSS MiB |")
print("| --- | --- | ---: | ---: | ---: |")
for r in rows:
    print(
        f"| {r['label']} | {r['lang']} | {r['wer'] * 100:.2f} | "
        f"{r['secs_per_10s_audio']:.3f} | {r['peak_rss_mib']:.1f} |"
    )

# Best turbo: lowest average secs_per_10s across pt+en per label.
turbo = [r for r in rows if r.get("model") == "turbo"]
by_label: dict[str, list[float]] = {}
for r in turbo:
    by_label.setdefault(r["label"], []).append(r["secs_per_10s_audio"])

best_avg = None
best_label = None
for label, secs in by_label.items():
    if len(secs) < 2:
        continue
    avg = sum(secs) / len(secs)
    if best_avg is None or avg < best_avg:
        best_avg = avg
        best_label = label

threshold = 3.0
if best_avg is not None and best_avg <= threshold:
    decision = "large-v3-turbo-q5_0 (turbo default)"
else:
    decision = "small-q5_1 (small default)"

print()
if best_avg is not None:
    print(
        f"**Decision** (turbo avg s/10s = {best_avg:.3f} from `{best_label}`, "
        f"threshold {threshold}): **{decision}**"
    )
else:
    print(f"**Decision** (no turbo runs, threshold {threshold}): **{decision}**")
PY
