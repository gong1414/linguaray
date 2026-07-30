import { createSignal, For, onMount, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

type EngineInfo = { id: string; label: string; kind: string; needs_key: boolean };

function App() {
  const [engines, setEngines] = createSignal<EngineInfo[]>([]);
  const [selected, setSelected] = createSignal("");
  const [keyInput, setKeyInput] = createSignal("");
  const [hasKey, setHasKey] = createSignal<Record<string, boolean>>({});
  const [input, setInput] = createSignal("");
  const [output, setOutput] = createSignal("");
  const [error, setError] = createSignal("");
  const [busy, setBusy] = createSignal(false);

  onMount(async () => {
    const list = await invoke<EngineInfo[]>("list_engines");
    setEngines(list);
    const status = await invoke<Record<string, boolean>>("key_status");
    setHasKey(status);
    setSelected(list.find((e) => e.kind === "provider")?.id ?? list[0]?.id ?? "");
  });

  async function saveKey() {
    if (!selected() || !keyInput()) return;
    await invoke("set_key", { providerId: selected(), key: keyInput() });
    setKeyInput("");
    const status = await invoke<Record<string, boolean>>("key_status");
    setHasKey(status);
  }

  async function doTranslate() {
    if (!input().trim() || !selected()) return;
    setBusy(true); setError(""); setOutput("");
    try {
      const res = await invoke<{ text: string; engine: string }>("translate", {
        req: { text: input(), from: "auto", to: "zh", options: {} },
        engine: selected(),
      });
      setOutput(res.text);
    } catch (e) {
      setError(String(e));
    } finally { setBusy(false); }
  }

  return (
    <main class="container">
      <h1>IslandPot</h1>
      <select value={selected()} onChange={(e) => setSelected(e.currentTarget.value)}>
        <For each={engines()}>{(e) => <option value={e.id}>{e.label}{hasKey()[e.id] ? " ✓" : ""}</option>}</For>
      </select>
      <input type="password" placeholder="API key…" value={keyInput()} onInput={(e) => setKeyInput(e.currentTarget.value)} />
      <button onClick={saveKey} disabled={!keyInput()}>Save key</button>
      <textarea rows={4} placeholder="输入要翻译的文本…" value={input()} onInput={(e) => setInput(e.currentTarget.value)} />
      <button onClick={doTranslate} disabled={busy() || !input().trim()}>{busy() ? "…" : "Translate"}</button>
      <Show when={output()}><div class="result">{output()}</div></Show>
      <Show when={error()}><div class="error">{error()}</div></Show>
    </main>
  );
}
export default App;
