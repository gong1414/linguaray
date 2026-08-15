import { createSignal, For, onMount, Show, type Component } from "solid-js";
import { Button, EmptyState, TextField } from "@linguaray/ui";
import { BookMarked } from "lucide-solid";
import { invoke } from "../../bridge/invoke";
import { detectLocale } from "../../i18n";
import "./VocabularyView.css";

type Item = {
  item_uuid: string;
  word: string;
  definition: string;
  source_language: string;
  target_language: string;
};

const COPY = {
  en: {
    title: "Vocabulary",
    empty: "No saved words yet",
    hint: "Save words from translations to build your list.",
    word: "Word",
    definition: "Definition",
    add: "Add",
    delete: "Delete",
    exportCsv: "Export CSV",
    exportJson: "Export JSON",
    exportAnki: "AnkiConnect",
    exportDone: "Export complete",
    exportFailed: "Export failed",
  },
  zh: {
    title: "生词本",
    empty: "暂无保存的单词",
    hint: "从翻译中保存单词以建立列表。",
    word: "单词",
    definition: "释义",
    add: "添加",
    delete: "删除",
    exportCsv: "导出 CSV",
    exportJson: "导出 JSON",
    exportAnki: "AnkiConnect",
    exportDone: "导出完成",
    exportFailed: "导出失败",
  },
};

export const VocabularyView: Component = () => {
  const t = COPY[detectLocale()];
  const [items, setItems] = createSignal<Item[]>([]);
  const [word, setWord] = createSignal("");
  const [definition, setDefinition] = createSignal("");
  const [notice, setNotice] = createSignal("");

  const reload = async () => {
    const page = await invoke<{ items: Item[] }>("vocabulary_list", { cursor: null });
    setItems(page.items);
  };

  onMount(() => {
    void reload().catch((e) => setNotice(String(e)));
  });

  const add = async () => {
    if (!word().trim()) return;
    await invoke("vocabulary_add", {
      word: word().trim(),
      definition: definition().trim(),
      sourceLanguage: "auto",
      targetLanguage: "zh",
    });
    setWord("");
    setDefinition("");
    await reload();
  };

  const remove = async (id: string) => {
    await invoke("vocabulary_delete", { itemUuid: id });
    await reload();
  };

  const exportFile = async (format: string) => {
    try {
      if (format === "anki") {
        await invoke("vocabulary_export_anki", { deckName: "LinguaRay" });
        setNotice(t.exportDone);
        return;
      }
      const { chooseExportPath } = await import("./history-ipc");
      const filePath = await chooseExportPath(`linguaray-vocabulary.${format}`);
      if (!filePath) return;
      const path = await invoke<string>("vocabulary_export_file", {
        filePath,
        format,
      });
      setNotice(path);
    } catch (e) {
      setNotice(`${t.exportFailed}: ${String(e)}`);
    }
  };

  return (
    <section class="vocabulary-view" aria-label={t.title}>
      <header>
        <h1>{t.title}</h1>
      </header>
      <div class="vocabulary-view__add">
        <TextField label={t.word} value={word()} onInput={(e) => setWord(e.currentTarget.value)} />
        <TextField
          label={t.definition}
          value={definition()}
          onInput={(e) => setDefinition(e.currentTarget.value)}
        />
        <Button onClick={() => void add()}>{t.add}</Button>
        <Button variant="ghost" onClick={() => void exportFile("csv")}>
          {t.exportCsv}
        </Button>
        <Button variant="ghost" onClick={() => void exportFile("json")}>
          {t.exportJson}
        </Button>
        <Button variant="ghost" onClick={() => void exportFile("anki")}>
          {t.exportAnki}
        </Button>
      </div>
      <Show when={notice()}>
        <p role="status">{notice()}</p>
      </Show>
      <Show when={items().length === 0}>
        <EmptyState icon={<BookMarked size={32} />} title={t.empty} description={t.hint} />
      </Show>
      <Show when={items().length > 0}>
        <ul class="vocabulary-view__list">
          <For each={items()}>
            {(item) => (
              <li>
                <strong>{item.word}</strong>
                <span>{item.definition}</span>
                <Button variant="ghost" onClick={() => void remove(item.item_uuid)}>
                  {t.delete}
                </Button>
              </li>
            )}
          </For>
        </ul>
      </Show>
    </section>
  );
};

export default VocabularyView;
