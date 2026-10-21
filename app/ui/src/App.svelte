<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import en from "./locales/en.json";
  import ptBr from "./locales/pt-BR.json";

  type HotkeyMode = "toggle" | "hold";
  type AppState = "idle" | "listening" | "transcribing" | "waiting_agent";
  type ScreenContext = "off" | "text" | "image";

  interface PopupStatus {
    state: AppState;
    hotkey_mode: HotkeyMode;
    screen_context: ScreenContext;
    locale_tag: string;
    state_label: string;
    hotkey_mode_label: string;
    screen_context_label: string;
    screen_context_note: string | null;
  }

  interface TranscriptPayload {
    text: string;
    latency_ms: number;
    model: string;
  }

  interface ErrorPayload {
    message: string;
  }

  type LocaleStrings = typeof en;

  let status: PopupStatus = {
    state: "idle",
    hotkey_mode: "toggle",
    screen_context: "off",
    locale_tag: "en",
    state_label: "…",
    hotkey_mode_label: "…",
    screen_context_label: "…",
    screen_context_note: null,
  };

  let transcript: TranscriptPayload | null = null;
  let agentReply: string | null = null;
  let errorMessage: string | null = null;
  let copiedNotice = false;

  function localeStrings(tag: string): LocaleStrings {
    return tag === "pt-BR" ? ptBr : en;
  }

  $: locale = localeStrings(status.locale_tag);

  async function refresh() {
    status = await invoke<PopupStatus>("get_popup_status");
  }

  onMount(() => {
    refresh();
    const unsubs: Array<() => void> = [];
    listen<PopupStatus>("lira://status", (event) => {
      status = event.payload;
      if (status.state === "listening" || status.state === "transcribing") {
        transcript = null;
        agentReply = null;
        errorMessage = null;
        copiedNotice = false;
      }
    }).then((u) => unsubs.push(u));
    listen<TranscriptPayload>("lira://transcript", (event) => {
      transcript = event.payload;
      errorMessage = null;
      copiedNotice = true;
    }).then((u) => unsubs.push(u));
    listen<{ session: string; text: string }>("lira://agent-reply", (event) => {
      agentReply = event.payload.text;
      errorMessage = null;
    }).then((u) => unsubs.push(u));
    listen<ErrorPayload>("lira://error", (event) => {
      errorMessage = event.payload.message;
      transcript = null;
      agentReply = null;
      copiedNotice = false;
    }).then((u) => unsubs.push(u));
    return () => {
      for (const u of unsubs) u();
    };
  });
</script>

<main>
  <h1>{locale.title}</h1>
  <p><strong>{locale.state}:</strong> {status.state_label}</p>
  <p class="muted">
    <strong>{locale.hotkeyMode}:</strong> {status.hotkey_mode_label}
  </p>
  <p class="muted">
    <strong>{locale.screenContext}:</strong> {status.screen_context_label}
  </p>
  {#if status.screen_context_note}
    <p class="note">{status.screen_context_note}</p>
  {/if}
  {#if transcript}
    <section class="result">
      <p class="transcript" data-selectable>{transcript.text}</p>
      {#if copiedNotice}
        <p class="muted">{locale.copiedToClipboard}</p>
      {/if}
      <p class="muted">
        {locale.latencyLabel.replace("{ms}", String(transcript.latency_ms))}
      </p>
    </section>
  {/if}
  {#if agentReply}
    <section class="result">
      <p class="muted">{locale.agentReply}</p>
      <p class="transcript" data-selectable>{agentReply}</p>
    </section>
  {/if}
  {#if errorMessage}
    <p class="error">{errorMessage}</p>
  {/if}
</main>

<style>
  .transcript {
    user-select: text;
    white-space: pre-wrap;
  }
  .error {
    color: #c62828;
  }
</style>
