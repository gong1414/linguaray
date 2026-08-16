/** Popup-window typed commands (selection retry, tts, settings, clipboard). */
import { commands } from "../../bridge/invoke";

/** P1-3: re-translate the ORIGINAL source text — never the clipboard/result. */
export const translateSelection = (sourceText?: string): Promise<void> =>
  commands.translateSelectionIpc(sourceText ?? null).then(() => undefined);

export const ttsSpeak = (text: string): Promise<void> =>
  commands.ttsSpeak(text, null).then(() => undefined);

export const ttsStop = (): Promise<void> => commands.ttsStop();

export const openSettingsWindow = (section?: string): Promise<void> =>
  commands.openSettingsWindow(section ?? null).then(() => undefined);
