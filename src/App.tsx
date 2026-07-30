import { createSignal, For, onMount, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

/** Mirrors src-tauri/src/lib.rs `EngineInfo`. */
type EngineInfo = {
  id: string;
  label: string;
  kind: "provider" | "traditional";
  needs_key: boolean;
};

type TranslateRequest = {
  text: string;
  from: string;
  to: string;
  options?: unknown;
};

type TranslateResult = {
  text: string;
  engine: string;
};

function App() {
  const [engines, setEngines] = createSignal<EngineInfo[]>([]);
  const [selected, setSelected] = createSignal<string>("");
  const [input, setInput] = createSignal("");
  const [output, setOutput] = createSignal("");
  const [error, setError] = createSignal("");
  const [busy, setBusy] = createSignal(false);

  onMount(async () => {
    const list = await invoke<EngineInfo[]>("list_engines");
    setEngines(list);
    // Default to the first AI provider (the headline feature).
    setSelected(list.find((e) => e.kind === "provider")?.id ?? list[0]?.id ?? "");
  });

  async function doTranslate() {
    if (!input().trim() || !selected()) return;
    setBusy(true);
    setError("");
    setOutput("");
    const req: TranslateRequest = {
      text: input(),
      from: "auto",
      to: "zh",
      options: {},
    };
    try {
      const res = await invoke<TranslateResult>("translate", {
        req,
        engine: selected(),
      });
      setOutput(res.text);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <main class="container">
      <h1>IslandPot</h1>
      <p class="subtitle">fill-key-and-use translation — v1 scaffold</p>

      <select
        value={selected()}
        onChange={(e) => setSelected(e.currentTarget.value)}
        disabled={engines().length === 0}
      >
        <For each={engines()} fallback={<option>loading engines…</option>}>
          {(e) => <option value={e.id}>{e.label}</option>}
        </For>
      </select>

      <textarea
        rows={4}
        placeholder="输入要翻译的文本…"
        value={input()}
        onInput={(e) => setInput(e.currentTarget.value)}
      />

      <button onClick={doTranslate} disabled={busy() || !input().trim()}>
        {busy() ? "…" : "Translate"}
      </button>

      <Show when={output()}>
        <div class="result">{output()}</div>
      </Show>
      <Show when={error()}>
        <div class="error">{error()}</div>
      </Show>
    </main>
  );
}

export default App;
