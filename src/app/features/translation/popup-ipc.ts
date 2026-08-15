/** Popup-window typed commands (selection retry, tts, settings, clipboard). */
import { invoke } from "../../../bridge/invoke";

/** P1-3: re-translate the ORIGINAL source text — never the clipboard/result. */
export const translateSelection = (sourceText?: string): Promise<void> =>
  invoke<void>("translate_selection_ipc", sourceText !== undefined ? { text: sourceText } : {});

export const ttsSpeak = (text: string): Promise<void> =>
  invoke<void>("tts_speak", { text, voiceId: null });

export const ttsStop = (): Promise<void> => invoke<void>("tts_stop");

export const openSettingsWindow = (section?: string): Promise<void> =>
  invoke<void>("open_settings_window", section ? { section } : {});
