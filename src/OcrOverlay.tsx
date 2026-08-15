import { createSignal, onCleanup, onMount, Show, type Component } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { detectLocale } from "./i18n";
import { OCR_COPY } from "./ocr-copy";
import "./OcrOverlay.css";

export type ScreenRect = { x: number; y: number; w: number; h: number };

/** `screencapture -R` uses logical screen points, not device pixels. */
export function captureArgs(rect: ScreenRect): {
  x: number;
  y: number;
  width: number;
  height: number;
} {
  return {
    x: Math.round(rect.x),
    y: Math.round(rect.y),
    width: Math.round(rect.w),
    height: Math.round(rect.h),
  };
}

export async function translateOcrText(text: string): Promise<void> {
  const trimmed = text.trim();
  if (!trimmed) return;
  await invoke("translate_selection_ipc", { text: trimmed });
}

export async function ocrBytesThenTranslate(bytes: Uint8Array): Promise<void> {
  const result = await invoke<{ text: string }>("ocr_recognize_bytes", {
    bytes: Array.from(bytes),
  });
  await translateOcrText(result.text);
}

export async function ocrPathThenTranslate(path: string): Promise<void> {
  const result = await invoke<{ text: string }>("ocr_from_image", { path });
  await translateOcrText(result.text);
}

export async function ocrClipboardThenTranslate(): Promise<void> {
  const result = await invoke<{ text: string }>("ocr_from_clipboard");
  await translateOcrText(result.text);
}

async function fileToBytes(file: File): Promise<Uint8Array> {
  if (typeof file.arrayBuffer === "function") {
    return new Uint8Array(await file.arrayBuffer());
  }
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(new Uint8Array(reader.result as ArrayBuffer));
    reader.onerror = () => reject(reader.error ?? new Error("read file"));
    reader.readAsArrayBuffer(file);
  });
}

const OcrOverlay: Component = () => {
  const t = OCR_COPY[detectLocale()];
  const [origin, setOrigin] = createSignal<{ x: number; y: number } | null>(null);
  const [rect, setRect] = createSignal<ScreenRect | null>(null);
  const [notice, setNotice] = createSignal<string | null>(null);
  const [busy, setBusy] = createSignal(false);

  const resetDrag = () => {
    setOrigin(null);
    setRect(null);
  };

  /**
   * This window is created ON DEMAND by `ocr_capture` and must not linger:
   * every terminal path (success, cancel, unrecoverable error) destroys it.
   * Recognition runs BEFORE destroying so failures stay visible here with a
   * retry — hiding first would write the error into a window nobody sees.
   */
  const endSession = async () => {
    resetDrag();
    await getCurrentWindow().destroy();
  };

  const cancel = () => {
    if (!busy()) void endSession();
  };

  const runOcr = async (fn: () => Promise<void>) => {
    if (busy()) return;
    setBusy(true);
    setNotice(null);
    try {
      await fn();
      await endSession();
    } catch (e) {
      // Keep the overlay open: the error is actionable here (retry / cancel).
      setNotice(String(e));
    } finally {
      // Reset even on success — if destroy() itself failed on a dying
      // window bridge, the lingering overlay must stay actionable, not
      // dead-lock on busy.
      setBusy(false);
    }
  };

  const onDown = (e: MouseEvent) => {
    if ((e.target as HTMLElement).closest("[data-ocr-toolbar]")) return;
    setOrigin({ x: e.screenX, y: e.screenY });
    setRect({ x: e.screenX, y: e.screenY, w: 0, h: 0 });
  };
  const onMove = (e: MouseEvent) => {
    const o = origin();
    if (!o) return;
    const x = Math.min(o.x, e.screenX);
    const y = Math.min(o.y, e.screenY);
    setRect({ x, y, w: Math.abs(e.screenX - o.x), h: Math.abs(e.screenY - o.y) });
  };
  const onUp = (e: MouseEvent) => {
    if ((e.target as HTMLElement).closest("[data-ocr-toolbar]")) return;
    const dragging = origin();
    const r = rect();
    resetDrag();
    if (!dragging || !r || r.w < 4 || r.h < 4) return;
    void runOcr(async () => {
      const result = await invoke<{ text: string }>("ocr_capture_region", captureArgs(r));
      await translateOcrText(result.text);
    });
  };

  const runBytes = (file: File) =>
    void runOcr(async () => {
      await ocrBytesThenTranslate(await fileToBytes(file));
    });

  const onDrop = (e: DragEvent) => {
    e.preventDefault();
    const file = e.dataTransfer?.files?.[0];
    if (file) runBytes(file);
  };

  const onFile = (e: Event) => {
    const input = e.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    input.value = "";
    if (file) runBytes(file);
  };

  const onClipboard = () =>
    void runOcr(async () => {
      await ocrClipboardThenTranslate();
    });

  onMount(() => {
    // The window is built hidden (ocr_capture) — show only once this DOM
    // exists, so a cold WebView never flashes gray before content. The catch
    // keeps the ui-lab page (plain browser, no window bridge) quiet.
    void getCurrentWindow()
      .show()
      .catch(() => {});
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        cancel();
      }
    };
    window.addEventListener("keydown", onKey);
    const onContextMenu = (e: MouseEvent) => {
      e.preventDefault();
      cancel();
    };
    window.addEventListener("contextmenu", onContextMenu);
    onCleanup(() => {
      window.removeEventListener("keydown", onKey);
      window.removeEventListener("contextmenu", onContextMenu);
    });
  });

  const r = () => rect();
  return (
    <div
      class="ocr-overlay"
      role="application"
      aria-label={t.overlayRole}
      onMouseDown={onDown}
      onMouseMove={onMove}
      onMouseUp={(e) => void onUp(e)}
      onDragOver={(e) => e.preventDefault()}
      onDrop={onDrop}
    >
      <div class="ocr-overlay__toolbar" data-ocr-toolbar>
        <input
          class="ocr-overlay__action"
          type="file"
          accept="image/*"
          aria-label={t.openImage}
          disabled={busy()}
          onChange={onFile}
        />
        <button
          type="button"
          class="ocr-overlay__action"
          disabled={busy()}
          onClick={() => onClipboard()}
        >
          {t.clipboardImage}
        </button>
        <button type="button" class="ocr-overlay__action" disabled={busy()} onClick={cancel}>
          {t.cancel}
        </button>
        <span class="ocr-overlay__hint" role="status">
          {busy() ? t.recognizing : t.hint}
        </span>
      </div>
      <Show when={notice()}>
        <p class="ocr-overlay__error" role="alert">
          {t.errorPrefix}: {notice()}
        </p>
      </Show>
      <Show when={r()}>
        {(box) => (
          <div
            class="ocr-overlay__region"
            style={{
              left: `${box().x - window.screenX}px`,
              top: `${box().y - window.screenY}px`,
              width: `${box().w}px`,
              height: `${box().h}px`,
            }}
          />
        )}
      </Show>
    </div>
  );
};

export default OcrOverlay;
