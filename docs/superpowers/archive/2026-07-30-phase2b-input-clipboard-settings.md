Archived-on: 2026-08-14 · reason: superseded by linguaray-plugin-core-design / completed, see git history

# Phase 2b: Input Translate + User-Initiated Clipboard + Settings UX — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete the translation surface so v1 has all three input modes — selection (done in 2a), **input** (type text to translate), and **user-initiated clipboard** (translate the current clipboard on demand) — plus settings persistence (default provider + target language), replacing 2a's hardcoded `"openai"`/`"zh"`.

**Architecture:** A second hotkey (`Ctrl+Space`) opens the **input window** — a small frameless window with a textarea; typing + Enter translates via the Phase-1 service and shows the result inline. A third action (a button in the main window: "Translate clipboard") reads the current clipboard once and translates it into the popup. Both reuse the Phase-1 `service::translate` and the 2a `popup`/`concurrency` machinery. **Settings** (default provider id, target language) are persisted via `tauri-plugin-store` and read by the hotkey handler / input window instead of the hardcoded values.

**Tech Stack:** Rust 1.95 · Tauri 2 · `tauri-plugin-store` (settings JSON) · existing modules (`service`, `providers`, `keystore`, `wire`, `popup`, `concurrency`, `clipboard`). Frontend: SolidJS — a new `InputPanel.tsx` window + settings UI in `App.tsx`.

**Spec reference:** `docs/superpowers/specs/2026-07-30-linguaray-v1-design.md` (§Scope: input translate + USER-INITIATED clipboard translate — passive listening is OUT; §G error classification; §F settings UI = provider catalog list + per-row key/model/default).

---

## File Structure

**Create:**
- `src-tauri/src/settings.rs` — typed settings (default provider id, target language) over `tauri-plugin-store`; load with defaults, save.
- `src/InputPanel.tsx` + `src/input-entry.tsx` — the input-translate window (textarea → translate → show result inline).
- `input.html` — entry HTML for the input window.

**Modify:**
- `src-tauri/Cargo.toml` — add `tauri-plugin-store`.
- `src-tauri/src/lib.rs` — register the store plugin; add `set_setting`/`get_settings` commands; add an input-hotkey handler (`Ctrl+Space`) that shows the input window; add a `translate_clipboard` command (user-initiated); replace hardcoded `"openai"`/`"zh"` in the selection handler with settings reads; manage a `Session` that can read settings.
- `src-tauri/tauri.conf.json` — add a third window for the input panel (frameless, hidden initially).
- `src-tauri/capabilities/default.json` — add `"input"` to windows; add `store:default` permission.
- `vite.config.ts` — add `input.html` to the multi-page build.
- `src/App.tsx` — add settings UI (choose default provider, target language) + a "Translate clipboard" button.

---

## Task 1: Settings persistence (tauri-plugin-store)

**Files:** Modify `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`, `src-tauri/capabilities/default.json`; Create `src-tauri/src/settings.rs`

- [ ] **Step 1: Add dep + capability.** In `src-tauri/Cargo.toml` `[dependencies]` add `tauri-plugin-store = "2"`. In `src-tauri/capabilities/default.json`, add `"store:default"` to `permissions` (keep existing).

- [ ] **Step 2: Create `src-tauri/src/settings.rs`:**

```rust
//! Typed settings over tauri-plugin-store (default provider id + target language).
//! Replaces 2a's hardcoded "openai"/"zh".
use serde::{Deserialize, Serialize};

const STORE_FILE: &str = "settings.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub default_provider: String,
    pub target_language: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self { default_provider: "openai".into(), target_language: "zh".into() }
    }
}

/// Load settings, falling back to defaults for missing keys (serde fills via Default).
pub fn load(app: &tauri::AppHandle) -> Settings {
    use tauri_plugin_store::StoreExt;
    let store = match app.store(STORE_FILE) {
        Ok(s) => s,
        Err(_) => return Settings::default(),
    };
    let provider = store
        .get("default_provider")
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| Settings::default().default_provider);
    let target = store
        .get("target_language")
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_else(|| Settings::default().target_language);
    Settings { default_provider: provider, target_language: target }
}

/// Save settings (writes through to disk).
pub fn save(app: &tauri::AppHandle, s: &Settings) -> Result<(), String> {
    use tauri_plugin_store::StoreExt;
    let store = app.store(STORE_FILE).map_err(|e| e.to_string())?;
    store.set("default_provider", serde_json::json!(s.default_provider));
    store.set("target_language", serde_json::json!(s.target_language));
    store.save().map_err(|e| e.to_string())
}
```

> **Implementer note:** verify `app.store(path)` returns a usable `Store` and that `.get(key)` returns `Option<serde_json::Value>` and `.save()` persists. The `StoreExt` trait is correct for current tauri-plugin-store 2.x. If the API differs (e.g. `store` returns `Result<Arc<Store>>` vs `Store`), adapt minimally. The `set` signature takes `(impl Into<String>, impl Into<Value>)`.

- [ ] **Step 3: Register plugin + module + check.** Add `pub mod settings;` to lib.rs. In `run()`, add `.plugin(tauri_plugin_store::Builder::new().build())` to the Builder chain (after `tauri_plugin_opener::init()`). `cargo check`. Expected: Finished.

- [ ] **Step 4: Commit.**
```bash
cd /Users/daoyu/Code/projects/linguaray && git checkout -b phase2b && git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/capabilities/default.json src-tauri/src/settings.rs src-tauri/src/lib.rs && git -c user.name=daoyu -c user.email=daoyu@local commit -m "feat(settings): typed settings over tauri-plugin-store"
```
> Create the `phase2b` branch in this commit (first task of the phase).

---

## Task 2: Settings Tauri commands + wire into selection handler

**Files:** Modify `src-tauri/src/lib.rs`

- [ ] **Step 1: Add commands + replace hardcoded values.** In `lib.rs`:

Add two commands:
```rust
#[tauri::command]
fn get_settings(app: tauri::AppHandle) -> settings::Settings {
    settings::load(&app)
}

#[tauri::command]
fn set_setting(app: tauri::AppHandle, key: String, value: String) -> Result<(), String> {
    let mut s = settings::load(&app);
    match key.as_str() {
        "default_provider" => s.default_provider = value,
        "target_language" => s.target_language = value,
        _ => return Err(format!("unknown setting: {key}")),
    }
    settings::save(&app, &s)
}
```
Register both in the `invoke_handler!` list.

In the selection hotkey handler (the spawned task), replace the hardcoded values:
```rust
// was:
//   let preset = providers::presets().into_iter().find(|p| p.id == "openai").expect(...);
//   let input = service::TranslateInput { text: &text, from: "auto", to: "zh", ... };
let s = settings::load(&app2);
let preset = providers::presets().into_iter()
    .find(|p| p.id == s.default_provider)
    .ok_or_else(|| format!("default provider '{}' not found", s.default_provider));
let preset = match preset { Ok(p) => p, Err(e) => { let _ = popup::error(&app2, &e); return; } };
let input = service::TranslateInput { text: &text, from: "auto", to: &s.target_language, options: Default::default() };
```

- [ ] **Step 2: cargo check + cargo test (no regression).** `cargo check` Finished; `cargo test` — all prior tests still pass (settings has no test yet; that's fine, it's thin glue).

- [ ] **Step 3: Commit.**
```bash
git add src-tauri/src/lib.rs && git -c user.name=daoyu -c user.email=daoyu@local commit -m "feat(settings): get_settings/set_setting commands + wire into selection handler"
```

---

## Task 3: Input-translate window (config + entry + component)

**Files:** Modify `src-tauri/tauri.conf.json`, `src-tauri/capabilities/default.json`, `vite.config.ts`; Create `input.html`, `src/input-entry.tsx`, `src/InputPanel.tsx`

- [ ] **Step 1: Third window config.** In `src-tauri/tauri.conf.json` `"windows"` array, add:
```json
{
  "label": "input",
  "title": "LinguaRay — Translate",
  "url": "input.html",
  "width": 420,
  "height": 280,
  "visible": false,
  "alwaysOnTop": true,
  "skipTaskbar": false,
  "resizable": true,
  "focus": true
}
```
(The input window has decorations — it's a focused tool window, not a transient popup.)

- [ ] **Step 2: capability.** In `capabilities/default.json`, change `"windows"` to `["main", "popup", "input"]`.

- [ ] **Step 3: entry HTML + entry tsx.** Create `input.html` (sibling of index.html):
```html
<!DOCTYPE html>
<html lang="en"><head><meta charset="UTF-8" /><title>LinguaRay</title></head>
<body><div id="root"></div><script type="module" src="/src/input-entry.tsx"></script></body>
</html>
```
Create `src/input-entry.tsx`:
```tsx
import { render } from "solid-js/web";
import InputPanel from "./InputPanel";
render(() => <InputPanel />, document.getElementById("root")!);
```

- [ ] **Step 4: InputPanel component.** Create `src/InputPanel.tsx`:
```tsx
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
      // default provider + target come from settings (Rust side reads them)
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
        value={text()} onInput={(e) => setText(e.currentTarget.value)} />
      <button onClick={go} disabled={busy() || !text().trim()}>{busy() ? "…" : "Translate"}</button>
      {out() && <div class="result">{out()}</div>}
      {err() && <div class="error">{err()}</div>}
    </main>
  );
}
export default InputPanel;
```
NOTE: this calls a NEW command `translate_default` (Task 4) — the input window doesn't pass a provider/to (empty `to:""` means "use settings"). The `to:""` is a sentinel that `translate_default` interprets as "use settings.target_language".

- [ ] **Step 5: vite multi-page.** In `vite.config.ts`, add `input: "input.html"` to the `build.rollupOptions.input` map: `{ main: "index.html", popup: "popup.html", input: "input.html" }`.

- [ ] **Step 6: pnpm build.** Expected: success (three bundles).

- [ ] **Step 7: Commit.**
```bash
git add src-tauri/tauri.conf.json src-tauri/capabilities/default.json input.html src/input-entry.tsx src/InputPanel.tsx vite.config.ts && git -c user.name=daoyu -c user.email=daoyu@local commit -m "feat(input): input-translate window + InputPanel component"
```

---

## Task 4: translate_default command + input hotkey + clipboard command

**Files:** Modify `src-tauri/src/lib.rs`

- [ ] **Step 1: Add `translate_default` command.** A convenience translate that resolves provider + target from settings (so the input window and clipboard command don't need to pass them). `to: ""` means "use settings.target_language":
```rust
#[tauri::command]
async fn translate_default(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<Session>>,
    req: TranslateRequest,
) -> Result<TranslateResult, String> {
    let s = settings::load(&app);
    let to = if req.to.is_empty() { s.target_language.clone() } else { req.to };
    let preset = providers::presets().into_iter()
        .find(|p| p.id == s.default_provider)
        .ok_or_else(|| format!("default provider '{}' not found", s.default_provider))?;
    let input = service::TranslateInput { text: &req.text, from: &req.from, to: &to, options: Default::default() };
    let text = service::translate(&state.client, &state.keystore, &preset, input).await.map_err(|e| e.to_string())?;
    Ok(TranslateResult { text, engine: preset.id })
}
```
Register `translate_default` in `invoke_handler!`.

- [ ] **Step 2: Register the input hotkey.** In the global-shortcut registration (the same handler or a second shortcut), add `"Ctrl+Space"` that shows the input window:
```rust
// In the handler, branch on the shortcut OR register a second shortcut with its own handler.
// If the plugin's API lets one handler serve multiple shortcuts, match on the shortcut id/str;
// else register Ctrl+Space with a separate handler:
fn on_input_hotkey(app: &tauri::AppHandle, _s: &Shortcut, event: ShortcutEvent) {
    if event.state != ShortcutState::Pressed { return; }
    let app2 = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Some(win) = app2.get_webview_window("input") {
            let _ = win.show();
            let _ = win.set_focus();
        }
    });
}
```
Match the real plugin API for registering a SECOND shortcut + handler (Task 10 of 2a established the API — reuse it; if the builder takes an array of `(shortcut, handler)`, add the pair there).

- [ ] **Step 3: Add `translate_clipboard` command** (user-initiated — reads the current clipboard ONCE and translates into the popup at the cursor):
```rust
#[tauri::command]
async fn translate_clipboard(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<Session>>,
) -> Result<(), String> {
    let text = clipboard::get_text()?;
    if text.trim().is_empty() { return Err("clipboard empty".into()); }
    let (x, y) = cursor::position();
    let _ = popup::show_at(&app, x, y);
    let s = settings::load(&app);
    let preset = providers::presets().into_iter()
        .find(|p| p.id == s.default_provider)
        .ok_or_else(|| format!("default provider '{}' not found", s.default_provider))?;
    let input = service::TranslateInput { text: &text, from: "auto", to: &s.target_language, options: Default::default() };
    match service::translate(&state.client, &state.keystore, &preset, input).await {
        Ok(out) => { let _ = popup::result(&app, &out, &preset.id); }
        Err(e) => { let _ = popup::error(&app, &e.to_string()); }
    }
    Ok(())
}
```
Register `translate_clipboard` in `invoke_handler!`.

> NOTE for implementer: `translate_clipboard` reads the clipboard ONCE on user action — it is NOT a background listener. This matches spec §Scope (user-initiated clipboard translate; passive listening removed). No sentinel/restore logic here — we're only reading, not simulating a copy.

- [ ] **Step 4: cargo check + cargo test.** Expected: Finished; all prior tests pass.

- [ ] **Step 5: Commit.**
```bash
git add src-tauri/src/lib.rs && git -c user.name=daoyu -c user.email=daoyu@local commit -m "feat(input): translate_default cmd + input hotkey + user-initiated clipboard cmd"
```

---

## Task 5: Settings UI + clipboard button in the main window

**Files:** Modify `src/App.tsx`

- [ ] **Step 1: Extend App.tsx.** Add (alongside the existing provider picker + key input): a "default provider" selector (dropdown of all engines, bound to `get_settings`/`set_setting`), a "target language" selector (a small fixed set: zh, en, ja, ko, fr, de, es — expandable), and a "Translate clipboard" button (calls `translate_clipboard`). READ the current App.tsx first; extend it, don't clobber.

Sketch (integrate into the existing component):
```tsx
// new state
const [defaultProvider, setDefaultProvider] = createSignal("");
const [targetLang, setTargetLang] = createSignal("zh");
const [clipBusy, setClipBusy] = createSignal(false);

// in onMount, after existing: load settings
const s = await invoke<{ default_provider: string; target_language: string }>("get_settings");
setDefaultProvider(s.default_provider);
setTargetLang(s.target_language);

// default-provider change
async function changeDefault(v: string) {
  setDefaultProvider(v);
  await invoke("set_setting", { key: "default_provider", value: v });
}
// target-language change
async function changeTarget(v: string) {
  setTargetLang(v);
  await invoke("set_setting", { key: "target_language", value: v });
}
// clipboard button
async function translateClip() {
  setClipBusy(true);
  try { await invoke("translate_clipboard"); } catch (e) { setError(String(e)); }
  finally { setClipBusy(false); }
}
```
Add to JSX: a `<select>` for default provider (options from `engines()`), a `<select>` for target language, and a `<button onClick={translateClip}>` "Translate clipboard".

- [ ] **Step 2: pnpm build.** Expected: success.

- [ ] **Step 3: Commit.**
```bash
git add src/App.tsx && git -c user.name=daoyu -c user.email=daoyu@local commit -m "feat(ui): settings (default provider, target lang) + translate-clipboard button"
```

---

## Task 6: Manual end-to-end verification

**Files:** none (manual)

- [ ] **Step 1: `pnpm tauri dev`.** Grant macOS Accessibility (needed for the selection hotkey's simulated copy).
- [ ] **Step 2: Settings.** In the main window, set a default provider (e.g. ollama if local, or openai with a key) and a target language. Restart the app — verify settings persisted.
- [ ] **Step 3: Selection** (regression from 2a): select text elsewhere, press Alt+Space → popup shows translation using the *chosen* default provider + target (not hardcoded).
- [ ] **Step 4: Input:** press Ctrl+Space → input window opens → type text → Translate → result inline.
- [ ] **Step 5: Clipboard:** copy some text manually, click "Translate clipboard" → popup shows translation. (Verify it does NOT trigger on its own without the button — passive listening must not exist.)
- [ ] **Step 6: latest-wins:** Alt+Space twice quickly with different selections → settles on the latest.

> No commit. If issues found, file concrete follow-ups. Mark done once all three modes work and settings persist.

---

## Self-Review (run after writing; fix inline)

- **Spec coverage:** §Scope input translate → Tasks 3-4. §Scope user-initiated clipboard translate → Task 4 (`translate_clipboard`, read-once, no listener). §F settings UI (default provider + per-row key, already from Phase 1; adds target language) → Tasks 1-2,5. §G error classification → Phase-1 `service` reused. Passive clipboard listening → explicitly NOT built (matches §Scope removal). 2a hardcoded `"openai"`/`"zh"` → replaced by settings reads in Task 2 (selection handler) and Task 4 (input/clipboard).
- **Placeholder scan:** Task 2's settings-load in the handler runs on every hotkey press (cheap JSON read; acceptable). Task 4's second-shortcut registration depends on the real plugin API (established in 2a Task 10) — adapt to whatever that API supports for multiple shortcuts; not a TBD, it's "reuse the known API." No "TODO/TBD".
- **Type consistency:** `Settings { default_provider, target_language }` consistent across settings.rs, get_settings, set_setting, App.tsx. `translate_default`'s `to: ""` sentinel interpreted as "use settings" — documented in the InputPanel note and the command. `Session` (from 2a) reused (no new AppState).
- **Consistency with 2a:** selection handler now reads settings instead of hardcoded values — the rest of 2a (sentinel, popup, latest-wins) is untouched.

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-07-30-phase2b-input-clipboard-settings.md`. Two execution options:

**1. Subagent-Driven (recommended)** — fresh subagent per task, two-stage review.

**2. Inline Execution** — batch with checkpoints.

**Which approach?**
