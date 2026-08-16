/**
 * Input window controller — draft autosave (debounced localStorage), focus
 * restore (cursor at end), translate_session flow, vocabulary favorite.
 */
import { useCallback, useEffect, useRef, useState } from "react";
import { detectLocale } from "../../app/i18n";
import { decodeSessionResult } from "./decode";
import { ensureProviderNameMap } from "./providerNames";
import { addVocabulary, translateSession } from "./input-ipc";
import type { TranslationState } from "./types";

const DRAFT_KEY = "linguaray.input-draft";
const DEBOUNCE_MS = 300;

export function useInputController() {
  const [text, setText] = useState("");
  const [state, setState] = useState<TranslationState>({ kind: "loading" });
  const [idle, setIdle] = useState(true);
  const [hasResult, setHasResult] = useState(false);
  const [favoritedKey, setFavoritedKey] = useState<string | null>(null);

  const textareaRef = useRef<HTMLTextAreaElement | null>(null);
  const timerRef = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const textRef = useRef(text);
  textRef.current = text;

  useEffect(() => {
    // Restore the saved draft + focus the textarea (cursor at end).
    const saved = localStorage.getItem(DRAFT_KEY);
    if (saved) setText(saved);
    const el = textareaRef.current;
    if (el) {
      el.focus();
      const end = saved?.length ?? 0;
      el.setSelectionRange(end, end);
    }
    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, []);

  // Debounced autosave on text change.
  useEffect(() => {
    if (timerRef.current) clearTimeout(timerRef.current);
    timerRef.current = setTimeout(() => {
      if (text) localStorage.setItem(DRAFT_KEY, text);
      else localStorage.removeItem(DRAFT_KEY);
    }, DEBOUNCE_MS);
  }, [text]);

  const translate = useCallback(async () => {
    const value = textRef.current.trim();
    if (!value) return;
    setIdle(false);
    setState({ kind: "loading" });
    try {
      // Ensure the provider name map is loaded BEFORE rendering results so
      // engine labels resolve to friendly names synchronously.
      await ensureProviderNameMap();
      const res = await translateSession(value);
      setState(decodeSessionResult(res));
      setHasResult(true);
    } catch (e) {
      setState({ kind: "error", sub: "generic", message: String(e) });
      setHasResult(true);
    } finally {
      setIdle(true);
    }
  }, []);

  const clear = useCallback(() => {
    setText("");
    setState({ kind: "loading" });
    setHasResult(false);
    setFavoritedKey(null);
    // Purge the persisted draft immediately (not debounced).
    if (timerRef.current) clearTimeout(timerRef.current);
    localStorage.removeItem(DRAFT_KEY);
  }, []);

  const favorite = useCallback(async (source: string, translation: string, key: string) => {
    await addVocabulary(source, translation, detectLocale() === "zh" ? "zh" : "en");
    setFavoritedKey(key);
  }, []);

  return {
    text,
    state,
    idle,
    hasResult,
    favoritedKey,
    setText,
    textareaRef,
    translate,
    clear,
    favorite,
  };
}

export type InputController = ReturnType<typeof useInputController>;
