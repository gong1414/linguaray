import { createSignal, onCleanup, onMount } from "solid-js";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { decodePopupMultiResult, decodePopupState } from "./decode";
import { translateSelection } from "./selection-ipc";
import type { PopupMultiPayload, PopupStatePayload, TranslationState } from "./types";

/**
 * Production popup controller. Owns:
 *  - the `state` signal (the single TranslationState the UI renders)
 *  - the `pinned` signal (Surface 01: pinned popups ignore blur-hide)
 *  - Tauri event subscriptions (popup-state + popup-multi-result)
 *  - blur-hide gating on pin state
 *  - lastSource: the ORIGINAL selected text (P1-3), saved from the backend
 *    payload's `source_text` on every loading/result/error/multi event so Retry
 *    can re-translate the SAME source via translate_selection_ipc (never
 *    translate_clipboard, never the translation result).
 *
 * Returns a plain object of accessors/actions; the component binds them.
 */
export function createPopupController() {
  const [state, setState] = createSignal<TranslationState>({ kind: "loading" });
  const [pinned, setPinned] = createSignal(false);
  /** uuid → friendly provider name. Loaded once on mount from provider_list. */
  const nameMap = new Map<string, string>();
  /** P1-3: last saved SOURCE text (original selection), for Retry. */
  let lastSource = "";
  const unlisteners: UnlistenFn[] = [];

  onMount(async () => {
    // B3: load the provider name map so engine labels resolve to friendly names.
    try {
      const profiles = await invoke<{ uuid: string; name: string }[]>("provider_list");
      for (const p of profiles) nameMap.set(p.uuid, p.name);
    } catch {
      // Best-effort: leave the map empty; labels fall back below.
    }
    unlisteners.push(
      await listen<PopupStatePayload>("popup-state", (e) => {
        const payload = e.payload;
        // P1-3: loading opens a new translation session — clear the prior
        // source, then adopt this session's source if the backend carried it.
        if (payload.status === "loading") {
          lastSource = payload.source_text ?? "";
        } else if (payload.source_text) {
          lastSource = payload.source_text;
        }
        setState(decodePopupState(payload));
      }),
    );
    unlisteners.push(
      await listen<PopupMultiPayload>("popup-multi-result", (e) => {
        const payload = e.payload;
        if (payload.source_text) lastSource = payload.source_text;
        setState(decodePopupMultiResult(payload));
      }),
    );

    // Blur-hide, gated by pin: a pinned popup stays visible on blur (S0 §4.1).
    const win = getCurrentWindow();
    unlisteners.push(
      await win.onFocusChanged(({ payload: focused }) => {
        if (!focused && !pinned()) win.hide();
      }),
    );
  });

  onCleanup(() => {
    for (const u of unlisteners) u();
  });

  /** B3: resolve a raw engine id (uuid or `provider/<uuid>`) to a friendly name. */
  const engineLabel = (raw: string): string => {
    if (nameMap.has(raw)) return nameMap.get(raw)!;
    if (raw.startsWith("provider/")) {
      const uuid = raw.slice("provider/".length);
      if (nameMap.has(uuid)) return nameMap.get(uuid)!;
    }
    const presetLabels: Record<string, string> = {
      openai: "OpenAI",
      anthropic: "Anthropic",
      gemini: "Gemini",
      ollama: "Ollama",
    };
    if (presetLabels[raw]) return presetLabels[raw];
    if (["google", "deepl", "microsoft", "baidu", "youdao", "tencent"].includes(raw)) {
      return "Fallback";
    }
    return "Unknown";
  };

  const pin = () => setPinned(true);
  const unpin = () => setPinned(false);

  const dismiss = async () => {
    setPinned(false);
    await getCurrentWindow().hide();
  };

  /**
   * P1-3: Retry re-translates the SAVED SOURCE text via translate_selection_ipc
   * (translateSelection). Never translate_clipboard, never the translation
   * result. No-op when there is no saved source. The backend re-emits
   * popup-state / popup-multi-result, which re-decode here.
   */
  const retrySelection = async () => {
    if (!lastSource) return;
    setState({ kind: "loading" });
    try {
      await translateSelection(lastSource);
    } catch (e) {
      setState({ kind: "error", sub: "generic", message: String(e) });
    }
  };

  /** Whether a saved SOURCE text is available for Retry (P1-3). */
  const hasSource = () => lastSource.length > 0;

  return { state, pinned, pin, unpin, dismiss, retrySelection, hasSource, engineLabel };
}
