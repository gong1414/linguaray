/** Input-window typed commands. */
import { commands } from "../../bridge/invoke";
import type { SessionResultFE } from "./types";

export const translateSession = async (text: string): Promise<SessionResultFE> => {
  const result = await commands.translateSession({ text, from: "auto", to: "" });
  return {
    outcomes: result.outcomes.map((outcome) => ({
      uuid: outcome.uuid,
      ok: outcome.ok,
      ...(outcome.text != null ? { text: outcome.text } : {}),
      ...(outcome.engine != null ? { engine: outcome.engine } : {}),
      ...(outcome.error != null ? { error: outcome.error } : {}),
    })),
    ...(result.actual_engine != null ? { actual_engine: result.actual_engine } : {}),
  };
};

export const addVocabulary = (word: string, definition: string, targetLanguage: string): Promise<void> =>
  commands.vocabularyAdd(word, definition, "auto", targetLanguage).then(() => undefined);
