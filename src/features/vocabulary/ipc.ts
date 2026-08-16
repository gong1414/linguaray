/** Typed wrappers for the vocabulary Rust commands. */
import { commands } from "../../bridge/invoke";
import { save } from "../../bridge/dialog";
import type { VocabularyItem } from "./copy";

export const vocabularyList = (): Promise<VocabularyItem[]> =>
  commands.vocabularyList(null).then(
    (page) => page.items,
  );

export const vocabularyAdd = (
  word: string,
  definition: string,
): Promise<void> =>
  commands.vocabularyAdd(word, definition, "auto", "zh").then(() => undefined);

export const vocabularyDelete = (itemUuid: string): Promise<void> =>
  commands.vocabularyDelete(itemUuid).then(() => undefined);

export const vocabularyExportFile = async (
  format: "csv" | "json",
): Promise<string | null> => {
  try {
    const filePath = await save({ defaultPath: `linguaray-vocabulary.${format}` });
    if (!filePath) return null;
    return await commands.vocabularyExportFile(filePath, format);
  } catch {
    return null;
  }
};

export const vocabularyExportAnki = (deckName: string): Promise<void> =>
  commands.vocabularyExportAnki(deckName).then(() => undefined);
