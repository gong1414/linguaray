import { createSignal, onCleanup, onMount } from "solid-js";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { decodePopupMultiResult, decodePopupState } from "./decode";
import type { PopupMultiPayload, PopupStatePayload, TranslationState } from "./types";

/**
 * Production popup controller. Owns:
 *  - the `state` signal (the single TranslationState the UI renders)
 *  - the `pinned` signal (Surface 01: pinned popups ignore blur-hide)
 *  - Tauri event subscriptions (popup-state + popup-multi-result)
 *  - blur-hide gating on pin state
 *  - retry (re-emits the last selection translation via translate_clipboard —
 *    delegated to the backend which re-reads the active selection and emits
 *    popup-state / popup-multi-result, which re-decode here)
 *
 * Returns a plain object of accessors/actions; the component binds them.
 */
export function createPopupController() {
  const [state, setState] = createSignal<TranslationState>({ kind: "loading" });
  const [pinned, setPinned] = createSignal(false);
  const unlisteners: UnlistenFn[] = [];

  onMount(async () => {
    unlisteners.push(
      await listen<PopupStatePayload>("popup-state", (e) => {
        setState(decodePopupState(e.payload));
      }),
    );
    unlisteners.push(
      await listen<PopupMultiPayload>("popup-multi-result", (e) => {
        setState(decodePopupMultiResult(e.payload));
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

  const pin = () => setPinned(true);
  const unpin = () => setPinned(false);

  const dismiss = async () => {
    setPinned(false);
    await getCurrentWindow().hide();
  };

  // Retry: ask the backend to re-run the current-selection translation. The
  // backend re-emits popup-state / popup-multi-result, which re-decode here.
  const retry = async () => {
    setState({ kind: "loading" });
    try {
      await invoke("translate_clipboard");
    } catch (e) {
      setState({ kind: "error", sub: "generic", message: String(e) });
    }
  };

  return { state, pinned, pin, unpin, dismiss, retry };
}
