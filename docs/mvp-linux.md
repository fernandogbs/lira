# Linux MVP (M1–M3)

Status as of tag `v0.2.0`: Linux MVP is complete on `main`.

## Delivered

| Marco | Status |
|-------|--------|
| M0 scaffold + CI | Done |
| M1 PTT + local STT + hold backends | Done |
| M2 Claude Code adapter + router | Done |
| M3 opt-in screen OCR | Done |
| M4 Windows | Out of scope for this MVP |

## Manual smoke (primary: Niri + DMS)

1. Build: `cargo build --release -p lira-cli -p lira-app`
2. Run `lira-app`; bind `lira toggle` in Niri (`packaging/niri/lira.kdl`)
3. Toggle → speak → transcript in popup (+ clipboard if enabled)
4. With `claude` on PATH and `[agent] enabled = true`: instruction → agent reply + notification
5. Optional: `[screen] context = "text"` → OCR appended to agent prompt; excluded apps skip capture
6. `lira sessions` lists stored sessions; `lira setup niri` prints block-out rules

See also `docs/m1-smoke.md` for multi-DE hold checks.

## Next

- M4 Windows
- Phase 2: TTS, spoken approval, vision image upload
- MCP server, DMS plugin, Flatpak (later)
