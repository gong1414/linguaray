import { createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

function InputPanel() {
  const [text, setText] = createSignal("");
  const [out, setOut] = createSignal("");
  const [err, setErr] = createSignal("");
  const [busy, setBusy] = createSignal(false);

  async function go() {
    if (!text().trim()) return;
    setBusy(true); setErr(""); setOut("");
    try {
      // default provider + target come from settings (Rust translate_default reads them).
      // to: "" is a sentinel meaning "use settings.target_language".
      const res = await invoke<{ text: string; engine: string }>("translate_default", {
        req: { text: text(), from: "auto", to: "", options: {} },
      });
      setOut(res.text);
    } catch (e) {
      setErr(String(e));
    } finally { setBusy(false); }
  }

  return (
    <main class="container" style={{ padding: "12px" }}>
      <textarea rows={4} placeholder="输入要翻译的文本…"
        value={text()} onInput={(e) => setText(e.currentTarget.value)}
        onKeyDown={(e) => { if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); go(); } }} />
      <button onClick={go} disabled={busy() || !text().trim()}>{busy() ? "…" : "Translate"}</button>
      {out() && <div class="result">{out()}</div>}
      {err() && <div class="error">{err()}</div>}
    </main>
  );
}
export default InputPanel;
