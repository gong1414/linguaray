/**
 * Popup controller — faithful port of the Solid createPopupController.
 *
 * Preserved invariants:
 *  - event listeners registered BEFORE provider_list loads (events during the
 *    load are captured, not lost)
 *  - unmount-during-await race guard (cancelled flag; late listeners dropped)
 *  - R7-P1-2 state generation: an in-flight Retry's error never overwrites a
 *    newer popup-state/popup-multi-result event
 *  - P1-3 lastSource: the ORIGINAL selection (from backend source_text), so
 *    Retry re-translates the source — never the clipboard, never the result
 *  - blur-hide gated by pin (a pinned popup stays visible)
 */
import { useCallback, useEffect, useRef, useState } from "react";
import { invoke } from "../../../bridge/invoke";
import { listen } from "../../../bridge/event";
import { getCurrentWindow } from "../../../bridge/window";
import { decodePopupMultiResult, decodePopupState } from "./decode";
import { translateSelection } from "./popup-ipc";
import type { PopupMultiPayload, PopupStatePayload, TranslationState } from "./types";

const PRESET_LABELS: Record<string, string> = {
  openai: "OpenAI",
  anthropic: "Anthropic",
  gemini: "Gemini",
  ollama: "Ollama",
};

export function usePopupController() {
  const [state, setState] = useState<TranslationState>({ kind: "loading" });
  const [pinned, setPinned] = useState(false);
  const [lastSource, setLastSource] = useState("");
  const [nameMap, setNameMap] = useState<ReadonlyMap<string, string>>(new Map());

  const ref = useRef({ pinned, lastSource, generation: 0, cancelled: false });
  ref.current.pinned = pinned;
  ref.current.lastSource = lastSource;

  useEffect(() => {
    ref.current.cancelled = false;
    const unlisteners: Array<() => void> = [];
    let done = false;
    const keep = (u: () => void) => {
      if (done || ref.current.cancelled) u();
      else unlisteners.push(u);
    };

    void listen<PopupStatePayload>("popup-state", (e) => {
      if (ref.current.cancelled) return;
      const payload = e.payload;
      // Bump the generation — any in-flight Retry is now stale.
      ref.current.generation++;
      if (payload.status === "loading") {
        // A new session: clear the prior source, then adopt this one.
        setLastSource(payload.source_text ?? "");
      } else if (payload.source_text) {
        setLastSource(payload.source_text);
      }
      setState(decodePopupState(payload));
    }).then(keep);

    void listen<PopupMultiPayload>("popup-multi-result", (e) => {
      if (ref.current.cancelled) return;
      ref.current.generation++;
      if (e.payload.source_text) setLastSource(e.payload.source_text);
      setState(decodePopupMultiResult(e.payload));
    }).then(keep);

    // Blur-hide, gated by pin. Registered synchronously-safe via ref reads.
    const win = getCurrentWindow();
    void win
      .onFocusChanged(({ payload: focused }) => {
        if (ref.current.cancelled) return;
        if (!focused && !ref.current.pinned) void win.hide();
      })
      .then(keep)
      .catch(() => {});

    // Provider name map LAST (best-effort; labels fall back below).
    void invoke<{ uuid: string; name: string }[]>("provider_list")
      .then((profiles) => {
        if (ref.current.cancelled) return;
        setNameMap(new Map(profiles.map((p) => [p.uuid, p.name])));
      })
      .catch(() => {});

    return () => {
      done = true;
      ref.current.cancelled = true;
      for (const u of unlisteners) u();
    };
  }, []);

  const engineLabel = useCallback(
    (raw: string): string => {
      if (nameMap.has(raw)) return nameMap.get(raw)!;
      if (raw.startsWith("provider/")) {
        const uuid = raw.slice("provider/".length);
        if (nameMap.has(uuid)) return nameMap.get(uuid)!;
      }
      if (PRESET_LABELS[raw]) return PRESET_LABELS[raw];
      if (["google", "deepl", "microsoft", "baidu", "youdao", "tencent"].includes(raw)) {
        return "Fallback";
      }
      return "Unknown";
    },
    [nameMap],
  );

  const dismiss = useCallback(async () => {
    setPinned(false);
    await getCurrentWindow().hide();
  }, []);

  const retrySelection = useCallback(async () => {
    if (!ref.current.lastSource) return;
    const myGen = ++ref.current.generation;
    setState({ kind: "loading" });
    try {
      await translateSelection(ref.current.lastSource);
      // The backend re-emits popup-state / popup-multi-result, which bumps the
      // generation and sets state. Nothing to do on silent success.
    } catch (e) {
      // Apply the error only if no newer event landed during the await.
      if (myGen === ref.current.generation && !ref.current.cancelled) {
        setState({ kind: "error", sub: "generic", message: String(e) });
      }
    }
  }, []);

  return {
    state,
    pinned,
    lastSource,
    hasSource: lastSource.length > 0,
    nameMap,
    engineLabel,
    pin: () => setPinned(true),
    unpin: () => setPinned(false),
    dismiss: () => void dismiss(),
    retrySelection: () => void retrySelection(),
  };
}

export type PopupController = ReturnType<typeof usePopupController>;
