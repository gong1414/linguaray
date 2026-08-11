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
  /**
   * P1-9/R2-D: nameMap version counter. provider_list may resolve AFTER a
   * popup-state result has already rendered an engine label. A plain Map gives
   * Solid no way to know it changed, so the already-rendered label sticks at
   * "Unknown". Bumping this signal after populating the map re-runs any
   * computation that read it (engineLabel), letting the card update to the
   * real provider name without a new backend event.
   */
  const [nameMapVersion, setNameMapVersion] = createSignal(0);
  /**
   * P1-3: last saved SOURCE text (original selection), for Retry. A signal (not
   * a plain `let`) so `hasSource()` is reactive — the loading shell mounts once
   * on popup open (initial loading state), so a later loading event that carries
   * source_text would never re-render a plain-let-derived Retry (P1-8).
   */
  const [lastSource, setLastSource] = createSignal("");
  const unlisteners: UnlistenFn[] = [];
  /**
   * B2/P1-9: guards the unmount-during-await race. onCleanup flips this to
   * `true` and runs the already-pushed unlisteners BEFORE any pending await in
   * onMount resolves. Listener callbacks check it (no setState after unmount),
   * and each `await listen(...)` result is dropped immediately if cleanup
   * already ran (no leaked listener registered after unmount).
   */
  let cancelled = false;

  onMount(async () => {
    // B2/P1-9: register event listeners BEFORE loading provider_list. Events
    // arriving during the provider_list load must still be captured (the prior
    // order awaited provider_list first and lost any event fired during load).
    const unState = await listen<PopupStatePayload>("popup-state", (e) => {
      if (cancelled) return;
      const payload = e.payload;
      // P1-3: loading opens a new translation session — clear the prior
      // source, then adopt this session's source if the backend carried it.
      if (payload.status === "loading") {
        setLastSource(payload.source_text ?? "");
      } else if (payload.source_text) {
        setLastSource(payload.source_text);
      }
      setState(decodePopupState(payload));
    });
    // Component unmounted while this await was pending: drop the listener
    // immediately so it is never leaked (onCleanup already ran).
    if (cancelled) { unState(); return; }
    unlisteners.push(unState);

    const unMulti = await listen<PopupMultiPayload>("popup-multi-result", (e) => {
      if (cancelled) return;
      const payload = e.payload;
      if (payload.source_text) setLastSource(payload.source_text);
      setState(decodePopupMultiResult(payload));
    });
    if (cancelled) { unMulti(); return; }
    unlisteners.push(unMulti);

    // Blur-hide, gated by pin: a pinned popup stays visible on blur (S0 §4.1).
    const win = getCurrentWindow();
    const unFocus = await win.onFocusChanged(({ payload: focused }) => {
      if (cancelled) return;
      if (!focused && !pinned()) win.hide();
    });
    if (cancelled) { unFocus(); return; }
    unlisteners.push(unFocus);

    // Load the provider name map LAST (best-effort: labels fall back below).
    try {
      const profiles = await invoke<{ uuid: string; name: string }[]>("provider_list");
      if (cancelled) return;
      for (const p of profiles) nameMap.set(p.uuid, p.name);
      // R2-D: bump the version so any already-rendered engine label (computed
      // before the map was populated) re-runs and resolves the real name.
      setNameMapVersion((v) => v + 1);
    } catch {
      // Best-effort: leave the map empty; labels fall back below.
    }
  });

  onCleanup(() => {
    // B2/P1-9: flip the flag first so callbacks pending on an in-flight event
    // become no-ops, then release the already-registered listeners.
    cancelled = true;
    for (const u of unlisteners) u();
  });

  /** B3: resolve a raw engine id (uuid or `provider/<uuid>`) to a friendly name.
   *  R2-D: reads nameMapVersion() first to establish a reactive dependency, so
   *  a label rendered before provider_list resolves re-runs once the map is
   *  populated. */
  const engineLabel = (raw: string): string => {
    nameMapVersion(); // establish reactive dependency on the name map
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
    if (!lastSource()) return;
    setState({ kind: "loading" });
    try {
      await translateSelection(lastSource());
    } catch (e) {
      setState({ kind: "error", sub: "generic", message: String(e) });
    }
  };

  /** Whether a saved SOURCE text is available for Retry (P1-3). Reactive (P1-8). */
  const hasSource = () => lastSource().length > 0;

  return { state, pinned, pin, unpin, dismiss, retrySelection, hasSource, engineLabel };
}
