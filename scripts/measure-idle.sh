#!/usr/bin/env bash
# Release binary for idle measurements: pnpm --dir app/ui build, then
# cargo build --release -p lira-app --features lira-app/custom-protocol
set -euo pipefail

BINARY="${1:-./target/release/lira-app}"
DURATION="${DURATION:-60}"
WARMUP=10
CLK_TCK="$(getconf CLK_TCK)"

all_descendants() {
  local -a queue=("$1")
  local -a all=("$1")
  local i=0
  while (( i < ${#queue[@]} )); do
    local pid="${queue[i]}"
    i=$((i + 1))
    local child
    while IFS= read -r child; do
      [[ -z "$child" ]] && continue
      all+=("$child")
      queue+=("$child")
    done < <(pgrep -P "$pid" 2>/dev/null || true)
  done
  printf '%s\n' "${all[@]}"
}

read_smaps_rollup_kb() {
  local pid="$1"
  if [[ -r "/proc/${pid}/smaps_rollup" ]]; then
    awk '/^Rss:/{r=$2} /^Pss:/{p=$2} END{print r+0, p+0}' "/proc/${pid}/smaps_rollup"
  else
    printf '0 0\n'
  fi
}

tree_mem_kb() {
  local root="$1"
  local total_rss=0 total_pss=0
  local pid rss pss
  while IFS= read -r pid; do
    [[ -z "$pid" ]] && continue
    read -r rss pss < <(read_smaps_rollup_kb "$pid")
    total_rss=$((total_rss + rss))
    total_pss=$((total_pss + pss))
  done < <(all_descendants "$root")
  printf '%s %s\n' "$total_rss" "$total_pss"
}

tree_cpu_jiffies() {
  local root="$1"
  local total=0
  local pid j
  while IFS= read -r pid; do
    [[ -z "$pid" ]] && continue
    if [[ -r "/proc/${pid}/stat" ]]; then
      j="$(awk '{print $14 + $15}' "/proc/${pid}/stat")"
      total=$((total + j))
    fi
  done < <(all_descendants "$root")
  printf '%s\n' "$total"
}

kill_tree() {
  local pid="$1"
  local child
  while IFS= read -r child; do
    [[ -z "$child" ]] && continue
    kill_tree "$child"
  done < <(pgrep -P "$pid" 2>/dev/null || true)
  kill -TERM "$pid" 2>/dev/null || true
}

ROOT_PID=""
cleanup() {
  if [[ -n "$ROOT_PID" ]]; then
    kill_tree "$ROOT_PID"
    wait "$ROOT_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT INT TERM

# LIRA_NO_TRAY is respected when already set in the environment.
"$BINARY" &
ROOT_PID=$!

sleep "$WARMUP"

rss_sum=0
pss_sum=0
cpu_sum=0
rss_max=0
pss_max=0
samples=0

prev_cpu="$(tree_cpu_jiffies "$ROOT_PID")"
prev_ts="$(date +%s%N)"

for ((s = 0; s < DURATION; s++)); do
  sleep 1

  if ! kill -0 "$ROOT_PID" 2>/dev/null; then
    echo "error: root process $ROOT_PID exited before sampling ended" >&2
    exit 1
  fi

  read -r rss_kb pss_kb < <(tree_mem_kb "$ROOT_PID")
  rss_mib=$((rss_kb / 1024))
  pss_mib=$((pss_kb / 1024))

  cur_cpu="$(tree_cpu_jiffies "$ROOT_PID")"
  cur_ts="$(date +%s%N)"
  delta_cpu=$((cur_cpu - prev_cpu))
  delta_ns=$((cur_ts - prev_ts))
  delta_s="$(awk -v ns="$delta_ns" 'BEGIN { printf "%.6f", ns / 1000000000 }')"
  cpu_pct="$(awk -v dj="$delta_cpu" -v tck="$CLK_TCK" -v ds="$delta_s" \
    'BEGIN { if (ds > 0) printf "%.2f", 100.0 * dj / (tck * ds); else print "0.00" }')"

  rss_sum=$((rss_sum + rss_mib))
  pss_sum=$((pss_sum + pss_mib))
  cpu_sum="$(awk -v a="$cpu_sum" -v b="$cpu_pct" 'BEGIN { printf "%.2f", a + b }')"
  if ((rss_max < rss_mib)); then
    rss_max=$rss_mib
  fi
  if ((pss_max < pss_mib)); then
    pss_max=$pss_mib
  fi
  samples=$((samples + 1))

  prev_cpu="$cur_cpu"
  prev_ts="$cur_ts"
done

rss_avg=$((rss_sum / samples))
pss_avg=$((pss_sum / samples))
cpu_avg="$(awk -v s="$cpu_sum" -v n="$samples" 'BEGIN { printf "%.2f", s / n }')"

tree_procs=$(all_descendants "$ROOT_PID" | wc -l)

printf 'avg RSS: %s MiB (max %s MiB)\n' "$rss_avg" "$rss_max"
printf 'avg PSS: %s MiB (max %s MiB)\n' "$pss_avg" "$pss_max"
printf 'avg CPU: %s%%\n' "$cpu_avg"
printf 'processes in tree: %s\n' "$tree_procs"
