import { invoke } from "@tauri-apps/api/core";

/**
 * Translate the live OS selection (fresh capture) OR a caller-supplied SOURCE
 * text (Retry). Calls the backend `translate_selection_ipc` command which NEVER
 * reads the clipboard. Distinct from `translateClipboard` which reads the
 * clipboard. Used by the tray `translate-selection` action and the popup Retry.
 *
 * P1-3: `sourceText` is the ORIGINAL selected text, not a translation result.
 */
export const translateSelection = (sourceText?: string): Promise<void> =>
  invoke<void>("translate_selection_ipc", sourceText !== undefined ? { text: sourceText } : {});

/** Translate the clipboard contents. Distinct from selection translation. */
export const translateClipboard = (): Promise<void> =>
  invoke<void>("translate_clipboard");
