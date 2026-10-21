#!/usr/bin/env bash
# Manual M1 test helper: builds a release lira-app, starts it with logging and
# prints the steps for the microphone test. It never opens the microphone by
# itself; recording only starts when you run `lira toggle` or press the bind.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

LOG="${LIRA_TEST_LOG:-/tmp/lira-manual-test.log}"
MODELS_DIR="${LIRA_MODELS_DIR:-${XDG_DATA_HOME:-$HOME/.local/share}/lira/models}"
SMALL_FILE="ggml-small-q5_1.bin"
SMALL_URL="https://huggingface.co/ggerganov/whisper.cpp/resolve/main/${SMALL_FILE}"
SMALL_SHA256="ae85e4a935d7a567bd102fe55afc16bb595bdb618e11b2fc7591bc08120411bb"
NIRI_SNIPPET="/tmp/lira-niri-test.kdl"

echo "== Dependencies =="
missing=0
for cmd in cargo pnpm wl-copy curl sha256sum; do
  if command -v "$cmd" >/dev/null 2>&1; then
    echo "ok: $cmd"
  else
    echo "MISSING: $cmd"
    missing=1
  fi
done
if [[ "$missing" == "1" ]]; then
  echo "Install the missing tools (e.g. sudo pacman -S wl-clipboard) and run again." >&2
  exit 1
fi

echo "== Model =="
mkdir -p "$MODELS_DIR"
if [[ -f "$MODELS_DIR/$SMALL_FILE" ]]; then
  echo "ok: $MODELS_DIR/$SMALL_FILE"
else
  echo "Downloading $SMALL_FILE (~190 MB)..."
  curl -fL --progress-bar -o "$MODELS_DIR/$SMALL_FILE.part" "$SMALL_URL"
  echo "$SMALL_SHA256  $MODELS_DIR/$SMALL_FILE.part" | sha256sum -c -
  mv "$MODELS_DIR/$SMALL_FILE.part" "$MODELS_DIR/$SMALL_FILE"
fi

echo "== Release build =="
pnpm --dir app/ui install --frozen-lockfile
pnpm --dir app/ui build
cargo build --release -p lira-app -p lira-cli --features lira-app/custom-protocol

LIRA="$ROOT/target/release/lira"
APP="$ROOT/target/release/lira-app"

if pgrep -x lira-app >/dev/null 2>&1; then
  echo "lira-app is already running (pid $(pgrep -x lira-app | tr '\n' ' ')). Quit it first." >&2
  exit 1
fi

echo "== Starting lira-app (log: $LOG) =="
: >"$LOG"
RUST_LOG="${RUST_LOG:-info}" nohup "$APP" >>"$LOG" 2>&1 &
APP_PID=$!

ready=0
for _ in $(seq 1 20); do
  if "$LIRA" status >/dev/null 2>&1; then
    ready=1
    break
  fi
  if ! kill -0 "$APP_PID" 2>/dev/null; then
    break
  fi
  sleep 0.5
done
if [[ "$ready" != "1" ]]; then
  echo "lira-app did not answer 'lira status'. Last log lines:" >&2
  tail -n 20 "$LOG" >&2 || true
  exit 1
fi
echo "lira status: $("$LIRA" status)"

cat >"$NIRI_SNIPPET" <<KDL
// Lira manual test. Mod+Space is taken by the DMS launcher, so use Mod+Alt+Space.
binds {
    Mod+Alt+Space repeat=false { spawn "$LIRA" "toggle"; }
}
KDL

cat <<TXT

== Manual test (you do this) ==
lira-app is running (pid $APP_PID). Nothing is recording yet.

1. Toggle by CLI:   $LIRA toggle      (starts listening, popup appears)
   Speak a sentence, then either run '$LIRA toggle' again or stay silent ~1.5 s
   (auto-stop). The popup shows "Transcribing..." and then the text.
2. Paste anywhere (Ctrl+V): the text must be in the clipboard.
3. Run '$LIRA toggle' once more to close the popup.
4. Keybind: cp $NIRI_SNIPPET ~/.config/niri/lira-test.kdl
   and add this line at the end of ~/.config/niri/config.kdl:
       include optional=true "lira-test.kdl"
   Niri reloads automatically. Then use Mod+Alt+Space instead of the CLI.
5. Repeat with a few pt-BR sentences (short ~3 s and long ~10 s).
6. History: sqlite3 ~/.local/share/lira/history.sqlite3 'select created_at, latency_ms, audio_ms, text from transcripts order by id desc limit 5;'

Latency per transcription: grep 'stt: transcribed' $LOG
Stop the app: kill $APP_PID
Log file to send back: $LOG
TXT
