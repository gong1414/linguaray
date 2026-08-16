/** Vocabulary controller — load/add/remove/export. */
import { useCallback, useEffect, useRef, useState } from "react";
import { detectLocale } from "../../app/i18n";
import { VOCABULARY_COPY, type VocabularyItem } from "./copy";
import * as ipc from "./ipc";

export function useVocabularyController() {
  const t = VOCABULARY_COPY[detectLocale()];
  const [items, setItems] = useState<VocabularyItem[]>([]);
  const [word, setWord] = useState("");
  const [definition, setDefinition] = useState("");
  const [notice, setNotice] = useState("");
  const [busy, setBusy] = useState(false);
  const cancelledRef = useRef(false);

  const reload = useCallback(async () => {
    const page = await ipc.vocabularyList();
    if (!cancelledRef.current) setItems(page);
  }, []);

  useEffect(() => {
    cancelledRef.current = false;
    void reload().catch((e) => !cancelledRef.current && setNotice(String(e)));
    return () => {
      cancelledRef.current = true;
    };
  }, [reload]);

  const add = useCallback(async () => {
    if (!word.trim() || busy) return;
    setBusy(true);
    try {
      await ipc.vocabularyAdd(word.trim(), definition.trim());
      setWord("");
      setDefinition("");
      await reload();
    } finally {
      if (!cancelledRef.current) setBusy(false);
    }
  }, [word, definition, busy, reload]);

  const remove = useCallback(
    async (id: string) => {
      await ipc.vocabularyDelete(id);
      await reload();
    },
    [reload],
  );

  const exportFile = useCallback(
    async (format: "csv" | "json" | "anki") => {
      try {
        if (format === "anki") {
          await ipc.vocabularyExportAnki("LinguaRay");
          setNotice(t.exportDone);
          return;
        }
        const path = await ipc.vocabularyExportFile(format);
        if (path) setNotice(path);
      } catch (e) {
        setNotice(`${t.exportFailed}: ${String(e)}`);
      }
    },
    [t.exportDone, t.exportFailed],
  );

  return {
    items,
    word,
    definition,
    notice,
    busy,
    setWord,
    setDefinition,
    add: () => void add(),
    remove: (id: string) => void remove(id),
    exportFile: (f: "csv" | "json" | "anki") => void exportFile(f),
  };
}

export type VocabularyController = ReturnType<typeof useVocabularyController>;
