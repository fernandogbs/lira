<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import en from "./locales/en.json";
  import ptBr from "./locales/pt-BR.json";

  type HotkeyMode = "toggle" | "hold";
  type AppState = "idle" | "listening";

  interface PopupStatus {
    state: AppState;
    hotkey_mode: HotkeyMode;
    state_label: string;
    hotkey_mode_label: string;
  }

  const locale =
    (navigator.language || "en").toLowerCase().startsWith("pt") ? ptBr : en;

  let status: PopupStatus = {
    state: "idle",
    hotkey_mode: "toggle",
    state_label: "…",
    hotkey_mode_label: "…",
  };

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
</main>
