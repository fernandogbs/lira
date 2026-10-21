#!/usr/bin/env bash
# FLEURS test split: real read speech, CC-BY-4.0 (https://huggingface.co/datasets/google/fleurs)
set -euo pipefail

DIR="${LIRA_BENCH_DIR:-$HOME/.cache/lira-bench}"
BASE_URL="https://huggingface.co/datasets/google/fleurs/resolve/main/data"
N_SAMPLES=30

audio_duration_secs() {
  local f="$1"
  if command -v soxi >/dev/null 2>&1; then
    soxi -D "$f"
  elif command -v python3 >/dev/null 2>&1; then
    python3 - "$f" <<'PY'
import struct, sys

def duration(path):
    try:
        with open(path, "rb") as f:
            if f.read(4) != b"RIFF":
                return None
            f.read(4)
            if f.read(4) != b"WAVE":
                return None
            byte_rate = None
            data_size = None
            while True:
                hdr = f.read(8)
                if len(hdr) < 8:
                    break
                chunk_id, chunk_size = struct.unpack("<4sI", hdr)
                cid = chunk_id.decode("ascii", errors="replace")
                if cid == "fmt ":
                    fmt = f.read(chunk_size)
                    if len(fmt) >= 16:
                        byte_rate = struct.unpack("<I", fmt[8:12])[0]
                elif cid == "data":
                    data_size = chunk_size
                    break
                else:
                    f.seek(chunk_size, 1)
                if chunk_size % 2:
                    f.read(1)
            if byte_rate and data_size and byte_rate > 0:
                return data_size / byte_rate
    except OSError:
        pass
    return None

d = duration(sys.argv[1])
if d is not None:
    print(d)
PY
  else
    echo ""
  fi
}

for lang in pt_br en_us; do
  dest="$DIR/fleurs/$lang"
  mkdir -p "$dest"

  tsv="$dest/test.tsv"
  if [[ ! -f "$tsv" ]]; then
    echo "Downloading $lang test.tsv..."
    curl -fsSL "$BASE_URL/$lang/test.tsv" -o "$tsv"
  fi

  archive="$dest/test.tar.gz"
  if [[ ! -f "$archive" ]]; then
    echo "Downloading $lang audio archive..."
    curl -fsSL "$BASE_URL/$lang/audio/test.tar.gz" -o "$archive"
  fi

  mapfile -t rows < <(
    awk -F'\t' -v n="$N_SAMPLES" '
      !seen[$1]++ { print; if (++c >= n) exit }
    ' "$tsv"
  )

  extract_list=()
  manifest_tmp="$dest/manifest.tsv.tmp"
  : >"$manifest_tmp"
  for row in "${rows[@]}"; do
    IFS=$'\t' read -r _id filename raw_trans _rest <<<"$row"
    extract_list+=("test/$filename")
    wav_path="$dest/test/$filename"
    printf '%s\t%s\n' "$wav_path" "$raw_trans" >>"$manifest_tmp"
  done

  mkdir -p "$dest/test"
  echo "Extracting ${#extract_list[@]} wav files for $lang..."
  tar -xzf "$archive" -C "$dest" "${extract_list[@]}"

  mv "$manifest_tmp" "$dest/manifest.tsv"

  count="${#rows[@]}"
  total_secs=0
  have_dur=1
  while IFS=$'\t' read -r wav_path _ref; do
    dur="$(audio_duration_secs "$wav_path" || true)"
    if [[ -z "$dur" ]]; then
      have_dur=0
      continue
    fi
    total_secs="$(LC_NUMERIC=C awk -v a="$total_secs" -v b="$dur" 'BEGIN{print a+b}')"
  done <"$dest/manifest.tsv"

  if [[ "$have_dur" -eq 1 ]]; then
    LC_NUMERIC=C printf '%s: %s utterances, %.2f s audio\n' "$lang" "$count" "$total_secs"
  else
    printf '%s: %s utterances (audio duration skipped: install soxi or python3)\n' "$lang" "$count"
  fi
done
