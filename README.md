# Lira

Open-source (MIT OR Apache-2.0) tray/popup app to talk by voice to coding agents (Claude Code first). Primary target: Linux on Niri (Wayland).

**M1** delivers local push-to-talk and transcription on Niri. **M2** routes transcripts to Claude Code (`claude -p` / `--resume`) with a rule-based intent router, session store, desktop notifications, and `lira sessions`. Screen context capture is **M3**.

See also [README.pt-BR.md](README.pt-BR.md).

## Run (development)

```bash
# Terminal 1 — UI dev server is started by Tauri
cd app/src-tauri && cargo tauri dev

# Or build the workspace
SKIP_DCO=1 ./scripts/check.sh
cargo build -p lira-cli
```

Manual M1 checklist (Niri): `./scripts/manual-test-m1.sh`

## CLI (hotkey / spawn)

With the app running, control it via the Unix socket at `$XDG_RUNTIME_DIR/lira.sock`:

```bash
lira toggle   # toggle listening (press again to stop / VAD auto-stop)
lira start    # start recording (hold / compositor press bind)
lira stop     # stop recording and transcribe (hold / compositor release bind)
lira status
lira sessions
```

On Niri (no GlobalShortcuts portal, no key-release binds), use **toggle** by default:

```kdl
Mod+Space repeat=false { spawn "lira" "toggle"; }
```

See `packaging/niri/lira.kdl` for the recommended window rule (`app-id` `lira-app`, floating top-right, block screen capture).

**Hold mode:** set `[hotkey] mode = "hold"` and choose `hold_backend`:

| Backend | When | How |
|---------|------|-----|
| `compositor` (default on Niri) | Hyprland `bind`/`bindr`, or any press/release binds | See `packaging/hyprland/lira.conf` |
| `portal` | KDE / GNOME / Hyprland portal | App requests GlobalShortcuts; activate→start, deactivate→stop |
| `evdev` | Niri opt-in only | Run `lira-hold` after setting `evdev_acknowledged = true`. Requires `input` group. **Never auto-enabled.** |

On Niri, prefer toggle + VAD. Read the security warning printed by `lira-hold` before enabling evdev.

Optional tray: on Linux the app creates a tray icon only when `org.kde.StatusNotifierWatcher` is on the session bus (e.g. DankMaterialShell / dux-shell). Set `LIRA_NO_TRAY=1` to disable. Tray failures are logged and never exit the app.

## Configuration

`~/.config/lira/config.toml` (sectioned, spec 5.2):

```toml
[hotkey]
mode = "toggle"           # or "hold" (VAD off while holding)
vad_autostop_ms = 1500    # toggle only; 0 disables
hold_backend = "compositor"  # or "portal" | "evdev"
# hold_key = "KEY_SPACE"   # for lira-hold
# evdev_acknowledged = false

[ui]
locale = "system"         # or "pt-BR" | "en"
popup = "auto"            # or "layer_shell" | "floating"

[stt]
engine = "whisper"
model = "small-q5_1"      # default after M1 bench; "large-v3-turbo-q5_0" for quality
language = "auto"         # or "pt" | "en" (auto follows UI locale)
vocabulary = []           # extra terms for the Whisper initial prompt
keep_warm_min = 5

[history]
enabled = true

[agent]
enabled = true
default_session = "focused"  # or a session id/name
copy_transcript = true

[policy]
voice_approve = "off"     # or "reversible"

[screen]
context = "off"           # or "text" (OCR) | "image"
scope = "window"          # or "screen"
preview = "first"         # or "always" | "never"
exclude_apps = []         # extra app_id substrings
```

Requires `claude` on `PATH` for agent mode. Set `[agent] enabled = false` for transcription-only.

Screen context is **off by default**. Enable `context = "text"` for local OCR (Tesseract `por`+`eng`). Captures are in-memory only. Print Niri block-out rules with `lira setup niri`.

Models download on first use (SHA-256 verified) under the XDG data directory.

## License

Licensed under MIT or Apache-2.0, at your option (`LICENSE-MIT`, `LICENSE-APACHE`).
