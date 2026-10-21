# Lira

Aplicativo de bandeja/pop-up open source (MIT OR Apache-2.0) para falar por voz com agentes de código (Claude Code primeiro). Alvo principal: Linux no Niri (Wayland).

O **M1** entrega push-to-talk e transcrição local no Niri: toggle ou hold via binds da CLI, captura com cpal, VAD Silero, whisper.cpp (`small-q5_1` padrão), pop-up layer-shell com o texto, cópia para a área de transferência e histórico SQLite opcional. Integração com Claude Code é o **M2**; captura de tela é o **M3**.

Versão em inglês: [README.md](README.md).

## Executar (desenvolvimento)

```bash
# Terminal 1 — o servidor de UI do Vite é iniciado pelo Tauri
cd app/src-tauri && cargo tauri dev

# Ou construir o workspace
SKIP_DCO=1 ./scripts/check.sh
cargo build -p lira-cli
```

Checklist manual do M1 (Niri): `./scripts/manual-test-m1.sh`

## CLI (atalho / spawn)

Com o app em execução, controle pelo socket Unix em `$XDG_RUNTIME_DIR/lira.sock`:

```bash
lira toggle   # alterna gravação (apertar de novo para parar / VAD)
lira start    # inicia gravação (hold / bind de pressionar)
lira stop     # para e transcreve (hold / bind de soltar)
lira status
```

No Niri (sem portal GlobalShortcuts e sem bind de release), use **toggle** por padrão:

```kdl
Mod+Space repeat=false { spawn "lira" "toggle"; }
```

Veja `packaging/niri/lira.kdl` para a regra de janela recomendada (`app-id` `lira-app`, flutuante no canto superior direito, `block-out-from "screen-capture"`).

**Modo hold:** defina `[hotkey] mode = "hold"` e escolha `hold_backend`:

| Backend | Quando | Como |
|---------|--------|------|
| `compositor` (padrão no Niri) | Hyprland `bind`/`bindr` | Ver `packaging/hyprland/lira.conf` |
| `portal` | KDE / GNOME / portal Hyprland | O app pede GlobalShortcuts; activate→start, deactivate→stop |
| `evdev` | Opt-in no Niri | Rode `lira-hold` com `evdev_acknowledged = true`. Exige grupo `input`. **Nunca automático.** |

No Niri, prefira toggle + VAD. Leia o aviso de segurança do `lira-hold` antes de ativar evdev.

Bandeja opcional: no Linux o ícone só é criado quando `org.kde.StatusNotifierWatcher` está no barramento de sessão (ex.: DankMaterialShell / dux-shell). Use `LIRA_NO_TRAY=1` para desativar. Falhas na bandeja são registradas em log e nunca encerram o app.

## Configuração

`~/.config/lira/config.toml` (por seções, spec 5.2):

```toml
[hotkey]
mode = "toggle"           # ou "hold" (VAD desligado no hold)
vad_autostop_ms = 1500    # só no toggle; 0 desliga
hold_backend = "compositor"  # ou "portal" | "evdev"
# hold_key = "KEY_SPACE"   # para lira-hold
# evdev_acknowledged = false

[ui]
locale = "system"         # ou "pt-BR" | "en"
popup = "auto"            # ou "layer_shell" | "floating"

[stt]
engine = "whisper"
model = "small-q5_1"      # padrão pós-benchmark M1; "large-v3-turbo-q5_0" para qualidade
language = "auto"         # ou "pt" | "en" (auto segue o idioma da interface)
vocabulary = []           # termos extras no prompt inicial do Whisper
keep_warm_min = 5

[history]
enabled = true

[screen]
context = "off"           # ou "text" | "image" (captura no M3)
```

Os modelos são baixados no primeiro uso (SHA-256) no diretório de dados XDG.

## Licença

Licenciado sob MIT ou Apache-2.0, à sua escolha (`LICENSE-MIT`, `LICENSE-APACHE`).
