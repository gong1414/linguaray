import { createSignal, For, onMount, Show, type Component } from "solid-js";
import { Button, EmptyState, TextField } from "@linguaray/ui";
import { BookOpen } from "lucide-solid";
import { invoke } from "@tauri-apps/api/core";
import { detectLocale } from "../../i18n";
import "./DictionaryView.css";

const COPY = {
  en: {
    title: "Dictionary",
    lookup: "Look up",
    word: "Word",
    noPackages: "No offline packages installed",
    noResult: "No definition found",
    source: "Source: {source}",
  },
  zh: {
    title: "词典",
    lookup: "查询",
    word: "单词",
    noPackages: "尚未安装离线词库",
    noResult: "未找到释义",
    source: "来源：{source}",
  },
};

export const DictionaryView: Component = () => {
  const t = COPY[detectLocale()];
  const [word, setWord] = createSignal("");
  const [result, setResult] = createSignal<{ definition: string; source: string } | null>(null);
  const [miss, setMiss] = createSignal(false);
  const [packages, setPackages] = createSignal<{ package_id: string; name: string }[]>([]);
  const [error, setError] = createSignal("");

  onMount(() => {
    void invoke<{ package_id: string; name: string }[]>("dict_list_packages")
      .then(setPackages)
      .catch((e) => setError(String(e)));
  });

  const lookup = async () => {
    setMiss(false);
    setResult(null);
    setError("");
    try {
      const found = await invoke<{ definition: string; source: string } | null>("dict_lookup", {
        word: word().trim(),
      });
      if (found) setResult(found);
      else setMiss(true);
    } catch (e) {
      setError(String(e));
    }
  };

  return (
    <section class="dictionary-view" aria-label={t.title}>
      <header>
        <h1>{t.title}</h1>
      </header>
      <div class="dictionary-view__lookup">
        <TextField label={t.word} value={word()} onInput={(e) => setWord(e.currentTarget.value)} />
        <Button onClick={() => void lookup()}>{t.lookup}</Button>
      </div>
      <Show when={packages().length === 0}>
        <EmptyState icon={<BookOpen size={32} />} title={t.noPackages} />
      </Show>
      <Show when={packages().length > 0}>
        <ul>
          <For each={packages()}>{(p) => <li>{p.name}</li>}</For>
        </ul>
      </Show>
      <Show when={result()}>
        <article>
          <p>{result()!.definition}</p>
          <p class="dictionary-view__source">{t.source.replace("{source}", result()!.source)}</p>
        </article>
      </Show>
      <Show when={miss()}>
        <p>{t.noResult}</p>
      </Show>
      <Show when={error()}>
        <p role="alert">{error()}</p>
      </Show>
    </section>
  );
};

export default DictionaryView;
