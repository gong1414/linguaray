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
  const [defaultProvider, setDefaultProvider] = createSignal("");
  const [targetLang, setTargetLang] = createSignal("zh");
  const [clipBusy, setClipBusy] = createSignal(false);
  const [a11yOk, setA11yOk] = createSignal(true);
  const [ksHealth, setKsHealth] = createSignal("");

  async function refreshA11y() {
    try { setA11yOk(await invoke<boolean>("a11y_status")); }
    catch { setA11yOk(true); } // non-macOS or command missing → assume ok
  }

  onMount(async () => {
    const list = await invoke<EngineInfo[]>("list_engines");
    setEngines(list);
    const status = await invoke<Record<string, boolean>>("key_status");
    setHasKey(status);
    // Review P1 #6: keystore fail-closed recovery — read health (does not throw),
    // surface a banner if unreadable. onMount never aborts on a corrupt keystore.
    setKsHealth(await invoke<string>("keystore_health"));
    setSelected(list.find((e) => e.kind === "provider")?.id ?? list[0]?.id ?? "");
    const s = await invoke<{ default_provider: string; target_language: string }>("get_settings");
    setDefaultProvider(s.default_provider);
    setTargetLang(s.target_language);
    refreshA11y(); // Accessibility onboarding (macOS): show banner if not granted
    // Re-check when the window regains focus (user just granted permission).
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    await getCurrentWindow().onFocusChanged(({ payload: focused }) => { if (focused) refreshA11y(); });
  });

  async function changeDefault(v: string) {
    setDefaultProvider(v);
    await invoke("set_setting", { key: "default_provider", value: v });
  }
  async function changeTarget(v: string) {
    setTargetLang(v);
    await invoke("set_setting", { key: "target_language", value: v });
  }
  async function translateClip() {
    setClipBusy(true);
    try {
      await invoke("translate_clipboard");
    } catch (e) {
      setError(String(e));
    } finally {
      setClipBusy(false);
    }
  }

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
        req: { text: input(), from: "auto", to: targetLang(), options: {} },
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
      <Show when={!a11yOk()}>
        <div class="error" style={{ "margin-bottom": "0.5rem" }}>
          Accessibility permission needed to read your selection. Grant it in System
          Settings → Privacy → Accessibility, then re-focus this window.{" "}
          <button onClick={refreshA11y} style={{ display: "inline" }}>Re-check</button>
        </div>
      </Show>
      <Show when={ksHealth() !== ""}>
        <div class="error" style={{ "margin-bottom": "0.5rem" }}>
          Keystore unreadable: {ksHealth()}. Your keys are preserved on disk.{" "}
          <button
            style={{ display: "inline" }}
            onClick={async () => {
              if (confirm("Archive the unreadable keystore (renamed to .broken-*) so you can re-enter keys?")) {
                await invoke("archive_keystore");
                setKsHealth("");
              }
            }}
          >Archive &amp; re-enter</button>{" "}
          <button
            style={{ display: "inline" }}
            onClick={async () => {
              if (confirm("Reset the keystore? The current file is archived (recoverable) to keystore.json.broken-* and a fresh one starts on next key entry.")) {
                await invoke("reset_keystore");
                setKsHealth("");
              }
            }}
          >Reset</button>
        </div>
      </Show>
      <select value={selected()} onChange={(e) => setSelected(e.currentTarget.value)}>
        <For each={engines()}>{(e) => <option value={e.id}>{e.label}{hasKey()[e.id] ? " ✓" : ""}</option>}</For>
      </select>
      <input type="password" placeholder="API key…" value={keyInput()} onInput={(e) => setKeyInput(e.currentTarget.value)} />
      <button onClick={saveKey} disabled={!keyInput()}>Save key</button>
      <div class="settings-group">
        <label>Default provider</label>
        <select value={defaultProvider()} onChange={(e) => changeDefault(e.currentTarget.value)}>
          <For each={engines()}>{(e) => <option value={e.id}>{e.label}</option>}</For>
        </select>
        <label>Target language</label>
        <select value={targetLang()} onChange={(e) => changeTarget(e.currentTarget.value)}>
          <For each={["zh", "en", "ja", "ko", "fr", "de", "es"]}>{(l) => <option value={l}>{l}</option>}</For>
        </select>
        <button onClick={translateClip} disabled={clipBusy()}>{clipBusy() ? "…" : "Translate clipboard"}</button>
      </div>
      <textarea rows={4} placeholder="输入要翻译的文本…" value={input()} onInput={(e) => setInput(e.currentTarget.value)} />
      <button onClick={doTranslate} disabled={busy() || !input().trim()}>{busy() ? "…" : "Translate"}</button>
      <Show when={output()}><div class="result">{output()}</div></Show>
      <Show when={error()}><div class="error">{error()}</div></Show>
    </main>
  );
}
export default App;
