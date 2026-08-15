/** Typed wrappers for the vocabulary Rust commands. */
import { invoke } from "../../../bridge/invoke";
import { save } from "../../../bridge/dialog";
import type { VocabularyItem } from "./copy";

export const vocabularyList = (): Promise<VocabularyItem[]> =>
  invoke<{ items: VocabularyItem[] }>("vocabulary_list", { cursor: null }).then(
    (page) => page.items,
  );

export const vocabularyAdd = (
  word: string,
  definition: string,
): Promise<void> =>
  invoke<void>("vocabulary_add", {
    word,
    definition,
    sourceLanguage: "auto",
    targetLanguage: "zh",
  });

export const vocabularyDelete = (itemUuid: string): Promise<void> =>
  invoke<void>("vocabulary_delete", { itemUuid });

export const vocabularyExportFile = async (
  format: "csv" | "json",
): Promise<string | null> => {
  try {
    const filePath = await save({ defaultPath: `linguaray-vocabulary.${format}` });
    if (!filePath) return null;
    return await invoke<string>("vocabulary_export_file", { filePath, format });
  } catch {
    return null;
  }
};

export const vocabularyExportAnki = (deckName: string): Promise<void> =>
  invoke<void>("vocabulary_export_anki", { deckName });
