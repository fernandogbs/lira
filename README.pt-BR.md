# Lira

Aplicativo de bandeja/pop-up open source (MIT OR Apache-2.0) para falar por voz com agentes de código (Claude Code primeiro). Alvo principal: Linux no Niri (Wayland).

O **M0** é um scaffold: shell Tauri, CLI por socket Unix, pop-up Svelte mínimo e limites de crates para áudio, STT, adaptadores, detecção de compositor e plugins. Captura de voz e integração com Claude Code vêm em marcos posteriores.

Versão em inglês: [README.md](README.md).

## Executar (desenvolvimento)

```bash
# Terminal 1 — o servidor de UI do Vite é iniciado pelo Tauri
cd app/src-tauri && cargo tauri dev

# Ou construir o workspace
SKIP_DCO=1 ./scripts/check.sh
cargo build -p lira-cli
```

## CLI (atalho / spawn)

Com o app em execução, controle pelo socket Unix em `$XDG_RUNTIME_DIR/lira.sock`:

```bash
lira toggle   # mostra/oculta o pop-up e alterna o estado (M0)
lira start
lira stop
lira status
```

No Niri, vincule o toggle sem repeat (sem key-release global):

```kdl
Mod+Space repeat=false { spawn "lira" "toggle"; }
```

Veja `packaging/niri/lira.kdl` para a regra de janela recomendada (`app-id` `lira`, flutuante no canto superior direito, `block-out-from "screen-capture"`).

Bandeja opcional: no Linux o ícone só é criado quando `org.kde.StatusNotifierWatcher` está no barramento de sessão (ex.: DankMaterialShell / dux-shell). Use `LIRA_NO_TRAY=1` para desativar. Falhas na bandeja são registradas em log e nunca encerram o app.

## Configuração

`~/.config/lira/config.toml` (por seções, spec 5.2):

```toml
[hotkey]
mode = "toggle"   # ou "hold" (aceito; hold é stub no M0)

[ui]
locale = "system" # ou "pt-BR" | "en" (system = detecção por LC_* / LANG)

[screen]
context = "off"   # ou "text" | "image" (M0: só config e textos na UI, sem captura)
```

## Licença

Licenciado sob MIT ou Apache-2.0, à sua escolha (`LICENSE-MIT`, `LICENSE-APACHE`).
