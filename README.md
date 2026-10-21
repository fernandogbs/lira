# Lira

Open-source (MIT OR Apache-2.0) tray/popup app to talk by voice to coding agents (Claude Code first). Primary target: Linux on Niri (Wayland).

**M1** delivers local push-to-talk and transcription on Niri: hold or toggle via CLI binds, cpal capture, Silero VAD auto-stop, whisper.cpp (`small-q5_1` default), layer-shell popup with transcript, clipboard copy, and optional SQLite history. Claude Code integration is **M2**; screen context capture is **M3**.

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
```

On Niri (no GlobalShortcuts portal, no key-release binds), use **toggle** by default:

```kdl
Mod+Space repeat=false { spawn "lira" "toggle"; }
```

See `packaging/niri/lira.kdl` for the recommended window rule (`app-id` `lira-app`, floating top-right, block screen capture).

**Hold mode:** set `[hotkey] mode = "hold"` and bind compositor press/release to `lira start` / `lira stop` where the compositor supports release binds (e.g. Hyprland `bind` + `bindr`). On Niri, prefer toggle + VAD. Portal GlobalShortcuts and opt-in `lira-hold` (evdev) land in the M1 exit follow-up; do not enable evdev without reading the security warning in the docs.

Optional tray: on Linux the app creates a tray icon only when `org.kde.StatusNotifierWatcher` is on the session bus (e.g. DankMaterialShell / dux-shell). Set `LIRA_NO_TRAY=1` to disable. Tray failures are logged and never exit the app.

## Configuration

`~/.config/lira/config.toml` (sectioned, spec 5.2):

```toml
[hotkey]
mode = "toggle"           # or "hold" (use start/stop binds; VAD off while holding)
vad_autostop_ms = 1500    # toggle only; 0 disables

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

[screen]
context = "off"           # or "text" | "image" (capture lands in M3)
```

Models download on first use (SHA-256 verified) under the XDG data directory.

## License

Licensed under MIT or Apache-2.0, at your option (`LICENSE-MIT`, `LICENSE-APACHE`).
