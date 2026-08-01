import { createSignal, onMount } from "solid-js";
import { listen } from "@tauri-apps/api/event";
import "./App.css";
import "./Popup.css";

type Payload = { status: string; text: string; engine: string };

function Popup() {
  const [status, setStatus] = createSignal("loading");
  const [text, setText] = createSignal("");
  const [engine, setEngine] = createSignal("");

  onMount(async () => {
    await listen<Payload>("popup-state", (e) => {
      setStatus(e.payload.status);
      setText(e.payload.text);
      setEngine(e.payload.engine);
    });
    // Hide on blur (clicking elsewhere dismisses the popup).
    const { getCurrentWindow } = await import("@tauri-apps/api/window");
    const win = getCurrentWindow();
    await win.onFocusChanged(({ payload: focused }) => { if (!focused) win.hide(); });
  });

  return (
    <main class="container" style={{ "min-height": "60px", padding: "10px" }}>
      {status() === "loading" && <div>…</div>}
      {status() === "result" && (
        <div>
          <div class="result">{text()}</div>
          {engine() && <div style={{ color: "#888", "font-size": "11px" }}>{engine()}</div>}
        </div>
      )}
      {status() === "error" && <div class="error">{text()}</div>}
    </main>
  );
}
export default Popup;
