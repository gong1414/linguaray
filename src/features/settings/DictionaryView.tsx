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
    install: "Install package",
    sourceDir: "Folder path",
    packageId: "Package id",
    packageName: "Name",
    version: "Version",
    installed: "Installed",
  },
  zh: {
    title: "词典",
    lookup: "查询",
    word: "单词",
    noPackages: "尚未安装离线词库",
    noResult: "未找到释义",
    source: "来源：{source}",
    install: "安装词库",
    sourceDir: "文件夹路径",
    packageId: "词库 ID",
    packageName: "名称",
    version: "版本",
    installed: "已安装",
  },
};

export const DictionaryView: Component = () => {
  const t = COPY[detectLocale()];
  const [word, setWord] = createSignal("");
  const [result, setResult] = createSignal<{ definition: string; source: string } | null>(null);
  const [miss, setMiss] = createSignal(false);
  const [packages, setPackages] = createSignal<{ package_id: string; name: string }[]>([]);
  const [error, setError] = createSignal("");
  const [sourceDir, setSourceDir] = createSignal("");
  const [packageId, setPackageId] = createSignal("");
  const [packageName, setPackageName] = createSignal("");
  const [version, setVersion] = createSignal("1.0");
  const [notice, setNotice] = createSignal("");

  const reloadPackages = async () => {
    const listed = await invoke<{ package_id: string; name: string }[]>("dict_list_packages");
    setPackages(listed);
  };

  onMount(() => {
    void reloadPackages().catch((e) => setError(String(e)));
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

  const install = async () => {
    setError("");
    setNotice("");
    try {
      await invoke("dict_install_package", {
        sourceDir: sourceDir().trim(),
        packageId: packageId().trim(),
        name: packageName().trim() || packageId().trim(),
        version: version().trim() || "1.0",
      });
      setNotice(t.installed);
      await reloadPackages();
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
      <div class="dictionary-view__install">
        <TextField
          label={t.sourceDir}
          value={sourceDir()}
          onInput={(e) => setSourceDir(e.currentTarget.value)}
        />
        <TextField
          label={t.packageId}
          value={packageId()}
          onInput={(e) => setPackageId(e.currentTarget.value)}
        />
        <TextField
          label={t.packageName}
          value={packageName()}
          onInput={(e) => setPackageName(e.currentTarget.value)}
        />
        <TextField
          label={t.version}
          value={version()}
          onInput={(e) => setVersion(e.currentTarget.value)}
        />
        <Button onClick={() => void install()} disabled={!sourceDir().trim() || !packageId().trim()}>
          {t.install}
        </Button>
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
      <Show when={notice()}>
        <p role="status">{notice()}</p>
      </Show>
      <Show when={error()}>
        <p role="alert">{error()}</p>
      </Show>
    </section>
  );
};

export default DictionaryView;
