# M1 smoke checklist

Run after building `lira` and `lira-app` (`cargo build -p lira-cli -p lira-app --release` or `./scripts/manual-test-m1.sh`).

## Niri + DankMaterialShell (primary)

- [ ] App starts without tray panic when DMS tray is present or absent (`LIRA_NO_TRAY=1` still works)
- [ ] Niri bind `spawn "lira" "toggle"` starts listening; second press (or VAD silence) yields transcript in popup
- [ ] Transcript is copied to clipboard
- [ ] Idle: mic closed; CPU near 0% (`./scripts/measure-idle.sh` optional)
- [ ] Hold on Niri: prefer toggle. For evdev: set `hold_backend = "evdev"`, `evdev_acknowledged = true`, run `lira-hold`, confirm warning text

## Hyprland

- [ ] Source `packaging/hyprland/lira.conf` (or equivalent `bind`/`bindr`)
- [ ] `[hotkey] mode = "hold"`, `hold_backend = "compositor"`
- [ ] Press Super+Space starts listening; release stops and transcribes

## KDE / GNOME (portal)

- [ ] `[hotkey] mode = "hold"`, `hold_backend = "portal"`
- [ ] App requests GlobalShortcuts; user confirms shortcut in portal dialog
- [ ] Activate starts listening; deactivate stops and transcribes
- [ ] If portal unavailable, log warns and CLI binds still work

## Regression

- [ ] `SKIP_DCO=1 ./scripts/check.sh`
- [ ] pt-BR / en UI labels still match (`pnpm --dir app/ui run check:i18n` via check.sh)
