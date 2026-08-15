/** Input-window typed commands. */
import { invoke } from "../../../bridge/invoke";
import type { SessionResultFE } from "./types";

export const translateSession = (text: string): Promise<SessionResultFE> =>
  invoke<SessionResultFE>("translate_session", { req: { text, from: "auto", to: "" } });

export const addVocabulary = (word: string, definition: string, targetLanguage: string): Promise<void> =>
  invoke<void>("vocabulary_add", {
    word,
    definition,
    sourceLanguage: "auto",
    targetLanguage,
  });
