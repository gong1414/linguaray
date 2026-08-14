Archived-on: 2026-08-14 · reason: superseded by linguaray-plugin-core-design / completed, see git history

# Phase 2a: Selection-Translate Minimum Loop — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** The minimum usable closed loop: user selects text anywhere, presses the global hotkey, and the translation appears in a frameless popup anchored at the cursor. This is the first thing that "feels like pot/Easydict."

**Architecture:** A `tauri-plugin-global-shortcut` handler fires on the hotkey; it (1) allocates a monotonic generation token (latest-wins), (2) captures the cursor position via the `mouse_position` crate, (3) captures the selection via a vendored `get-selected-text` running the §B sentinel state machine, (4) shows a second frameless Tauri window at the cursor, (5) calls `translate_service` (from Phase 1), (6) renders the result into the popup only if still the latest generation. Selection capture happens in the hotkey handler BEFORE the popup steals focus, and the token is re-checked at every transition.

**Tech Stack:** Rust 1.95 · Tauri 2 · `tauri-plugin-global-shortcut` · `mouse_position` (cross-platform cursor) · vendored `get-selected-text` (sentinel clipboard algorithm) · `enigo` (simulate Cmd+C/Ctrl+C) · existing Phase-1 modules (`service`, `providers`, `keystore`, `wire`). Frontend: SolidJS, a new `Popup.tsx` window.

**Spec reference:** `docs/superpowers/specs/2026-07-30-linguaray-v1-design.md` (§B selection capture + sentinel algorithm, §C cursor-anchored popup, §D hotkey, §concurrency generation-token latest-wins, §G error classification).

---

## File Structure

**Create:**
- `src-tauri/src/selection.rs` — vendored selection capture: the §B sentinel state machine over a thin clipboard abstraction. Pure-Rust, testable with a fake clipboard.
- `src-tauri/src/clipboard.rs` — tiny platform clipboard abstraction (get text, set text, sequence number). macOS: `NSPasteboard` via `cocoa`/`objc` OR the `arboard` crate (simpler). Windows: `GetClipboardSequenceNumber` + `arboard`. **Decision: use `arboard`** for get/set (cross-platform) + a small per-platform sequence-number getter, since `arboard` does not expose the OS sequence number.
- `src-tauri/src/cursor.rs` — `cursor_position() -> (i32,i32)` wrapper over `mouse_position`.
- `src-tauri/src/concurrency.rs` — `GenerationToken` (atomic monotonic counter) + a `SelectionMutex`.
- `src-tauri/src/selection_engine.rs` — orchestrates the §B algorithm against the clipboard abstraction (this is where the sentinel logic lives, decoupled from the OS clipboard so it's unit-testable with a fake).
- `src-tauri/src/popup.rs` — show/move/hide the frameless popup window at a screen point; set its translation payload.
- `src/Popup.tsx` + `src/popup-entry.tsx` — the popup window's SolidJS app (shows loading → text/error, tagged with engine).
- `src-tauri/tests/selection_engine.rs` — fake-clipboard tests of the sentinel state machine (success, NoSelection-timeout, newer-writer-no-restore).

**Modify:**
- `src-tauri/Cargo.toml` — add `tauri-plugin-global-shortcut`, `mouse_position`, `arboard`, `enigo`.
- `src-tauri/src/lib.rs` — add modules; wire the global-shortcut handler in `run()`; manage a shared `Session` (reqwest client, keystore, generation token, selection mutex).
- `src-tauri/tauri.conf.json` — add a second window config for the popup (frameless, transparent, visible:false initially, decorations:false).
- `src-tauri/capabilities/default.json` — grant `global-shortcut:default` permission.
- `index.html` / new `popup.html` — entry HTML for the popup window.

---

## Task 1: Add dependencies + capability

**Files:** Modify `src-tauri/Cargo.toml`, `src-tauri/capabilities/default.json`

- [ ] **Step 1: Add Cargo deps.** Add to `[dependencies]` in `src-tauri/Cargo.toml`:
```toml
tauri-plugin-global-shortcut = "2"
mouse_position = "0.1"
arboard = "3"
enigo = "0.2"
```

- [ ] **Step 2: Add capability.** In `src-tauri/capabilities/default.json`, add `"global-shortcut:default"` to the `permissions` array (keep existing `core:default`, `opener:default`).

- [ ] **Step 3: Verify + commit.**
Run: `cd src-tauri && cargo check` (cargo at `~/.cargo/bin/cargo`). Expected: Finished (may add crates).
```bash
cd /Users/daoyu/Code/projects/linguaray && git checkout -b phase2a && git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/capabilities/default.json && git -c user.name=daoyu -c user.email=daoyu@local commit -m "deps: global-shortcut, mouse_position, arboard, enigo for Phase 2a"
```
> NOTE for implementer: create the `phase2a` branch in this commit step (first task of the new phase).

---

## Task 2: Concurrency primitives (generation token + selection mutex)

**Files:** Create `src-tauri/src/concurrency.rs`; modify `src-tauri/src/lib.rs` (`pub mod concurrency;`)

- [ ] **Step 1: Write the failing test.** Append to `concurrency.rs` (we'll write the impl in step 3, so define module + test first):

```rust
//! Latest-wins generation token + selection mutex (spec §concurrency).
use std::sync::atomic::{AtomicU64, Ordering};
use parking_lot::Mutex;

pub struct GenerationToken {
    current: AtomicU64,
    selection: Mutex<()>,
}

impl GenerationToken {
    pub fn new() -> Self { Self { current: AtomicU64::new(0), selection: Mutex::new(()) } }
    /// Allocate the next generation; it becomes "current". Returns the new gen.
    pub fn next(&self) -> u64 {
        let g = self.current.fetch_add(1, Ordering::SeqCst) + 1;
        g
    }
    /// True iff `gen` is still the latest.
    pub fn is_latest(&self, gen: u64) -> bool {
        // current == gen OR current == gen (fetch_add means current lags by 1 of next alloc)
        self.current.load(Ordering::SeqCst) == gen
    }
    pub fn selection_lock(&self) -> parking_lot::MutexGuard<'_, ()> { self.selection.lock() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn newest_is_latest() {
        let t = GenerationToken::new();
        let a = t.next();
        let b = t.next();
        assert!(!t.is_latest(a));
        assert!(t.is_latest(b));
    }
}
```

> **Implementer note on `is_latest` semantics:** `fetch_add` returns the *old* value then stores old+1. So after `next()` returns `g`, `current` holds `g` only if no further `next()` happened. If a newer `next()` ran, current > g → not latest. But careful: `next()` does `fetch_add(..)+1`; if it returns `g`, the stored value is `g` (old+1 where old = g-1). Wait — re-derive: start current=0. `next()`: old=fetch_add(1)=0, store 1, return 0+1=1. current now 1. So after returning g, current==g. A second `next()`: old=1, store 2, return 2. current==2. So `is_latest(g)` = (current == g). Correct as written. **Verify this reasoning by running the test** — if it fails, fix `is_latest` so the test passes (the test is the source of truth).

- [ ] **Step 2: Add module + run test (expect PASS).** Add `pub mod concurrency;` to `lib.rs`. Run `cd src-tauri && cargo test concurrency`. Expected: `newest_is_latest ... ok`.

- [ ] **Step 3: Commit.**
```bash
git add src-tauri/src/concurrency.rs src-tauri/src/lib.rs && git -c user.name=daoyu -c user.email=daoyu@local commit -m "feat(concurrency): generation token (latest-wins) + selection mutex"
```

---

## Task 3: Clipboard abstraction (sequence number + get/set text)

**Files:** Create `src-tauri/src/clipboard.rs`; modify `src-tauri/src/lib.rs`

- [ ] **Step 1: Write the abstraction.** Create `src-tauri/src/clipboard.rs`:

```rust
//! Thin OS clipboard abstraction. `arboard` for get/set text; a per-platform
//! sequence number (Win: GetClipboardSequenceNumber; macOS: NSPasteboard.changeCount).
//! The sequence number is load-bearing for the §B restore guard.
use std::sync::Mutex;

// arboard::Clipboard is not Send/Sync-safe to share raw; guard it.
static CLIP: Mutex<Option<arboard::Clipboard>> = Mutex::new(None);

fn clip() -> std::result::Result<std::sync::MutexGuard<'static, Option<arboard::Clipboard>>, String> {
    let mut g = CLIP.lock().map_err(|e| e.to_string())?;
    if g.is_none() {
        *g = Some(arboard::Clipboard::new().map_err(|e| e.to_string())?);
    }
    Ok(g)
}

pub fn get_text() -> std::result::Result<String, String> {
    let mut g = clip()?;
    g.as_mut().unwrap().get_text().map_err(|e| e.to_string())
}

pub fn set_text(s: &str) -> std::result::Result<(), String> {
    let mut g = clip()?;
    g.as_mut().unwrap().set_text(s).map_err(|e| e.to_string())
}

/// Monotonic clipboard sequence number (advances on any clipboard write, ours
/// included). macOS: NSPasteboard.changeCount; Windows: GetClipboardSequenceNumber.
#[cfg(target_os = "macos")]
pub fn sequence() -> u64 {
    use objc::runtime::Object;
    use objc::{class, msg_send, sel, sel_impl};
    unsafe {
        let pb: *mut Object = msg_send![class!(NSPasteboard), generalPasteboard];
        let count: isize = msg_send![pb, changeCount];
        count as u64
    }
}

#[cfg(target_os = "windows")]
pub fn sequence() -> u64 {
    unsafe { windows_sys::Win32::System::DataExchange::GetClipboardSequenceNumber() as u64 }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn sequence() -> u64 { 0 }
```

- [ ] **Step 2: Add deps for objc (macOS) + windows-sys (Windows).** Add to `[dependencies]`:
```toml
objc = "0.2"
windows-sys = { version = "0.59", features = ["Win32_System_DataExchange", "Win32_Foundation"] }
```
(These are platform-gated at use, so including both is fine; only the relevant one compiles per target.)

- [ ] **Step 3: Add module + check.** `pub mod clipboard;` in lib.rs. Run `cargo check`. Expected: Finished. (macOS build; the windows branch is cfg-gated out.)

- [ ] **Step 4: Commit.**
```bash
git add src-tauri/src/clipboard.rs src-tauri/src/lib.rs src-tauri/Cargo.toml src-tauri/Cargo.lock && git -c user.name=daoyu -c user.email=daoyu@local commit -m "feat(clipboard): arboard get/set + per-platform sequence number"
```

---

## Task 4: Selection engine — sentinel state machine (THE core of §B), TDD with a fake clipboard

**Files:** Create `src-tauri/src/selection_engine.rs`, `src-tauri/tests/selection_engine.rs`; modify lib.rs

This is the testable heart of §B. It is written against a **trait** so tests use a fake clipboard; the real `clipboard.rs` impls the trait in Task 5.

- [ ] **Step 1: Write the trait + engine + NoSelection result.** Create `src-tauri/src/selection_engine.rs`:

```rust
//! The §B sentinel clipboard state machine, decoupled from the OS clipboard via a
//! trait so it is unit-testable with a fake. Real wiring (enigo Cmd+C, real
//! clipboard.rs) lives in selection.rs (Task 5).

/// What the engine needs from a clipboard. The real impl wraps `clipboard.rs`;
/// tests use a fake.
pub trait ClipboardLike {
    fn get_text(&self) -> std::result::Result<String, String>;
    fn set_text(&self, s: &str) -> std::result::Result<(), String>;
    /// Monotonic sequence number that advances on ANY write (ours included).
    fn sequence(&self) -> u64;
}

pub enum Capture {
    Selected(String),
    NoSelection,
}

/// Run the §B algorithm. `copy` is the simulated-copy step (Cmd+C/Ctrl+C),
/// injected so the engine stays pure/testable. Returns the selected text or
/// NoSelection (sentinel still present after `copy` ran => nothing was selected).
pub fn capture<C: ClipboardLike, F: FnMut() -> std::result::Result<(), String>>(
    clip: &C,
    mut copy: F,
    timeout_iters: usize,
) -> std::result::Result<Capture, String> {
    // 1. Save current content (best-effort; ignore read errors => empty).
    let saved = clip.get_text().unwrap_or_default();
    // 2. Write a unique sentinel.
    let sentinel = format!("__linguaray_sel_{}__", clip.sequence());
    clip.set_text(&sentinel)?;
    let marker_sequence = clip.sequence();
    // 3. Simulate copy.
    copy()?;
    // 4. Bounded-wait for the sequence to leave the marker (a successful copy
    //    overwrites the sentinel, advancing the sequence).
    let mut waited = 0usize;
    let mut now = clip.sequence();
    while now == marker_sequence && waited < timeout_iters {
        std::thread::sleep(std::time::Duration::from_millis(20));
        now = clip.sequence();
        waited += 1;
    }
    if now == marker_sequence {
        // Copy didn't happen / nothing selected. Restore saved, return NoSelection.
        let _ = clip.set_text(&saved);
        return Ok(Capture::NoSelection);
    }
    // 5. Copy succeeded: read selection, record owned_sequence, restore only if
    //    nothing else wrote since.
    let owned_sequence = clip.sequence();
    let text = clip.get_text()?;
    if clip.sequence() == owned_sequence {
        let _ = clip.set_text(&saved);
    } // else: newer content — don't clobber.
    Ok(Capture::Selected(text))
}
```

- [ ] **Step 2: Write the fake-clipboard tests.** Create `src-tauri/tests/selection_engine.rs`:

```rust
use linguaray_lib::selection_engine::{capture, ClipboardLike, Capture};

/// A fake clipboard that advances its sequence on every set, and can simulate
/// a "selection" appearing after a copy, or a competing writer.
struct Fake {
    text: String,
    seq: u64,
    /// If Some, the copy() closure will overwrite the sentinel with this text + bump seq.
    selection: Option<String>,
    /// If true, a competing writer bumps seq between copy and restore.
    competitor: bool,
}
impl ClipboardLike for Fake {
    fn get_text(&self) -> Result<String, String> { Ok(self.text.clone()) }
    fn set_text(&self, _s: &str) -> Result<(), String> { unreachable!("set needs &mut — see note") }
    fn sequence(&self) -> u64 { self.seq }
}
```

> **Implementer note (IMPORTANT):** the `ClipboardLike::set_text` signature above takes `&self` but must mutate state — that won't work with `Fake`. The engine calls `clip.set_text(...)`. To make the fake mutable, use interior mutability: give `Fake` a `Cell`/`RefCell` interior and make `set_text(&self, s)` mutate through a `RefCell`. Rework the `Fake` to use `RefCell` for `text` and `seq`, and have `set_text` bump `seq` and set `text`. Also, to simulate the copy *overwriting the sentinel*, the test's `copy` closure (passed to `capture`) must itself mutate the fake (write the selection text + bump seq). Since the closure captures by reference, give it `&Fake` (interior-mutable). Rewrite the test file fully with `RefCell`-based interior mutability so all three scenarios compile and pass. The three required tests:
> - `success`: selection present → returns `Capture::Selected("hi")`, clipboard restored to saved.
> - `no_selection`: copy does nothing (sentinel stays) → returns `Capture::NoSelection`, saved restored.
> - `competitor_wins`: a competing writer bumps seq after copy → returns `Capture::Selected(...)` but does NOT restore (newer content protected).
>
> This rework is part of the task — produce a compiling, passing test file. The engine signature in step 1 is correct; only the test harness needs interior mutability.

- [ ] **Step 3: Add module + run tests.** `pub mod selection_engine;` in lib.rs. Run `cd src-tauri && cargo test --test selection_engine`. Expected: 3 PASS.

- [ ] **Step 4: Commit.**
```bash
git add src-tauri/src/selection_engine.rs src-tauri/tests/selection_engine.rs src-tauri/src/lib.rs && git -c user.name=daoyu -c user.email=daoyu@local commit -m "feat(selection): sentinel state machine (§B) + fake-clipboard tests"
```

---

## Task 5: Selection wiring (real clipboard + enigo Cmd+C)

**Files:** Create `src-tauri/src/selection.rs`; modify lib.rs

- [ ] **Step 1: Write the OS-backed ClipboardLike + capture entrypoint.** Create `src-tauri/src/selection.rs`:

```rust
//! Wires the §B engine to the real OS clipboard + enigo keystroke simulation.
use crate::clipboard;
use crate::selection_engine::{self, Capture, ClipboardLike};

struct OsClipboard;
impl ClipboardLike for OsClipboard {
    fn get_text(&self) -> Result<String, String> { clipboard::get_text() }
    fn set_text(&self, s: &str) -> Result<(), String> { clipboard::set_text(s) }
    fn sequence(&self) -> u64 { clipboard::sequence() }
}

fn simulate_copy() -> Result<(), String> {
    use enigo::{Enigo, Key, KeyboardControllable};
    let mut enigo = Enigo::new().map_err(|e| e.to_string())?;
    #[cfg(target_os = "macos")]
    {
        enigo.key_down(Key::Meta); enigo.key_click(Key::Layout('c')); enigo.key_up(Key::Meta);
    }
    #[cfg(not(target_os = "macos"))]
    {
        enigo.key_down(Key::Control); enigo.key_click(Key::Layout('c')); enigo.key_up(Key::Control);
    }
    Ok(())
}

/// Capture the current selection via the §B algorithm. ~timeout_ms total.
pub fn capture_selection(timeout_ms: u64) -> Result<Capture, String> {
    let iters = (timeout_ms / 20) as usize;
    selection_engine::capture(&OsClipboard, || simulate_copy(), iters)
}
```

> **Implementer note:** the exact `enigo` API (Key variants, method names) may differ by the 0.2.x version resolved. Verify against `cargo doc --open` for enigo or its docs.rs page at execution time and adjust minimally. The macOS sequence (hold Meta, click c, release Meta) is the intent; Windows uses Control. If `enigo` 0.2's API differs materially, use whatever the resolved version exposes to achieve "simulate Cmd+C / Ctrl+C."

- [ ] **Step 2: Add module + check.** `pub mod selection;` in lib.rs. `cargo check`. Expected: Finished.

- [ ] **Step 3: Commit.**
```bash
git add src-tauri/src/selection.rs src-tauri/src/lib.rs && git -c user.name=daoyu -c user.email=daoyu@local commit -m "feat(selection): wire sentinel engine to OS clipboard + enigo copy"
```

---

## Task 6: Cursor position

**Files:** Create `src-tauri/src/cursor.rs`; modify lib.rs

- [ ] **Step 1: Write it.** Create `src-tauri/src/cursor.rs`:

```rust
//! Screen cursor position (spec §C popup anchoring). Tauri 2 has no built-in
//! global cursor-position API, so we use the `mouse_position` crate.
pub fn position() -> (i32, i32) {
    use mouse_position::mouse_position::{get_mouse_position, Mouse};
    match get_mouse_position() {
        Mouse::Position { x, y } => (x as i32, y as i32),
        _ => (0, 0),
    }
}
```

- [ ] **Step 2: Add module + check + commit.** `pub mod cursor;` in lib.rs. `cargo check`. Then:
```bash
git add src-tauri/src/cursor.rs src-tauri/src/lib.rs && git -c user.name=daoyu -c user.email=daoyu@local commit -m "feat(cursor): screen position via mouse_position"
```

---

## Task 7: Popup window config (Tauri) + entry HTML

**Files:** Modify `src-tauri/tauri.conf.json`; create `popup.html`; create `src/popup-entry.tsx`; modify vite config + package scripts

- [ ] **Step 1: Add popup window to tauri.conf.json.** In the `"windows"` array, add a second window:
```json
{
  "label": "popup",
  "title": "",
  "url": "popup.html",
  "width": 360,
  "height": 180,
  "decorations": false,
  "transparent": true,
  "visible": false,
  "alwaysOnTop": true,
  "skipTaskbar": true,
  "resizable": false,
  "focus": true
}
```
Keep the existing main window as the first entry.

- [ ] **Step 2: Create `popup.html`** (sibling to `index.html`):
```html
<!DOCTYPE html>
<html lang="en">
  <head><meta charset="UTF-8" /><title>LinguaRay</title></head>
  <body><div id="root"></div><script type="module" src="/src/popup-entry.tsx"></script></body>
</html>
```

- [ ] **Step 3: Create `src/popup-entry.tsx`:**
```tsx
import { render } from "solid-js/web";
import Popup from "./Popup";
render(() => <Popup />, document.getElementById("root")!);
```

- [ ] **Step 4: Vite multi-page config.** In `vite.config.ts`, add `build.rollupOptions.input` for both pages. Read the current `vite.config.ts` first, then set input to include `index` (main) and `popup`:
```ts
import { defineConfig } from "vite";
import solid from "vite-plugin-solid";
export default defineConfig({
  plugins: [solid()],
  // existing server/clearScreen config stays
  build: { rollupOptions: { input: { main: "index.html", popup: "popup.html" } } },
});
```
(Merge with existing config — don't drop existing keys like `server.port` / `clearScreen`.)

- [ ] **Step 5: Verify frontend build.** `pnpm build`. Expected: success (both bundles).

- [ ] **Step 6: Commit.**
```bash
git add src-tauri/tauri.conf.json popup.html src/popup-entry.tsx vite.config.ts && git -c user.name=daoyu -c user.email=daoyu@local commit -m "feat(popup): frameless popup window config + multi-page vite entry"
```

---

## Task 8: Popup Rust control (show at point, set payload, hide)

**Files:** Create `src-tauri/src/popup.rs`; modify lib.rs

- [ ] **Step 1: Write popup.rs.** Create `src-tauri/src/popup.rs`:

```rust
//! Show/move/hide the frameless popup window; push a payload (loading / result).
use tauri::{Manager, WebviewWindow, Emitter};

const POPUP: &str = "popup";

pub fn show_at(app: &tauri::AppHandle, x: i32, y: i32) -> Result<(), String> {
    let win: WebviewWindow = app.get_webview_window(POPUP).ok_or("no popup window")?;
    win.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y })).map_err(|e| e.to_string())?;
    win.emit("popup-state", Payload { status: "loading", text: "", engine: "" }).map_err(|e| e.to_string())?;
    win.show().map_err(|e| e.to_string())?;
    win.set_focus().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn result(app: &tauri::AppHandle, text: &str, engine: &str) -> Result<(), String> {
    let win: WebviewWindow = app.get_webview_window(POPUP).ok_or("no popup window")?;
    win.emit("popup-state", Payload { status: "result", text, engine }).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn error(app: &tauri::AppHandle, msg: &str) -> Result<(), String> {
    let win: WebviewWindow = app.get_webview_window(POPUP).ok_or("no popup window")?;
    win.emit("popup-state", Payload { status: "error", text: msg, engine: "" }).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn hide(app: &tauri::AppHandle) -> Result<(), String> {
    let win: WebviewWindow = app.get_webview_window(POPUP).ok_or("no popup window")?;
    win.hide().map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Clone, serde::Serialize)]
struct Payload<'a> { status: &'a str, text: &'a str, engine: &'a str }
```

> **Implementer note:** verify the `Emitter`/`emit` API and `get_webview_window` exist in the resolved Tauri 2 version (they do in current 2.x). `tauri::Position::Physical` + `PhysicalPosition` are correct. If the import paths differ, adjust minimally.

- [ ] **Step 2: Add module + check + commit.** `pub mod popup;` in lib.rs. `cargo check`. Then:
```bash
git add src-tauri/src/popup.rs src-tauri/src/lib.rs && git -c user.name=daoyu -c user.email=daoyu@local commit -m "feat(popup): show/move/hide + payload emit"
```

---

## Task 9: Popup SolidJS component (listens for state)

**Files:** Create `src/Popup.tsx`

- [ ] **Step 1: Write Popup.tsx.** Create `src/Popup.tsx`:

```tsx
import { createSignal, onMount } from "solid-js";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

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
    const win = (await import("@tauri-apps/api/window")).getCurrentWindow();
    win.onFocusChanged(({ payload: focused }) => { if (!focused) win.hide(); });
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
```

- [ ] **Step 2: Verify build.** `pnpm build`. Expected: success.

- [ ] **Step 3: Commit.**
```bash
git add src/Popup.tsx && git -c user.name=daoyu -c user.email=daoyu@local commit -m "feat(popup): SolidJS popup component (loading/result/error)"
```

---

## Task 10: Global-shortcut handler wiring it all together

**Files:** Modify `src-tauri/src/lib.rs`

- [ ] **Step 1: Define the Session + handler.** In `lib.rs`, add a shared session struct and the handler. Add imports and replace `run()`:

```rust
use std::sync::Arc;
use parking_lot::Mutex;
use tauri::{Manager, Emitter};

struct Session {
    client: reqwest::Client,
    keystore: keystore::Keystore,
    gen: concurrency::GenerationToken,
}

#[tauri::command]
async fn translate(req: TranslateRequest, engine: String, state: tauri::State<'_, Arc<Session>>) -> Result<TranslateResult, String> {
    // (keep the existing translate body, but take Arc<Session> instead of Arc<AppState>;
    //  read state.client / state.keystore)
    // ... existing logic ...
}
// likewise update set_key/delete_key/key_status to Arc<Session>
```

Then the hotkey handler, registered in `run()`:

```rust
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new()
            .with_shortcut("Alt+Space")
            .expect("shortcut")
            .with_handler(|app, _shortcut, event| {
                if event.state != tauri_plugin_global_shortcut::ShortcutState::Pressed { return; }
                let app = app.clone();
                // Run on a thread: capture must not block the shortcut loop, and
                // selection capture involves sleeps.
                tauri::async_runtime::spawn(async move {
                    let state: tauri::State<'_, Arc<Session>> = app.state();
                    let gen = state.gen.next();
                    // (a) capture cursor + selection under the selection mutex.
                    let (x, y) = { let _g = state.gen.selection_lock(); cursor::position() };
                    let captured = {
                        let _g = state.gen.selection_lock();
                        selection::capture_selection(800)
                    };
                    if !state.gen.is_latest(gen) { return; } // newer press superseded us
                    let text = match captured {
                        Ok(selection::Capture::Selected(t)) => t,
                        Ok(selection::Capture::NoSelection) => { let _ = popup::hide(&app); return; }
                        Err(e) => { let _ = popup::error(&app, &e); return; }
                    };
                    // (b) show popup loading at cursor.
                    let _ = popup::show_at(&app, x, y);
                    // (c) translate via Phase-1 service.
                    let preset = providers::presets().into_iter()
                        .find(|p| p.id == "openai") // default; real default-choice in 2b
                        .expect("default preset");
                    let input = service::TranslateInput { text: &text, from: "auto", to: "zh", options: Default::default() };
                    match service::translate(&state.client, &state.keystore, &preset, input).await {
                        Ok(out) => { if state.gen.is_latest(gen) { let _ = popup::result(&app, &out, &preset.id); } }
                        Err(e) => { if state.gen.is_latest(gen) { let _ = popup::error(&app, &e.to_string()); } }
                    }
                });
            })
            .build())
        .setup(|app| {
            // (existing keystore init + endpoint validation from Phase 1, but build Arc<Session>)
            let dir = app.path().app_local_data_dir().expect("dir");
            let keystore = keystore::Keystore::new(dir).expect("keystore");
            for p in providers::presets() { providers::validate_endpoint(&p.endpoint).expect("endpoint"); }
            let client = reqwest::Client::builder().redirect(reqwest::redirect::Policy::none()).build().expect("client");
            app.manage(Arc::new(Session { client, keystore, gen: concurrency::GenerationToken::new() }));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![translate, list_engines, set_key, delete_key, key_status])
        .run(tauri::generate_context!())
        .expect("run");
}
```

> **Implementer notes (resolve at execution):**
> - The exact `tauri_plugin_global_shortcut::Builder` API (`.with_shortcut` / `.with_handler` / `.build()` and the handler closure signature) may differ by the resolved 2.x version. Check the plugin's docs.rs and match its real API. The *intent* is: register `Alt+Space`, on Pressed run the handler. If the API takes a list of `(shortcut, handler)` pairs or uses `Manager::on_webview_event`-style registration, adapt — keep the same behavior.
> - The handler closure must clone `app` and move work to `tauri::async_runtime::spawn` because capture has sleeps and must not block the global-shortcut thread.
> - `app.state::<Arc<Session>>()` retrieves the managed session.
> - Remove the old `AppState` struct (replaced by `Session`); update `translate`/`set_key`/`delete_key`/`key_status` signatures to `tauri::State<'_, Arc<Session>>` and read `state.client`/`state.keystore`.

- [ ] **Step 2: cargo check + iterate.** Run `cargo check`. Resolve API mismatches in the global-shortcut builder until it compiles. If after reasonable effort the plugin API is materially different, report BLOCKED with the actual API you found.

- [ ] **Step 3: Run the full test suite (no regression).** `cargo test` → expect all prior tests still pass (the new modules' tests: concurrency + selection_engine).

- [ ] **Step 4: Commit.**
```bash
git add -A && git -c user.name=daoyu -c user.email=daoyu@local commit -m "feat(hotkey): wire global shortcut → capture → popup → translate (§B/§C/§D)"
```

---

## Task 11: Manual end-to-end verification

**Files:** none (manual)

- [ ] **Step 1: Run the dev app.** `cd /Users/daoyu/Code/projects/linguaray && pnpm tauri dev`. Window opens.
- [ ] **Step 2: Set a key** in the main window for `openai` (or start Ollama locally and switch the default in Task 10's preset id).
- [ ] **Step 3: Select text** in any other app (e.g. Safari, Notes), press **Alt+Space**. Expected: a frameless popup appears near the cursor showing the translation (or a classified error).
- [ ] **Step 4: Verify restore**: your clipboard content before the press should be back (best-effort; on macOS Accessibility must be granted — System Settings → Privacy → Accessibility → enable LinguaRay).
- [ ] **Step 5: Verify latest-wins**: press Alt+Space twice quickly with different selections; the popup should settle on the second, never show the first's stale result.
- [ ] **Step 6: Verify NoSelection**: press Alt+Space with nothing selected → popup hides (or doesn't appear), no spurious "translation" of old clipboard.

> This task produces no commit. If issues are found, file them as concrete follow-up tasks. Mark this task done once the loop works.

---

## Self-Review (run after writing; fix inline)

- **Spec coverage:** §B sentinel → Tasks 4-5. §C popup anchoring → Tasks 6-9. §D hotkey → Task 10. §concurrency generation-token → Task 2 + 10. §G error classification → Phase-1 `service` reused. macOS Accessibility permission → Task 11 step 4 (manual gating; the simulated keystroke needs it). **Gap:** the `clipboard.rs` macOS `sequence()` uses raw `objc` msg_send to NSPasteboard.changeCount — verify that API path works; if `objc` crate ergonomics fight, fall back to `cocoa` or an `NSPasteboard` wrapper. The test of `sequence()` is manual (hard to unit-test OS APIs) — acceptable, matches §I.
- **Placeholder scan:** Task 10 has several "adapt to the real plugin API" notes — these are *honest unknowns* about a third-party API, resolved at execution by reading docs. Task 4 step 2's RefCell rework is specified, not hand-waved. No "TODO/TBD".
- **Type consistency:** `Session` replaces `AppState` everywhere in Task 10. `selection::Capture` enum used in Task 5 and Task 10. `GenerationToken` method names (`next`/`is_latest`/`selection_lock`) consistent across Tasks 2 and 10.

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-07-30-phase2a-selection-translate-loop.md`. Two execution options:

**1. Subagent-Driven (recommended)** — fresh subagent per task, two-stage review, fast iteration. (Same as Phase 1.)

**2. Inline Execution** — batch with checkpoints.

**Which approach?**
