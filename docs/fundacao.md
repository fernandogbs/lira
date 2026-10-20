# Fundação: Lira, assistente de voz na bandeja para agentes (open source)

Nome: **Lira**. Repositório do núcleo: `fernandogbs/lira`, **privado até o M1 funcionar e público depois**. Add-ons pagos em `fernandogbs/lira-pro`, fechado. Hoje o `fernandogbs/lira` ainda não existe (404) e o nome `lira` está livre no AUR. Aviso de proximidade: existe o [amd/LIRA](https://github.com/amd/LIRA) (MIT, 29 estrelas, último push em 06/2026), uma ferramenta da AMD para rodar modelos de transcrição na NPU do Ryzen AI. É de baixa visibilidade, mas é do mesmo domínio (ASR). Para evitar confusão em buscas, usar um nome descritivo no README e nos pacotes (ex.: "Lira: voice for your coding agents"). Pesquisa feita em 20/10/2026 (horário de Bahia, UTC-3).
As estrelas e a última atividade vieram da API do GitHub nesse dia. O que não consegui confirmar está marcado como **[não verificado]**.

---

## 1. Resumo direto

- **Ditado por voz já está resolvido em open source.** O Handy (Rust + Tauri, MIT, ~34 mil estrelas) faz exatamente "aperta o atalho, fala, o texto aparece", nos três sistemas e offline. Concorrer com ele é perda de tempo.
- **Falar com o Claude Code também já existe.** O próprio Claude Code tem `/voice` com push-to-talk e consegue ditar para sessões em segundo plano. Mas só dentro do terminal, só com conta Claude.ai, e o áudio vai para a nuvem da Anthropic. O VoiceMode (MCP, MIT, ~1,4 mil estrelas) e o backtalk dão conversa falada com o Claude Code.
- **O espaço livre é uma camada de comando do sistema inteiro, independente de agente, para sessões que já estão rodando.** Exemplos: "como tá a sessão do backend?", "manda o Codex rodar os testes depois que terminar", "resume o que mudou". Tudo com STT local, contexto de tela opcional e aprovação segura. Ninguém faz isso bem e com vários agentes ao mesmo tempo.
- **Decisões já tomadas:**
  - fase 1 só com Claude Code (com a interface de adaptador pronta para Codex/ACP);
  - **open core**: núcleo MIT OR Apache-2.0 com DCO e add-ons pagos num repositório fechado;
  - prioridade **Linux (Niri + DankMaterialShell primeiro) → Windows → macOS**, com KDE, GNOME e Hyprland também suportados;
  - modo do atalho configurável: `toggle` (padrão no Niri) ou `hold` (opt-in via `lira-hold`);
  - **pt-BR e inglês** desde o início, tanto na interface quanto na transcrição.
- **Recomendação:** não fazer fork do Handy, mas usar ele como referência (mesma stack, licença MIT) e talvez as crates dele. Tudo que é ditado genérico fica fora do escopo.

---

## 2. Panorama

### 2.1 Open source: ditado (aperta, fala, cola)

| Projeto | Estrelas | Licença | Última atividade | Stack | Observação |
|---|---|---|---|---|---|
| [cjpais/Handy](https://github.com/cjpais/Handy) | 34.214 | MIT | 20/10/2026 | Rust + Tauri 2 | Referência do setor. Whisper e Parakeet V3, VAD Silero, `tauri-plugin-global-shortcut`, `rdev`, `handy-keys`. No Wayland, o atalho é configurado no ambiente gráfico e chama a CLI do Handy. |
| [OpenWhispr/openwhispr](https://github.com/OpenWhispr/openwhispr) | 9.654 | MIT | 20/10/2026 | Electron + React | Modelos locais (Parakeet/Whisper) ou nuvem com chave própria. Por ser Electron, gasta mais RAM. |
| [Beingpax/VoiceInk](https://github.com/Beingpax/VoiceInk) | 6.873 | GPL-3.0 (segundo o README) | 20/10/2026 | Swift, só macOS | Tem "modos" por app e contexto de tela. Open source, mas atualizações automáticas exigem licença paga. |
| [EpicenterHQ/epicenter](https://github.com/EpicenterHQ/epicenter) (antigo Whispering) | 4.825 | outra (NOASSERTION) | 20/10/2026 | TypeScript | Agora é um conjunto de apps local-first. |
| [kitlangton/Hex](https://github.com/kitlangton/Hex) → [anomalyco/hex](https://github.com/anomalyco/hex) | 2.901 (arquivado) / 415 | MIT | 20/10/2026 | Swift → Rust | Ditado para macOS reescrito em Rust nativo. |
| [savbell/whisper-writer](https://github.com/savbell/whisper-writer) | 1.106 | GPL-3.0 | 08/2024 | Python | Parado. |

### 2.2 Open source: voz para agentes de código ou LLMs

| Projeto | Estrelas | Licença | Última atividade | Stack | Observação |
|---|---|---|---|---|---|
| [mbailey/voicemode](https://github.com/mbailey/voicemode) | 1.410 | MIT | 17/10/2026 | Python, MCP | Conversa falada com o Claude Code e outros clientes MCP. Whisper e Kokoro locais, OpenAI como reserva. O agente é quem "chama" a voz, não existe um atalho global do sistema. |
| [jaredrhod/backtalk](https://github.com/jaredrhod/backtalk) | 85 | AGPL-3.0 | 30/08/2026 | Python | Segura a tecla, fala, ouve a resposta em cerca de 1 a 2 s. STT e TTS locais. Pede aprovação falada antes de agir. Só funciona com o Claude Code. |
| [ethanplusai/jarvis](https://github.com/ethanplusai/jarvis) | 896 | outra | 09/2026 | Python | Assistente de voz para o Claude Code no Mac. |
| [noisy/noisy-studio](https://github.com/noisy/noisy-studio) | 13 | MIT | 10/2026 | Python | Daemon de voz e painel que roteiam a fala para o Claude Code. Pequeno, mas próximo da ideia de rotear. |
| [kornkutan/tars-voice](https://github.com/kornkutan/tars-voice) | 1 | MIT | 08/2026 | Rust | Push-to-talk para o Pi Coding Agent. |
| [isair/jarvis](https://github.com/isair/jarvis) | 2.116 | outra | 19/10/2026 | Python | Assistente local com voz, memória e MCP. Fica ouvindo o ambiente o tempo todo, o oposto do que queremos. |
| [openinterpreter/01](https://github.com/openinterpreter/01) | 5.161 | AGPL-3.0 | 11/2024 | Python | Interface de voz do Open Interpreter. Parado. |
| [leon-ai/leon](https://github.com/leon-ai/leon) | 17.595 | MIT | 20/10/2026 | TypeScript | Assistente pessoal genérico, não focado em agentes de código. |

### 2.3 Open source: atalho + contexto de tela + LLM

| Projeto | Estrelas | Licença | Última atividade | Stack | Observação |
|---|---|---|---|---|---|
| [farzaa/clicky](https://github.com/farzaa/clicky) | 7.735 | MIT | 28/04/2026 | Swift, macOS | "Professor" ao lado do cursor: vê a tela, fala e aponta. O autor congelou o repositório aberto e as novidades agora são fechadas (heyclicky.com). |
| [screenpipe/screenpipe](https://github.com/screenpipe/screenpipe) | 22.004 | licença comercial própria (`LICENSE.md`), **não é OSI** | 20/10/2026 | Rust | Grava a tela **continuamente** e faz OCR. É o modelo "Recall" que queremos evitar. |
| [MohitGoyal09/BlinkAI](https://github.com/MohitGoyal09/BlinkAI), [jazzpher/arrow-assistant](https://github.com/jazzpher/arrow-assistant), [AxelXoket/VoxDesk](https://github.com/AxelXoket/VoxDesk) | 0–4 | variadas | 2026 | TS/Python | Projetos pequenos com voz, tela e LLM. Mostram demanda, mas não têm adoção. |

### 2.4 Bibliotecas que vamos usar

| Biblioteca | Estrelas | Licença | Papel |
|---|---|---|---|
| [ggml-org/whisper.cpp](https://github.com/ggml-org/whisper.cpp) | 54.513 | MIT | STT local |
| [tazz4843/whisper-rs](https://github.com/tazz4843/whisper-rs) | 945 | Unlicense | Bindings Rust (último push em 07/2025; o Handy migrou para `transcribe-cpp`, avaliar) |
| [k2-fsa/sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx) | 15.404 | Apache-2.0 | STT, TTS e VAD via ONNX, com Parakeet e streaming |
| [rhasspy/piper](https://github.com/rhasspy/piper) | 11.303 | MIT | TTS local (último push em 08/2025) |
| [agentclientprotocol/agent-client-protocol](https://github.com/agentclientprotocol/agent-client-protocol) | 4.473 | Apache-2.0 | Protocolo padrão entre editor e agente (Rust). Base dos adaptadores. |
| [tauri-apps/plugins-workspace](https://github.com/tauri-apps/plugins-workspace) | 1.834 | Apache-2.0 | Plugins global-shortcut, tray e autostart |
| [nashaofu/xcap](https://github.com/nashaofu/xcap) | 1.026 | Apache-2.0 | Captura de tela em Rust (macOS, Windows, X11, Wayland) |
| [NiiightmareXD/windows-capture](https://github.com/NiiightmareXD/windows-capture) | 519 | MIT | Windows Graphics Capture em Rust |
| [robertknight/ocrs](https://github.com/robertknight/ocrs) | 1.906 | Apache-2.0 | OCR em Rust puro, alternativa ao Tesseract |

### 2.5 Produtos fechados e recursos embutidos

- **Claude Code `/voice`** ([docs](https://code.claude.com/docs/en/voice-dictation)):
  - push-to-talk configurável e ditado para sessões em segundo plano pela agent view;
  - exige conta Claude.ai (não funciona com chave de API, Bedrock nem Vertex), transcreve na nuvem da Anthropic e não funciona via SSH;
  - só é acionado dentro do terminal: atalhos só com modificador não chegam a apps de terminal ([issue #86708](https://github.com/anthropics/claude-code/issues/86708)).
  - **É o concorrente mais direto, e é de graça.**
- **Wispr Flow** ([preços](https://wisprflow.ai/pricing)): grátis com 2.000 palavras por semana no desktop; Pro a US$15/mês (US$12/mês no anual). Nuvem.
- **Superwhisper** ([planos](https://superwhisper.com/docs/billing/plans)): grátis com Whisper local; Pro a US$8,49/mês, US$84,99/ano ou US$249,99 vitalício.
- **Apps desktop do ChatGPT e do Claude:** abrem com atalho global e aceitam voz **[não verificado nesta pesquisa: atalhos exatos e modo voz no desktop]**. Mas só conversam com o próprio chat, não com os seus agentes locais.

---

## 3. Diferencial

**"O rádio dos seus agentes":** um atalho do sistema inteiro para falar com **qualquer** sessão de agente que já está rodando, sem trocar de janela.

1. **Escolher a sessão por voz ou pelo contexto:**
   - "backend, …", "codex, …";
   - ou a sessão ligada ao repositório ou janela em foco.
2. **Quatro verbos que cobrem 90% do uso:**
   - perguntar ("como tá?", "tá travado em quê?");
   - instruir ou enfileirar ("quando terminar, roda os e2e");
   - resumir ("o que mudou desde as 15h?");
   - aprovar ou negar (só ações reversíveis, ver seção 9).
3. **Independente de agente:**
   - Claude Code, Codex, Gemini CLI e outros, via ACP quando existir;
   - pelo SDK ou modo headless da CLI;
   - um wrapper PTY como último recurso;
   - mais um servidor MCP próprio, para o agente "pedir a palavra" (notificar, perguntar).
4. **Local primeiro:** STT local por padrão e nada sai da máquina além do que vai para o agente que você já usa. Funciona com chave de API ou provedor próprio, ao contrário do `/voice`.
5. **Contexto de tela opcional, capturado só no instante do atalho** (seção 6).

O que **não** entra: ditado em qualquer campo de texto (já é do Handy), assistente pessoal genérico, escuta contínua.

**Leitura honesta:** o risco é a Anthropic e a OpenAI levarem o `/voice` para fora do terminal (app desktop ou bandeja). A defesa é ser multiagente e local, coisa que eles não têm incentivo para fazer.

---

## 4. Escopo do MVP

### Fase 1: transcrição (6 a 8 semanas com desenvolvimento por agentes, estimativa)
- App na bandeja, sem janela principal: um pop-up pequeno aparece no atalho e some depois.
- Push-to-talk (segurar) e toggle (tocar).
- STT local: whisper.cpp (`base`/`small`) ou Parakeet via sherpa-onnx. Modelo baixado na primeira execução.
- Adaptador v1: **só Claude Code** (headless `claude -p` / Agent SDK, retomando a sessão). A trait `AgentAdapter` e a pasta `crates/adapters/` já nascem prontas para receber Codex e ACP depois, com um agente fake nos testes para provar que a interface não está amarrada ao Claude.
- Registro de sessões: lista as sessões ativas, com nome, repositório e status.
- Resposta em texto no pop-up, mais uma notificação do sistema quando o agente terminar.
- Contexto de tela **desligado por padrão**, em modo OCR local com Tesseract (seção 6).
- Plataforma da fase 1: **Linux primeiro** (o Fernando usa EndeavourOS/Arch), depois Windows. macOS fica por último.
- **pt-BR e inglês** desde o início, na interface (i18n) e na transcrição (seção 5.3).

### Fase 2: voz
- TTS local (Piper ou Kokoro via sherpa-onnx) lendo as respostas frase por frase, com interrupção pelo atalho.
- Opcional: modo "realtime" com OpenAI Realtime ou Gemini Live, pago, com chave do usuário e desligado por padrão. O Claude não tem API de voz em tempo real, então para ele o fluxo é STT → Claude → TTS (cerca de 1 a 2 s, compatível com o que o backtalk relata).
- Aprovação falada para ações reversíveis.
- Captura de tela em modo imagem (visão).

### Fora do MVP
Mobile, palavra de ativação, sincronização na nuvem, gravação de reuniões, colar texto em qualquer app.

---

## 5. Arquitetura

```
┌──────────────────── lira (um processo) ────────────────────────┐
│  Tauri 2 shell: tray, global shortcut, pop-up webview (lazy)   │
│        │ eventos agrupados (≤30/s)                             │
│  ┌─────▼────────────── core (Rust, tokio) ───────────────────┐ │
│  │ hotkey ─► audio (cpal) ─► VAD ─► STT ─► intent router     │ │
│  │                                          │                │ │
│  │ screen ctx (opcional) ─► OCR/redação ────┤                │ │
│  │                                          ▼                │ │
│  │      session registry ◄──► adapters: claude (v1)          │ │
│  │      policy (aprovações)   | codex, acp, pty (depois)     │ │
│  │      store: SQLite (histórico, sessões, config)           │ │
│  │      TTS (fase 2) ◄── respostas em streaming              │ │
│  └───────────────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────────────┘
```

- **Núcleo em Rust** (`tokio`): toda a lógica fica aqui, e a interface só exibe.
- **Áudio:**
  - `cpal` captura em 16 kHz mono, com um buffer circular só enquanto o atalho está pressionado;
  - VAD Silero corta o silêncio.
- **STT:** uma trait `Transcriber`:
  - `WhisperCpp` (whisper-rs ou transcribe-cpp);
  - `SherpaOnnx` (Parakeet);
  - `OsNative` (fase 1.5): macOS Speech/SpeechAnalyzer **[não verificado: disponibilidade do SpeechAnalyzer por versão]**, Windows `Windows.Media.SpeechRecognition`;
  - `Cloud` (OpenAI, Groq, Deepgram) com chave do usuário.
- **Roteador de intenção:**
  - primeiro, regras ("<nome da sessão>, …", "status", "resume", "aprova");
  - na dúvida, um LLM pequeno classifica, com um modelo local opcional.
  - Nada de LLM por padrão em todo comando, para economizar custo e latência.
- **Adaptadores:** a trait `AgentAdapter` tem `list_sessions`, `send(session, msg)`, `status`, `summarize` e `events()`. O modelo é o mesmo do ACP, então um adaptador ACP cobre todo agente que fala ACP.
- **Servidor MCP próprio:** expõe `notify_user`, `ask_user` e `request_approval`, assim o agente consegue falar com você pela bandeja.
- **Política:** cada ação tem uma classe (`read`, `reversible`, `destructive`). `destructive` nunca é aprovada por voz.
- **Interface:** Tauri 2 com um único webview, criado só quando é usado e destruído ao fechar. Svelte ou SolidJS, sem framework pesado.

---

### 5.1 Fronteira de extensão (para os add-ons fechados)

O núcleo aberto precisa aceitar os add-ons do `lira-pro` **sem fork e sem `#ifdef` de negócio no código aberto**. Rust não tem ABI estável, então nada de carregar `.so` em tempo de execução. Há duas camadas:

1. **Protocolo de plugin fora do processo (a fronteira principal e estável):**
   - JSON-RPC 2.0 sobre stdio, no mesmo estilo de LSP e MCP, versionado (`lira-plugin-protocol v1`).
   - Um plugin é um executável declarado em `~/.config/lira/plugins/*.toml`.
   - Ele declara o que fornece: `stt` (STT na nuvem), `tts`/`voice` (voz em tempo real), `adapter` (adaptadores extras), `sync` e `team`.
   - O núcleo descobre, inicia sob demanda e mata o plugin quando fica ocioso (RAM zero quando não está em uso).
   - Isola as licenças (o código fechado nunca é ligado ao binário aberto), isola falhas, e permite plugins em qualquer linguagem.
2. **Crates públicas com traits estáveis**, para quem quer desempenho dentro do processo:
   - `lira-adapter-api` e `lira-stt-api`, com versionamento semântico, `#[non_exhaustive]` e testes de contrato.
   - O `lira-pro` pode compilar um binário "Lira Pro" que reúne essas crates abertas com as fechadas. A MIT/Apache permite isso, inclusive com contribuições recebidas via DCO.
   - Para áudio em tempo real, se o JSON-RPC ficar lento, usar socket Unix com frames binários. Decidir isso no benchmark da fase 2.

Regra de governança: **toda funcionalidade que existe no núcleo continua no núcleo**. O pago só acrescenta coisas (nuvem, tempo real, times), nunca tira nem limita o que já é aberto.

### 5.2 Configurações (o que o usuário controla)

Tudo fica em `~/.config/lira/config.toml` (editável à mão) e numa tela de configurações simples. Os padrões são conservadores.

| Grupo | Opção | Valores | Padrão |
|---|---|---|---|
| **Atalho** | `hotkey.mode` | `toggle` \| `hold` | `toggle` no Niri e onde não há release bind; `hold` onde o portal ou X11 suportam |
| | `hotkey.hold_backend` | `portal` \| `compositor` (`bindr` no Hyprland) \| `evdev` (`lira-hold`) | o melhor disponível; **`evdev` nunca é automático** |
| | `hotkey.vad_autostop_ms` | 0–5000 | 1500 no modo toggle |
| **Idioma** | `ui.locale` | `pt-BR` \| `en` \| `system` | `system` |
| | `stt.language` | `pt` \| `en` \| `auto` | igual à interface (`pt` se a interface estiver em pt-BR) |
| | `stt.vocabulary` | lista de termos | nome do repositório, branch e termos técnicos comuns |
| **Modelo** | `stt.engine` / `stt.model` | ver 5.3 | `whisper` / `large-v3-turbo-q5_0` |
| | `stt.keep_warm_min` | 0–60 | 5 (descarrega o modelo da RAM depois) |
| **Tela** | `screen.enabled` | bool | `false` |
| | `screen.scope` | `window` \| `screen` | `window` |
| | `screen.mode` | `ocr` \| `image` | `ocr` |
| | `screen.exclude_apps` | lista de app_id | gerenciadores de senha e bancos (6.5) |
| | `screen.preview` | `always` \| `first` \| `never` | `first` |
| **Agente** | `agent.default_session` | nome \| `focused` | `focused` (repositório da janela em foco) |
| **Aprovação** | `policy.voice_approve` | `off` \| `reversible` | `off` na fase 1 |

**Aviso obrigatório para o `hold` via evdev**, mostrado na tela e exigindo confirmação marcada:
> "Para segurar a tecla no Niri, o Lira precisa ler o teclado direto (`/dev/input`). Isso exige seu usuário no grupo `input`, o que dá a **qualquer programa seu** a capacidade de ler tudo que você digita, inclusive senhas. O `lira-hold` só escuta a tecla configurada e não grava nada, mas a permissão vale para o sistema inteiro. Recomendado: usar o modo toggle."

A tela traz um botão "Desativar e voltar ao toggle" e as instruções para sair do grupo (`gpasswd -d $USER input`). O Lira nunca roda `usermod` sozinho.

### 5.3 Transcrição local (STT) para pt-BR + en

**Dados verificados:**

| Modelo | Disco | RAM | PT (WER) | EN (WER) | Fonte |
|---|---|---|---|---|---|
| whisper `base` | 142 MiB | ~388 MB | **[não verificado]**, bem pior que o `small` | — | [whisper.cpp README](https://github.com/ggml-org/whisper.cpp#memory-usage) |
| whisper `small` | 466 MiB | ~852 MB | 10,9% (FLEURS pt, normalizado) | — | RAM: whisper.cpp; WER: [card remynd/whisper-large-v3-pt](https://huggingface.co/remynd/whisper-large-v3-pt) |
| whisper `medium` | 1,5 GiB | ~2,1 GB | 8,6% (FLEURS pt) | — | idem |
| whisper `large-v3-turbo-q5_0` | 547 MiB | **[não verificado]**, estimativa de ~0,8–1 GB | 3,3–3,8% (FLEURS pt); **6,7% num benchmark pt-BR** (o melhor do ranking) | 3,9% (FLEURS) | [models/README](https://github.com/ggml-org/whisper.cpp/blob/master/models/README.md), [Yap benchmark](https://yapapp.cc/data/on-device-asr-benchmark), [tech4humans PT-BR](https://huggingface.co/datasets/tech4humans/Audio-Transcription-Models-Comparison-PT-BR) |
| Parakeet TDT 0.6B v3 (int8, sherpa-onnx) | 487 MB (tar.bz2) | ~2 GB segundo a NVIDIA (versão fp32) | 4,76% (FLEURS pt), mas **11,6% no benchmark pt-BR** | 4,85% (FLEURS) | [card NVIDIA](https://huggingface.co/nvidia/parakeet-tdt-0.6b-v3), [release sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx/releases/tag/asr-models), tech4humans |

Fatos que pesam:
- **O Parakeet v3 tem português**, mas o card da NVIDIA avisa que **foi treinado em português europeu** ("our training data uses European Portuguese while most benchmarks use Brazilian Portuguese"). No benchmark pt-BR ele cai para 11,6%, contra 6,7% do Whisper turbo.
- O Parakeet **só detecta o idioma automaticamente**; não dá para fixar `pt`. Ele é muito mais rápido no CPU (é o "CPU-optimized" do Handy).
- O Whisper aceita idioma fixo e um *initial prompt* com vocabulário: nome do repositório, "PR", "deploy", "endpoint". Isso ajuda muito no pt-BR misturado com termos técnicos em inglês, que é o jeito que dev brasileiro fala.

**Recomendação:**
- **Padrão: whisper `large-v3-turbo-q5_0`**, com `stt.language` fixo (`pt` ou `en`) e o vocabulário no prompt. É o melhor para pt-BR, e o tamanho cabe no disco (547 MiB). A RAM só é usada enquanto o modelo está carregado (descarrega depois de `keep_warm_min`).
- **"Rápido / pouca RAM": whisper `small` q5** (~10,9% PT), para máquinas fracas.
- **Opcional: Parakeet v3** via sherpa-onnx, para quem fala mais inglês ou quer a menor latência no CPU. Aviso na tela: "menos preciso em pt-BR".
- **Detecção automática de idioma** fica como opção (`auto`), não como padrão. Em frases curtas a detecção erra, e fala em pt-BR com muito termo em inglês confunde a escolha. O padrão segue o idioma da interface, com um atalho ou comando para trocar (`lira lang en`).
- **Validar no M1** na máquina do Fernando, com 30 frases pt-BR e 30 en gravadas, medindo WER, latência para 10 s de áudio e RSS de pico. Se o turbo-q5 passar de ~3 s no CPU dele, o padrão vira o `small` e o turbo fica como opção de "qualidade".

### 5.4 Internacionalização (i18n)
- Interface em **pt-BR e en** desde o M0: textos em arquivos Fluent (`.ftl`) ou JSON por locale, com chaves no Rust (mensagens da CLI e notificações) e na UI. Nada de string fixa no código, e o CI verifica chaves faltando.
- Comandos de voz do roteador nos dois idiomas: "status"/"como tá", "resume"/"summarize", "aprova"/"approve", "cancela"/"cancel".
- Respostas do agente: o Lira pede ao Claude Code para responder no idioma da interface (instrução curta no prompt), a menos que o usuário fale em outro idioma.
- README e docs em inglês, com `README.pt-BR.md`.

## 6. Contexto de tela (opcional, ligado pelo usuário)

### 6.1 Princípios
- **Desligado por padrão.** Liga por app ou globalmente, e cada ativação pode ter seu próprio atalho (ex.: `Ctrl+Alt+Espaço` = voz, `Ctrl+Alt+Shift+Espaço` = voz + tela).
- **Captura só no instante em que o atalho é acionado.** Um quadro, nada de gravação contínua nem histórico de imagens. A imagem fica em memória, é descartada depois do envio e nunca vai para o disco. Só o texto que foi enviado fica no histórico, se o usuário quiser.
- **Contraste com o Microsoft Recall:**
  - Em 2024, o Recall tirava screenshots a cada poucos segundos e guardava o OCR num SQLite em texto puro, acessível a malware ou a outro usuário da máquina ([The Verge](https://www.theverge.com/2024/6/3/24170305/microsoft-windows-recall-ai-screenshots-security-privacy-issues), [Ars Technica](https://arstechnica.com/ai/2024/06/windows-recall-demands-an-extraordinary-level-of-trust-that-microsoft-hasnt-earned/)).
  - A Microsoft teve que tornar o recurso opt-in, criptografar os dados, exigir Windows Hello e adiar o lançamento ([Ars](https://arstechnica.com/gadgets/2024/06/microsoft-makes-recall-feature-off-by-default-after-security-and-privacy-backlash/), [The Verge](https://www.theverge.com/2024/6/13/24178144/microsoft-windows-ai-recall-feature-delay)).
  - O screenpipe segue o mesmo modelo contínuo. O nosso é o oposto, e isso precisa estar na primeira linha do README.
- **Indicador visível:** um flash na borda da área capturada, ícone da bandeja em vermelho por cerca de 1 s e uma miniatura no pop-up com "remover antes de enviar".

### 6.2 Escopo da captura
- **Janela ativa** (padrão): menos dados, menos tokens, menos risco.
- **Tela inteira** (o monitor onde está o cursor).
- **Região** (fase 2): arrastar para selecionar.

### 6.3 APIs e permissões por sistema

| SO | API | Permissão | Observação |
|---|---|---|---|
| macOS 12.3+ | ScreenCaptureKit (`SCScreenshotManager` para um quadro, macOS 14+) | "Gravação de Tela" em Privacidade, pedida na primeira vez. As versões recentes reexibem o aviso periodicamente **[não verificado: frequência no macOS atual]** | `xcap` cobre. Dá para filtrar janelas, inclusive para excluir apps antes de capturar. |
| Windows 10 1903+ | Windows.Graphics.Capture | Sem permissão para apps desktop. Mostra borda amarela no Windows 10; a borda pode ser desligada no 11 **[não verificado]** | `windows-capture` / `xcap`. Janelas com `SetWindowDisplayAffinity` (DRM, alguns apps de banco) saem pretas, o que é bom aqui. |
| Linux X11 | XGetImage / XComposite | Nenhuma | Qualquer app captura tudo, então o cuidado é nosso. |
| Linux Wayland | xdg-desktop-portal `Screenshot` (ou `ScreenCast` com PipeWire) | Diálogo do portal, com comportamento diferente em GNOME, KDE e wlroots | Captura da janela ativa nem sempre é possível. Na prática, tela inteira ou o usuário escolhe no diálogo. |

### 6.4 Dois modos de envio

| | **A. OCR local → só texto** (padrão) | **B. Imagem → modelo de visão** |
|---|---|---|
| Privacidade | Melhor: só o texto sai, depois de passar pela redação | Pior: o pixel inteiro sai |
| Custo | Centenas de tokens de texto | Até ~1.560 tokens (padrão) ou ~4.784 (alta resolução) por imagem |
| Qualidade | Bom para código, logs, erros e formulários; perde layout e gráficos | Entende UI, diagramas e gráficos |
| Motor | macOS: Vision (`VNRecognizeTextRequest`). Windows: `Windows.Media.Ocr`. Linux: Tesseract ou `ocrs` (Rust puro, Apache-2.0) | O modelo do adaptador (Claude, GPT, Gemini) |

Com OCR, a redação (6.5) roda sobre o texto, o que é muito mais confiável do que redigir pixels.

### 6.5 Redação e exclusões
- **Lista de exclusão por app/bundle ID**, com padrões que já vêm de fábrica:
  - 1Password, Bitwarden, KeePassXC, Keychain Access;
  - apps de banco, configurável, com uma lista inicial BR (Itaú, Bradesco, Nubank etc.);
  - janelas privadas e anônimas de navegador, quando detectáveis.
- Se o app em foco estiver na lista, **não captura** e avisa "tela não enviada (app protegido)".
- No macOS, o ScreenCaptureKit permite excluir janelas desses apps mesmo na captura de tela inteira.
- **Redação de texto** depois do OCR: regex para CPF/CNPJ, cartão (com Luhn), e-mails, tokens (`sk-…`, `ghp_…`, JWT, chaves AWS) e campos de senha visíveis.
- **Prévia obrigatória na primeira vez** e depois opcional: o usuário vê exatamente o que vai sair.

### 6.6 Custo por screenshot (Claude)

Fonte: [Claude vision docs](https://docs.claude.com/en/docs/build-with-claude/vision). Fórmula: `⌈largura/28⌉ × ⌈altura/28⌉` tokens visuais. Modelos padrão têm teto de 1568 px no lado maior e 1.568 tokens; os modelos 4.7+ são de alta resolução, com 2576 px e 4.784 tokens.

| Captura | Tokens (padrão) | Tokens (alta res.) | Custo/1.000 capturas a US$1/MTok (Haiku 4.5) | a US$4/MTok (Opus 5.5) |
|---|---|---|---|---|
| Janela 1000×1000 | 1.296 | 1.296 | ~US$1,30 | ~US$5,18 |
| Tela 1920×1080 | 1.560 | 2.691 | ~US$1,56 | ~US$10,76 |
| Tela 4K | 1.560 | 4.784 | ~US$1,56 | ~US$19,14 |
| OCR de uma tela de código (~300–800 tokens de texto, estimativa) | — | — | ~US$0,30–0,80 | ~US$1,20–3,20 |

Os valores de US$1,30, US$5,18 e US$19,14 estão na documentação; o resto foi calculado com a mesma fórmula. Regra do app: **reduzir a imagem para no máximo 1568 px no lado maior antes de enviar**, mesmo para modelos de alta resolução, a menos que o usuário peça "alta fidelidade". Isso corta até 3x o custo. Importante: no Claude Code com assinatura, isso consome a cota do plano, não dinheiro direto.

---

## 7. Plataformas: Linux como alvo principal

O Fernando usa **Niri** (compositor Wayland com tiling rolável) no EndeavourOS, e ele é o **alvo principal de desenvolvimento e teste**. KDE, GNOME e Hyprland também precisam funcionar. O código detecta a sessão em tempo de execução (`XDG_SESSION_TYPE`, `XDG_CURRENT_DESKTOP`, `NIRI_SOCKET`, `HYPRLAND_INSTANCE_SIGNATURE`) e escolhe o backend certo para cada capacidade.

### 7.0 Niri (alvo principal)

Fontes: [repositório niri-wm/niri](https://github.com/niri-wm/niri) (antigo YaLTeR/niri; ~28,5 mil estrelas, GPL-3.0, release mais recente [v26.04](https://github.com/niri-wm/niri/releases) de 25/04/2026) e a wiki em `docs/wiki/` (lida no commit `aee25c9`, de 20/10/2026).

**1. Atalho global: o portal GlobalShortcuts não existe no Niri hoje.**
- O Niri usa o `xdg-desktop-portal-gnome` como backend ([niri-portals.conf](https://github.com/niri-wm/niri/blob/main/resources/niri-portals.conf): `default=gnome;gtk;`), mas o GlobalShortcuts desse portal precisa de um provedor no compositor que o Niri ainda não implementa.
- Há PRs abertos: [#3587 "feat: GlobalShortcut portal"](https://github.com/niri-wm/niri/pull/3587) (draft, criado em 03/2026) e [#1165 `hyprland-global-shortcuts-v1`](https://github.com/niri-wm/niri/pull/1165) (WIP).
- **Bind de soltar tecla também não existe:** o PR [#3621 "Implement press and release keybinds + modifier-only binds"](https://github.com/niri-wm/niri/pull/3621) ainda está aberto. Hoje os binds disparam ao apertar (com repetição, que dá para desligar com `repeat=false`), e a [wiki de binds](https://github.com/niri-wm/niri/blob/main/docs/wiki/Configuration:-Key-Bindings.md) não documenta bind de release.
- **Plano para o Niri:**
  - **Modo toggle como padrão:** aperta para começar a ouvir, aperta de novo para enviar. Se o usuário esquecer, a VAD para sozinha depois de cerca de 1,5 s de silêncio.
  - Config sugerida (KDL, `~/.config/niri/config.kdl`):
    ```kdl
    binds {
        Mod+Space repeat=false hotkey-overlay-title="Lira: falar" { spawn "lira" "toggle"; }
        Mod+Shift+Space repeat=false hotkey-overlay-title="Lira: falar + tela" { spawn "lira" "toggle" "--screen"; }
        Mod+Ctrl+Space repeat=false hotkey-overlay-title="Lira: cancelar" { spawn "lira" "cancel"; }
    }
    ```
  - **Segurar para falar de verdade, sem suporte do compositor:** um helper opcional `lira-hold` lê o teclado direto via evdev/libinput (`/dev/input/event*`). Isso exige o usuário no grupo `input`, o que dá ao processo a capacidade de ler **todas** as teclas do sistema, ou seja, o mesmo poder de um keylogger. Por isso só entra como opt-in explícito, filtrando uma única tecla, sem log e com o aviso no README. Não vem ligado.
  - Quando o #3621 ou o #3587 forem aceitos, a Lira passa a usar o release bind ou o portal automaticamente, sem mudar a arquitetura.

**2. Captura de tela no Niri:**
- **Janela em foco:** `niri msg --json focused-window` (ou pelo socket `$NIRI_SOCKET`) devolve `id`, `title`, `app_id`, `pid`, `workspace_id` e `is_floating` (struct `Window` em [niri-ipc](https://github.com/niri-wm/niri/blob/main/niri-ipc/src/lib.rs)). Isso alimenta a **lista de exclusão antes de capturar** e a escolha de sessão por contexto (pelo `pid` → cwd → repositório).
- **Captura da janela sem diálogo:** a ação IPC `screenshot-window --id <id> --write-to-disk <bool> --path <abs>` existe no niri-ipc. Porém o screenshot **sempre vai também para a área de transferência**, o que suja o clipboard do usuário. Mitigação: salvar com `--path` em `$XDG_RUNTIME_DIR` (tmpfs), apagar logo depois de ler e restaurar o clipboard anterior **[não verificado: em qual release o `--path` entrou e se dá para evitar o clipboard]**.
- **Portal Screenshot / ScreenCast:** o Niri implementa as interfaces D-Bus `org.gnome.Shell.Screenshot` e `org.gnome.Mutter.ScreenCast` (`src/dbus/`), então os portais do `xdg-desktop-portal-gnome` funcionam. A [wiki de screencasting](https://github.com/niri-wm/niri/blob/main/docs/wiki/Screencasting.md) confirma o ScreenCast via portal + PipeWire, de monitores e de janelas. Esse é o caminho portátil (o mesmo do GNOME), mas pode abrir diálogo.
- **wlr-screencopy:** o Niri implementa `zwlr_screencopy_manager_v1` (é o que o `grim` usa nos exemplos da wiki). Captura o monitor inteiro sem diálogo.
- **Exclusão feita pelo próprio compositor:** a window rule [`block-out-from "screen-capture"`](https://github.com/niri-wm/niri/blob/main/docs/wiki/Configuration:-Window-Rules.md) pinta a janela de preto em **todas** as capturas, inclusive de ferramentas de terceiros e das ações `screenshot-screen`/`screenshot-window`. A Lira pode gerar esse trecho para gerenciadores de senha e bancos. É uma camada a mais de proteção, além da lista própria.
- **Ordem na Lira:** IPC `screenshot-window` (só a janela, sem diálogo) → screencopy (tela inteira) → portal (reserva).

**3. Bandeja:** o Niri não tem barra própria (a config padrão inicia a Waybar, segundo [Integrating-niri](https://github.com/niri-wm/niri/blob/main/docs/wiki/Integrating-niri.md)). **O Fernando usa DankMaterialShell (DMS) no lugar da Waybar** (ver 7.0.2). Mesmo assim, **o Lira tem que funcionar 100% sem bandeja**: atalho, pop-up e CLI (`lira status`, `lira sessions`, `lira settings` abre a janela). A bandeja é só um extra.

**4. Pop-up:**
- **Opção A, layer-shell** (`gtk-layer-shell`, que o Handy já usa): camada `overlay`/`top` com `keyboard-interactivity on-demand`, ancorada num canto. Fica acima de tudo, não entra no tiling e não rola com as colunas. A [wiki](https://github.com/niri-wm/niri/blob/main/docs/wiki/Layer‐Shell-Components.md) avisa que nas camadas bottom e background o foco exclusivo só funciona sem janelas, então usar top ou overlay. Dá para estilizar com layer rules (`namespace "lira"`, `block-out-from`, `shadow`).
- **Opção B, janela normal flutuante:** app-id `lira`, com a window rule `open-floating true` + `default-floating-position` (a [wiki de janelas flutuantes](https://github.com/niri-wm/niri/blob/main/docs/wiki/Floating-Windows.md) diz que elas ficam sempre acima das tiladas):
  ```kdl
  window-rule {
      match app-id="^lira$"
      open-floating true
      default-floating-position x=24 y=24 relative-to="top-right"
      block-out-from "screen-capture"
  }
  ```
- **Decisão:** layer-shell no Niri, Hyprland e KDE. Janela normal no GNOME (o Mutter não implementa wlr-layer-shell) e como reserva geral. O `lira setup niri` imprime os trechos KDL acima, mas não edita a config do usuário sozinho.

### 7.0.2 DankMaterialShell (DMS) no Niri (ambiente do Fernando)

Fonte: repositório [AvengeMedia/DankMaterialShell](https://github.com/AvengeMedia/DankMaterialShell), que **agora redireciona para [duxlabs-dev/dux-shell](https://github.com/duxlabs-dev/dux-shell)** ("Dux Shell"; ~8,5 mil estrelas; **MIT**, relicenciado de GPL-3.0 para MIT, segundo `quickshell/LICENSE_CHANGE_12_11_2025.md`; versão `v2.0.0`; pacote `dms-shell` no repositório extra do Arch). Código lido no commit `3284aca` de 20/10/2026. É feito em Quickshell (QML) + Go e substitui Waybar, mako, fuzzel, swaylock etc.

- **Bandeja:** **sim, mostra ícones StatusNotifierItem com menus.**
  - O widget `Modules/DBar/Widgets/SystemTrayBar.qml` usa `Quickshell.Services.SystemTray`.
  - O clique chama `activate()` (a não ser que o item seja `onlyMenu`) e abre o menu D-Bus (`item.menu`) quando `hasMenu`. Também há um slot de bandeja na "island" compacta.
  - **Ressalvas** (issues já fechados, mas mostram onde quebra):
    - [#811](https://github.com/duxlabs-dev/dux-shell/issues/811): ícones de alguns apps não apareciam;
    - [#1719](https://github.com/duxlabs-dev/dux-shell/issues/1719): o Dropbox reclamava de falta de AppIndicator depois da 1.4;
    - [#3728](https://github.com/duxlabs-dev/dux-shell/issues/3728): menu vazio quando o item estava no overflow.
  - O Tauri usa `libayatana-appindicator` (SNI + dbusmenu), então deve funcionar. **[não verificado: testar o ícone e o menu do Tauri no DMS no M0.]**
- **IPC:** `dms ipc call <alvo> <função>` (são 33 alvos em `quickshell/DMSShellIPC.qml`, [docs](https://duxlabs.dev/docs/dms/keybinds-ipc)). Úteis para o Lira:
  - `toast` e o serviço de notificações do próprio DMS, que é o daemon freedesktop (as notificações "agente terminou" do Lira aparecem nele);
  - `mic` (status e mute do microfone, para mostrar e respeitar o mute);
  - `spotlight openQuery/toggleQuery` (abrir o launcher já com um texto).
- **Plugins:** o DMS carrega plugins QML de `~/.config/dms/plugins/`, com tipos `widget`, `daemon`, `launcher` (por trigger) e outros ([PLUGINS/README.md](https://github.com/duxlabs-dev/dux-shell/blob/master/quickshell/PLUGINS/README.md)). **Oportunidade:** um plugin `lira-dms` pequeno e opcional (MIT, repositório próprio) com:
  - (a) um widget na barra mostrando o estado (ouvindo, transcrevendo, sessão ativa) via `lira status --json` ou o socket;
  - (b) um launcher com trigger (ex.: `>`) para mandar texto ou escolher a sessão sem voz.
  - Fica **depois do M2**, porque não é necessário para o núcleo.
- **Atalhos:** continuam no `config.kdl` do Niri (o DMS tem uma tela de keybinds, mas o bind do Lira é um `spawn` comum).

### 7.0.1 Matriz de suporte no Linux

| Capacidade | **Niri** (principal) | **DMS sobre Niri/Hyprland** (shell do Fernando) | KDE Plasma 6 | GNOME 48+ | Hyprland | X11 (qualquer) |
|---|---|---|---|---|---|---|
| Atalho global | Bind no config → `lira toggle` (portal: PR #3587 aberto) | Igual ao compositor (bind no Niri) | Portal GlobalShortcuts | Portal GlobalShortcuts (49+ recomendado) | Portal (`xdg-desktop-portal-hyprland`) ou `bind`/`bindr` | `tauri-plugin-global-shortcut` |
| Segurar para falar | Não nativo (PR #3621); toggle + VAD ou `lira-hold` opt-in | Igual ao compositor | Sim (activate/deactivate) | Sim | Sim (`bindr` ou portal) | Sim |
| Captura da janela ativa | IPC `screenshot-window` | Igual ao compositor | Portal (com diálogo) **[não verificado: captura silenciosa da janela ativa]** | Portal (com diálogo) | `grim -g` com a geometria do `hyprctl` | `xcap` |
| Detectar janela ativa | `niri msg --json focused-window` | Igual ao compositor | Script KWin via D-Bus **[não verificado]** | Sem API pública; precisa de extensão **[não verificado]** | `hyprctl activewindow -j` | `_NET_ACTIVE_WINDOW` |
| Bandeja (SNI) | Só com uma barra que tenha tray (DMS, Waybar etc.) | **Sim, nativa, com menus** (Quickshell SystemTray) **[testar o Tauri no M0]** | Nativa | Precisa da extensão AppIndicator | Waybar | Depende do WM |
| Pop-up | layer-shell | layer-shell (o DMS também é layer-shell; testar sobreposição) | layer-shell | Janela normal | layer-shell | Janela normal |

### 7.1 Atalho global
- **Wayland:** o plugin oficial do Tauri não funciona ([issue #28](https://github.com/tauri-apps/global-hotkey/issues/28) aberta; PRs [#162](https://github.com/tauri-apps/global-hotkey/pull/162) e [#172](https://github.com/tauri-apps/global-hotkey/pull/172) ainda abertos).
  - Usar o portal `org.freedesktop.portal.GlobalShortcuts` via `ashpd` ou [tauri-plugin-xdg-portal](https://docs.rs/crate/tauri-plugin-xdg-portal/latest/source/README.md).
  - Suporte: KDE Plasma 5.27+, GNOME 48+ (o bug de BindShortcuts foi corrigido no 49), Hyprland. **sway/wlroots e COSMIC ainda não têm.**
  - O portal manda eventos de activate e deactivate, então o "segurar para falar" funciona.
  - O usuário confirma o atalho num diálogo do compositor, e pode trocá-lo depois nas configurações do sistema. O app tem que ler o atalho que ficou valendo (`ListShortcuts`) e mostrá-lo.
- **Niri:** ver 7.0 (bind no config + `lira toggle`).
- **Reserva universal:** o comando `lira toggle` (e também `lira start`/`lira stop` para segurar) fala com o processo pelo socket Unix em `$XDG_RUNTIME_DIR/lira.sock`. O usuário liga esse comando a um atalho do próprio compositor. Exemplo no Hyprland: `bind = SUPER, V, exec, lira start` + `bindr = SUPER, V, exec, lira stop`. É o mesmo padrão do Handy e cobre sway e i3.
- **X11:** `tauri-plugin-global-shortcut` funciona direto. Para tecla só com modificador, usar `rdev`.

### 7.2 Áudio
- `cpal` sobre ALSA, que no Arch atual passa pelo PipeWire (`pipewire-alsa`/`pipewire-pulse`). Capturar em 16 kHz mono e abrir o microfone só durante o atalho.
- Fase 2: avaliar o backend PipeWire nativo (`pipewire-rs`) para escolher o dispositivo e reduzir latência.

### 7.3 Captura de tela (opt-in)
- **Wayland:** portal `Screenshot` para um quadro, que é o padrão. O portal `ScreenCast` + PipeWire fica para quando for preciso escolher uma janela específica (KDE e GNOME oferecem seleção de janela, mas o comportamento varia). Na primeira vez aparece o diálogo do portal, e no KDE e no GNOME dá para lembrar a permissão **[não verificado: persistência em cada versão]**.
- **X11:** captura direta via `xcap`, sem permissão. Como qualquer app consegue capturar a tela no X11, a lista de exclusões (6.5) é a única proteção.
- **OCR:** Tesseract (`tesseract` + `tesseract-data-por` e `-eng` no Arch), chamado via `leptess` ou pelo binário. `ocrs` em Rust puro fica como alternativa sem dependência do sistema.

### 7.4 Bandeja
- StatusNotifierItem (D-Bus) via `libayatana-appindicator`, que é o que o Tauri usa no Linux. Funciona nativo no KDE, XFCE e na Waybar do Hyprland/sway (módulo `tray`).
- **No GNOME a bandeja não aparece sem a extensão "AppIndicator and KStatusNotifierItem Support".** Detectar isso e avisar no primeiro uso, e oferecer como alternativa o pop-up só pelo atalho, sem ícone.

### 7.5 Webview
- WebKitGTK é o ponto mais frágil (consumo e travamentos). Seguir os workarounds do Handy (DMA-BUF desligado por padrão), manter a UI mínima e destruir o webview quando ele fechar.

### 7.6 Empacotamento

| Formato | Prós | Contras | Decisão |
|---|---|---|---|
| **AUR** (`lira-bin` e `lira-git`) | Natural para o público Arch/EndeavourOS; sem sandbox, então socket, `tesseract` e os portais funcionam sem atrito | Só Arch | **Primeiro canal** |
| **AppImage** | Um arquivo, qualquer distro, o Tauri gera nativamente | Sem atualização automática do sistema; precisa de FUSE; leva o WebKitGTK junto ou depende da versão do sistema | Segundo canal, para outras distros |
| **.deb / .rpm** | O Tauri gera; Ubuntu e Fedora | Mais manutenção | Junto com o AppImage |
| **Flatpak** | Flathub dá visibilidade; o sandbox obriga a usar portais, que nós já usamos | O sandbox atrapalha a chamada ao `claude` do host (precisa de `flatpak-spawn --host`, o que reduz a segurança e é visto com desconfiança no Flathub), o acesso ao `~/.claude` e o Tesseract do sistema | **Depois do M2**, só se houver demanda |

### 7.7 Windows (M4) e macOS (por último)
- **Windows:** plugin oficial de atalho, Windows.Graphics.Capture (`windows-capture`), `Windows.Media.Ocr`, WebView2 e instalador MSI/NSIS do Tauri. O Claude Code roda nativo ou via WSL, e o adaptador precisa tratar os dois casos.
- **macOS:** permissões de Acessibilidade, Microfone e Gravação de Tela; ScreenCaptureKit; Vision para OCR; Metal no whisper.cpp; notarização (US$99/ano).

---

## 8. Plano de marcos da fase 1

Estimativas para desenvolvimento por agentes. Cada marco termina numa release taggeada e num PR em draft revisado.

| Marco | Entrega | Critério de pronto |
|---|---|---|
| **M0. Scaffold + CI no Linux** | Workspace Cargo, app Tauri 2 com ícone na bandeja, socket + `lira toggle`, base de i18n (pt-BR/en), `config.toml` com o esquema da 5.2, CI no GitHub Actions (Ubuntu + container Arch), `cargo clippy`/`fmt`/test, verificação do DCO, PKGBUILD do `lira-git` | **No Niri + DMS:** instala pelo AUR local; `lira toggle` via bind abre e fecha o pop-up layer-shell; funciona sem bandeja; ícone e menu do Tauri aparecem na bandeja do DMS; **interface e CLI em pt-BR e en** (troca por `ui.locale`, CI verifica chaves de i18n). RSS ocioso medido no CI. Smoke test manual no KDE, no GNOME e no Hyprland antes de fechar o marco. |
| **M1. Push-to-talk + transcrição local** | Backend de atalho por compositor (bind Niri → CLI; portal no KDE/GNOME/Hyprland; X11); configuração `hotkey.mode` (`toggle` padrão; `hold` via portal/`bindr`; `lira-hold` evdev opt-in com aviso); VAD auto-stop; `cpal`; whisper.cpp `large-v3-turbo-q5_0` (padrão) e `small` com download do modelo; `stt.language` pt/en/auto + vocabulário; pop-up com o texto | **No Niri + DMS primeiro:** apertar, falar e apertar de novo (ou parar no silêncio) dá texto; benchmark com 30 frases pt-BR + 30 en (WER, latência < 2–3 s para 10 s de áudio, RSS de pico) define o modelo padrão final; 0% de CPU parado; `lira-hold` testado com o aviso. Depois, segurar para falar validado no KDE/GNOME/Hyprland via portal. **Fim do M1 = o repositório fica público.** |
| **M2. Adaptador Claude Code** | Trait `AgentAdapter` + adaptador `claude-code` (headless/SDK, retoma a sessão); registro de sessões; roteador por regras ("status", "resume", "<sessão>, …"); notificação quando o agente terminar; política de aprovações | Mandar uma instrução e receber a resposta de uma sessão real sem trocar de janela; agente fake passando nos testes da trait |
| **M3. Contexto de tela opt-in** | Niri: IPC `focused-window` + `screenshot-window`; portal Screenshot nos outros; `xcap` no X11; gerador de `block-out-from`; Tesseract; redação (CPF/CNPJ/cartão/tokens); lista de exclusão; indicador visual + prévia | Desligado por padrão; nada gravado em disco; teste de redação com fixtures |
| **M4. Windows** | Atalho oficial, Windows.Graphics.Capture, `Windows.Media.Ocr`, instalador | Mesmos critérios do M1 ao M3 no Windows 11 |
| **Depois** | macOS; fase 2 (TTS, aprovação por voz, imagem para modelos de visão); adaptadores Codex e ACP | — |

---

## 9. Metas de CPU e RAM (e como cumprir)

| Estado | CPU | RAM (meta) |
|---|---|---|
| Ocioso na bandeja | ~0% (sem polling, só esperando eventos) | ≤ 40 MB (sem webview vivo) |
| Gravando | < 5% de um núcleo | + buffer de áudio (~2 MB/min a 16 kHz mono 16-bit) |
| Transcrevendo (whisper `base`, 10 s de áudio) | pico de 1 a 2 s | + 150–300 MB **só durante** (modelo carregado sob demanda) |
| Pop-up aberto | < 3% | ≤ 120 MB total |

Essas metas são estimativas para validar com benchmark na semana 1, não números medidos.

Como cumprir:
1. **Webview sob demanda:** criado no atalho e destruído alguns segundos depois de fechar. Ícone e menu da bandeja são nativos.
2. **Modelo STT com ciclo de vida:** carrega no primeiro uso e descarrega depois de N minutos parado (configurável: "manter quente" para quem quer latência mínima).
3. **Microfone fechado fora do push-to-talk.** Sem palavra de ativação e sem VAD contínuo.
4. **Eventos dos agentes agrupados** no núcleo (≤ 30/s para a UI e zero quando a UI está fechada). O histórico vai para o SQLite.
5. **Adaptadores sem polling:** usam stream, stdio ou eventos. Se precisar de polling, no máximo a cada 5 s e só quando o pop-up estiver aberto.
6. Build `release` com LTO e `opt-level="s"` para o binário, e modelos baixados à parte (não vêm no instalador).
7. Benchmark de CI: medir RSS ocioso e o tempo de STT em cada release, e falhar o build se regredir mais de 20%.

---

## 10. Segurança e aprovações

- Aprovação por voz só para ações `reversible` (ex.: "pode rodar os testes", "pode criar a branch"). `destructive` (migrations, `rm`, push, deploy, enviar mensagem) exige clique no pop-up.
- Confirmação falada no formato "sim, aprova X". Um "sim" solto não aprova (aprendizado do backtalk: "an exact spoken yes").
- Chaves de API no keychain do sistema (`keyring` crate), nunca em texto puro.
- O servidor MCP local só escuta em socket ou localhost, com token por sessão.

---

## 11. Licença e modelo de negócio. **DECIDIDO: open core (opção b)**

**Núcleo** `fernandogbs/lira`: **MIT OR Apache-2.0, DCO (`Signed-off-by`, checado no CI), sem CLA.** Fica privado até o M1 e é público depois.
**Add-ons** `fernandogbs/lira-pro`: repositório **fechado**, vendido por **licença de site/assento**. O que entra:
- voz em tempo real;
- STT na nuvem gerenciado;
- adaptadores extras;
- sync entre máquinas;
- funções de time.

Eles se conectam pela fronteira da seção 5.1.

- A MIT/Apache permite distribuir os add-ons fechados junto com o núcleo. O Fernando é dono do copyright do código dele.
- Sem CLA, ele **não** pode relicenciar o código dos contribuidores, mas não precisa, porque a licença permissiva já permite uso comercial. A única coisa que fica fechada para sempre é a opção de transformar o núcleo em proprietário.
- O núcleo precisa ser útil sozinho (transcrição local + Claude Code + tela via OCR). Se o grátis for capado, a comunidade não vem, e sem comunidade o pago não vende.

**Alternativas rejeitadas:**
- (a) Totalmente aberto, sem pago: não tem caminho de receita, e ele quer poder monetizar.
- (c) FSL-1.1-Apache-2.0 ([fsl.software](https://fsl.software/), vira Apache/MIT depois de 2 anos): não é OSI, e afasta contribuidores e distros (AUR/Flathub aceitam, mas a comunidade torce o nariz) num projeto que depende de tração.
- (d) Fechado freemium: compete de frente com o `/voice` gratuito do Claude Code, o Wispr Flow e o Superwhisper, sem a vantagem de confiança e privacidade que o código aberto dá.

**Pré-validação do pago (antes de construir o `lira-pro`):** uma landing page com demo em vídeo e lista de espera com preços visíveis (ex.: Pro US$5–8/mês ou US$60/ano; Team por assento), mais um botão de "reservar" (pré-venda ou intenção). Critério sugerido: pelo menos 100 inscritos e 5% com intenção de pagar depois de 30 dias do repositório público. Se não bater, o projeto continua só como OSS e portfólio.

**Também:**
- **Não copiar código** do backtalk e do openinterpreter/01 (AGPL-3.0), do VoiceInk e do whisper-writer (GPL-3.0) e do screenpipe (licença comercial). O Niri é GPL-3.0, mas falar com ele por IPC, socket ou CLI não contamina a Lira. Mas a crate `niri-ipc` é **GPL-3.0-or-later** (herdada do workspace, verificado no `Cargo.toml`), então **não pode ser dependência do núcleo MIT/Apache**. Usar `niri msg --json` ou escrever tipos serde próprios para o JSON do socket `$NIRI_SOCKET`, sem copiar o código.
- **Atenção aos modelos:** cada peso tem a sua licença (Whisper MIT, Parakeet CC-BY-4.0 **[não verificado na versão V3]**, vozes do Piper variam). Baixar sob demanda e mostrar a licença.

---

## 12. Estrutura do repositório

```
lira/                          # github.com/fernandogbs/lira (privado até o M1, depois público)
├── Cargo.toml                 # workspace
├── crates/
│   ├── core/                  # orquestração, roteador, política, sessão
│   ├── audio/                 # cpal, VAD, buffer
│   ├── stt/                   # trait + whisper / sherpa / os-native / cloud
│   ├── tts/                   # fase 2
│   ├── screen/                # captura, OCR, redação, exclusões
│   ├── adapters/
│   │   ├── api/               # trait AgentAdapter + agente fake de testes
│   │   ├── claude-code/       # fase 1
│   │   ├── codex/             # depois (pasta reservada)
│   │   └── acp/               # depois
│   ├── plugin-protocol/       # JSON-RPC v1 + host de plugins (fronteira com o lira-pro)
│   ├── mcp-server/            # notify_user, ask_user, request_approval
│   ├── hotkey/                # bind Niri/CLI, portal GlobalShortcuts, X11, socket
│   ├── compositor/            # niri (IPC), hyprland, kde, gnome, x11: janela ativa e captura
│   └── store/                 # SQLite (rusqlite), migrações
├── app/                       # Tauri 2 (src-tauri + ui/ em Svelte ou Solid)
├── cli/                       # lira toggle | start | stop | status
├── packaging/                 # aur/ (PKGBUILD lira-bin, lira-git), appimage/, flatpak/ (futuro), niri/ (trechos KDL)
├── i18n/                      # pt-BR/ e en/ (.ftl), usados pelo núcleo, CLI e UI
├── docs/                      # ARCHITECTURE.md, PRIVACY.md, ADAPTERS.md (+ README.pt-BR.md na raiz)
├── bench/                     # benchmarks de RAM, CPU e latência
├── CONTRIBUTING.md            # DCO, como escrever um adaptador
└── .github/workflows/         # CI Linux primeiro (Ubuntu + Arch), Windows no M4
```

Cada adaptador é uma crate com testes contra um "agente fake", para que contribuidores consigam adicionar agentes sem conhecer o núcleo.

---

## 13. Riscos

| Risco | Prob. | Impacto | Mitigação |
|---|---|---|---|
| A Anthropic ou a OpenAI lançam voz fora do terminal / no desktop | Alta | Alto | Ser multiagente, local e com contexto de sessão. Não competir em ditado. |
| CLIs dos agentes mudam o formato de saída e quebram os adaptadores | Alta | Médio | Preferir ACP e SDKs oficiais; testes de contrato; PTY só como último recurso. |
| Fragmentação no Wayland (atalho, captura, bandeja) | Certa | **Alto** (Linux é o alvo principal) | Backends por compositor (7.0.1); `lira toggle` como reserva; smoke test em Niri, KDE, GNOME e Hyprland a cada marco. |
| Niri sem push-to-talk real e sem portal de atalho | Certa (hoje) | Médio | Toggle + VAD; `lira-hold` opt-in; acompanhar os PRs #3587 e #3621. |
| Transcrição em pt-BR ruim ou lenta no CPU | Média | Alto | Benchmark próprio no M1; turbo-q5 vs small; prompt de vocabulário; idioma fixo por padrão. |
| Mudanças no DMS (renomeado para Dux Shell, v2.0) quebrarem a bandeja ou o plugin | Média | Baixo | Lira funciona sem bandeja; plugin DMS separado e opcional. |
| Add-ons pagos esvaziarem o núcleo e afastarem a comunidade | Média | Alto | Regra da 5.1 (o pago só acrescenta); pré-validação antes de construir o pago. |
| Nome confundido com o amd/LIRA | Baixa | Baixo | Descrição clara no README e no AUR; checar `lira` no crates.io antes de publicar. |
| WebKitGTK instável no Linux | Média | Médio | UI mínima; seguir os workarounds do Handy. |
| Vazamento de dados pela captura de tela | Média | Alto | Desligado por padrão, OCR + redação, exclusões, prévia, nada no disco. |
| Projeto OSS sem tração (mais um wrapper de voz) | Média | Alto | Demo forte de 30 s: "falando com 3 agentes ao mesmo tempo sem trocar de janela". Lançar em HN, r/ClaudeAI e no Discord do Handy. |
| Assinatura de código (certificado Windows; notarização Apple US$99/ano) | Certa | Baixo | Linux não precisa; Windows sem assinatura no início; custo da Apple só quando o macOS entrar. |

---

## 14. Perguntas em aberto para o Fernando

1. **Contexto de tela no v1:** só OCR local (recomendado), ou já com imagem para modelos de visão?
2. **Relação com o orquestrador (Kanban/Map):** o Lira vira depois o "módulo de comando rápido" dele, ou fica independente para sempre?
3. **Preços da landing de pré-validação:** os valores sugeridos (US$5–8/mês) fazem sentido, e quem monta a landing (Hermes/Suze)?
4. **Sua máquina para o benchmark do M1:** CPU e RAM (e se tem GPU), para decidir entre turbo-q5 e small como padrão.
5. **Plugin para o DMS (`lira-dms`):** vale fazer depois do M2, ou fica para a comunidade?
