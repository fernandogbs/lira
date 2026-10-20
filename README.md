# Lira

Open-source (MIT OR Apache-2.0) tray/popup app to talk by voice to coding agents (Claude Code first). Primary target: Linux on Niri (Wayland).

**M0** is a scaffold: Tauri shell, Unix-socket CLI, minimal Svelte popup, and crate boundaries for audio, STT, adapters, compositor detection, and plugins. Voice capture and Claude Code integration come in later milestones.

## Run (development)

```bash
# Terminal 1 — UI dev server is started by Tauri
cd app/src-tauri && cargo tauri dev

# Or build the workspace
SKIP_DCO=1 ./scripts/check.sh
cargo build -p lira-cli
```

## CLI (hotkey / spawn)

With the app running, control it via the Unix socket at `$XDG_RUNTIME_DIR/lira.sock`:

```bash
lira toggle   # show/hide popup and toggle listening (M0 UI state)
lira start
lira stop
lira status
```

On Niri, bind toggle without repeat (no global key-release):

```kdl
Mod+Space repeat=false { spawn "lira" "toggle"; }
```

See `packaging/niri/lira.kdl` for the recommended window rule (`app-id` `lira`, floating top-right, block screen capture).

Optional tray: set `LIRA_NO_TRAY=1` to disable. Tray failures are logged and never exit the app.

## Configuration

`~/.config/lira/config.toml`:

```toml
hotkey_mode = "toggle"   # or "hold" (accepted; hold is a stub in M0)
```

## License

Licensed under MIT or Apache-2.0, at your option (`LICENSE-MIT`, `LICENSE-APACHE`).
