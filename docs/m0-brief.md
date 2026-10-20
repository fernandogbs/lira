# Lira M0 brief (scaffold)

Lira: open-source (MIT OR Apache-2.0) tray/popup app to talk by voice to coding agents (Claude Code first).
Primary target: Linux, Niri compositor (Wayland) on EndeavourOS, with DankMaterialShell (no Waybar).
Full design: docs/fundacao.md (do NOT read it whole; this brief is enough for M0).

## Architecture rules that matter for M0
- One process: Tauri 2 shell (popup webview, optional tray) + Rust core (tokio). All logic in Rust; UI only displays.
- Popup: Svelte (Vite), minimal. Webview should be created on demand and destroyed when hidden (keeps RAM low; WebKitGTK is fragile).
- Global hotkey on Niri: there is NO GlobalShortcuts portal and NO key-release binds. Default mode is toggle.
  User binds in ~/.config/niri/config.kdl: `Mod+Space repeat=false { spawn "lira" "toggle"; }`.
  `lira toggle|start|stop|status` talk to the running app over a Unix socket at `$XDG_RUNTIME_DIR/lira.sock`.
- Tray is OPTIONAL. Niri has no bar; DankMaterialShell may or may not host StatusNotifierItem. App must work 100% without tray
  (hotkey -> CLI -> popup). Tray failure must never panic or exit.
- Popup on Niri: final target is layer-shell; for M0 use a normal window with app-id/class `lira` + document this Niri rule:
  window-rule { match app-id="^lira$"; open-floating true; default-floating-position x=24 y=24 relative-to="top-right"; block-out-from "screen-capture" }
- Do NOT depend on the `niri-ipc` crate (GPL-3.0). Niri IPC (later) = raw JSON over `$NIRI_SOCKET`.
- Plugin boundary (5.1): out-of-process plugins, JSON-RPC 2.0 over stdio, versioned `lira-plugin-protocol v1`, declared in
  `~/.config/lira/plugins/*.toml`, capabilities: stt, tts, adapter, sync, team. Public trait crates use semver + `#[non_exhaustive]`.
- AgentAdapter trait (M2 implements claude-code): list_sessions, send(session, msg), status, summarize, events().
- Detect session at runtime from XDG_SESSION_TYPE, XDG_CURRENT_DESKTOP, NIRI_SOCKET, HYPRLAND_INSTANCE_SIGNATURE.

## M0 deliverables
Workspace layout (Cargo workspace, edition 2021, `license = "MIT OR Apache-2.0"` in workspace.package):
- crates/core (lira-core): `ipc` (Unix socket server/client, line-delimited JSON request/response, `Command::{Toggle,Start,Stop,Status}`,
  socket path from XDG_RUNTIME_DIR with fallback to std::env::temp_dir), `settings` (TOML at `dirs::config_dir()/lira/config.toml`,
  `hotkey_mode = "toggle" | "hold"`, default toggle; `hold` accepted but only a stub; load creates defaults if missing; save),
  `i18n` (locales pt-BR and en; detect from LC_ALL/LC_MESSAGES/LANG; fallback en; tiny static tables, no heavy deps), `state` (Idle/Listening).
- crates/audio (lira-audio): stub only (no cpal yet).
- crates/stt (lira-stt): trait `Transcriber` stub.
- crates/adapters/api (lira-adapter-api): trait `AgentAdapter` (async via `async-trait` or boxed futures) + `FakeAdapter` + contract tests.
- crates/adapters/claude-code (lira-adapter-claude-code): empty stub crate implementing nothing yet (README says M2).
- crates/compositor (lira-compositor): `detect_session()` from env vars -> enum {Niri, Hyprland, Kde, Gnome, X11, Unknown}; niri module stub.
- crates/plugin-protocol (lira-plugin-protocol): serde types for JSON-RPC 2.0 request/response/error, `PROTOCOL_VERSION = "1"`,
  plugin manifest struct (TOML) with capabilities enum; round-trip tests.
- cli (crate `lira-cli`, binary `lira`): clap; subcommands toggle|start|stop|status; prints localized messages;
  exit 1 with localized "Lira is not running" if socket absent. Integration test: start core ipc server on a temp socket, run client.
- app/src-tauri (crate `lira-app`, Tauri 2): on start bind the socket (core ipc server), handle commands: toggle shows/creates or
  hides/destroys popup window "popup"; start/stop = explicit; status returns state. Optional tray (tray-icon feature) wrapped so errors
  are logged and ignored; setting/env `LIRA_NO_TRAY=1` disables it. Reads settings at start.
- app/ui: Svelte 5 + Vite + TypeScript, pnpm. One tiny popup showing localized "Listening…/Idle" (pt-BR/en JSON) and hotkey_mode.
  Icons in app/src-tauri/icons/ are provided already.
- packaging/niri/lira.kdl: binds + window-rule snippets above.
- scripts/check.sh: `pnpm --dir app/ui install --frozen-lockfile && pnpm --dir app/ui build`, `cargo fmt --all --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo build --workspace`,
  then scripts/check-dco.sh (every commit in `${DCO_BASE:-main}..HEAD` has `Signed-off-by:` matching author name+email).
- .github/workflows/ci.yml: ubuntu-24.04, on pull_request + push to main, concurrency cancel-in-progress, apt deps
  (libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev libssl-dev libxdo-dev build-essential pkg-config),
  dtolnay/rust-toolchain stable with rustfmt+clippy, Swatinem/rust-cache, pnpm + node with pnpm cache, then run scripts/check.sh
  minus DCO; separate `dco` job on PRs with fetch-depth 0 using DCO_BASE=origin/${{ github.base_ref }}.
- CONTRIBUTING.md (DCO 1.1, `git commit -s`, how to add an adapter against FakeAdapter), README (what M0 is, how to run, Niri setup),
  lira.code-workspace (folders ".", recommended rust-analyzer + svelte extensions).
- Keep everything minimal. No dead code beyond clearly marked stubs. Clippy-clean.

Done = `scripts/check.sh` passes locally on Arch/Niri and CI green.

## Addendum (spec 5.2 settings, 5.4 i18n, 7.0.2 DankMaterialShell / dux-shell)
- Settings file `~/.config/lira/config.toml` is sectioned (spec 5.2). M0 schema (serde, all fields defaulted, unknown sections ignored):
  `[hotkey] mode = "toggle" | "hold"` (default toggle; hold = stub),
  `[ui] locale = "system" | "pt-BR" | "en"` (default system; i18n uses it, system = env detection),
  `[screen] context = "off" | "text" | "image"` (default off). Field + i18n strings only, NO implementation. UI copy: enabling it
  suggests "text"; "image" shows a token-cost warning. Put those strings (pt-BR/en) in both Rust and UI tables.
- i18n (5.4): no hardcoded user-facing strings in CLI or UI; add a test (Rust) and a script/test (UI, run by check.sh) asserting
  pt-BR and en have exactly the same keys. Add README.pt-BR.md (README stays in English).
- crates/stt: cargo features `cpu` (default), `cuda`, `vulkan`, all empty placeholders for M1.
- Tray (7.0.2): Fernando runs DankMaterialShell (now dux-shell), which hosts StatusNotifierItem with dbusmenu menus. On Linux,
  at startup check the session bus for `org.kde.StatusNotifierWatcher` (e.g. via `zbus` blocking call NameHasOwner, or skip zbus and
  just attempt tray creation). If present -> create tray with a small menu (Toggle / Quit, localized) and log
  "tray: StatusNotifierWatcher found, tray enabled"; if absent -> log "tray: no StatusNotifierWatcher, running without tray". Never fail.
