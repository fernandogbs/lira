<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import en from "./locales/en.json";
  import ptBr from "./locales/pt-BR.json";

  type HotkeyMode = "toggle" | "hold";
  type AppState = "idle" | "listening";
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

  function localeStrings(tag: string): LocaleStrings {
    return tag === "pt-BR" ? ptBr : en;
  }

  $: locale = localeStrings(status.locale_tag);

  async function refresh() {
    status = await invoke<PopupStatus>("get_popup_status");
  }

  onMount(() => {
    refresh();
    const unlisten = listen<PopupStatus>("lira://status", (event) => {
      status = event.payload;
    });
    return async () => {
      (await unlisten)();
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
</main>
