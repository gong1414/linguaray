import { createSignal, type Component } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";

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
  const [origin, setOrigin] = createSignal<{ x: number; y: number } | null>(null);
  const [rect, setRect] = createSignal<ScreenRect | null>(null);
  const [notice, setNotice] = createSignal("");

  const resetDrag = () => {
    setOrigin(null);
    setRect(null);
  };

  const hide = async () => {
    resetDrag();
    await getCurrentWindow().hide();
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
  const onUp = async (e: MouseEvent) => {
    if ((e.target as HTMLElement).closest("[data-ocr-toolbar]")) return;
    const dragging = origin();
    const r = rect();
    resetDrag();
    if (!dragging || !r || r.w < 4 || r.h < 4) return;
    await hide();
    try {
      const result = await invoke<{ text: string }>("ocr_capture_region", captureArgs(r));
      await translateOcrText(result.text);
    } catch (err) {
      setNotice(String(err));
    }
  };

  const runBytes = async (file: File) => {
    try {
      await ocrBytesThenTranslate(await fileToBytes(file));
      await hide();
    } catch (e) {
      setNotice(String(e));
    }
  };

  const onDrop = (e: DragEvent) => {
    e.preventDefault();
    const file = e.dataTransfer?.files?.[0];
    if (file) void runBytes(file);
  };

  const onFile = (e: Event) => {
    const input = e.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    input.value = "";
    if (file) void runBytes(file);
  };

  const onClipboard = async () => {
    try {
      await ocrClipboardThenTranslate();
      await hide();
    } catch (e) {
      setNotice(String(e));
    }
  };

  const r = () => rect();
  return (
    <div
      style={{
        width: "100%",
        height: "100%",
        cursor: "crosshair",
        background: "rgba(0,0,0,0.15)",
      }}
      onMouseDown={onDown}
      onMouseMove={onMove}
      onMouseUp={(e) => void onUp(e)}
      onDragOver={(e) => e.preventDefault()}
      onDrop={onDrop}
    >
      <div
        data-ocr-toolbar
        style={{
          position: "fixed",
          top: "16px",
          left: "50%",
          transform: "translateX(-50%)",
          display: "flex",
          gap: "8px",
          padding: "8px 12px",
          background: "rgba(15,23,42,0.9)",
          color: "#fff",
          "border-radius": "8px",
          "font-family": "system-ui",
          "font-size": "13px",
          cursor: "default",
        }}
      >
        <label>
          Open image
          <input type="file" accept="image/*" hidden onChange={onFile} />
        </label>
        <button type="button" onClick={() => void onClipboard()}>
          Clipboard image
        </button>
        <span>Draw a region, drop a file, or paste</span>
      </div>
      {notice() && (
        <p role="alert" style={{ position: "fixed", bottom: "16px", left: "16px", color: "#fecaca" }}>
          {notice()}
        </p>
      )}
      {r() && (
        <div
          style={{
            position: "fixed",
            left: `${r()!.x - window.screenX}px`,
            top: `${r()!.y - window.screenY}px`,
            width: `${r()!.w}px`,
            height: `${r()!.h}px`,
            border: "1px solid #38bdf8",
            background: "rgba(56,189,248,0.15)",
          }}
        />
      )}
    </div>
  );
};

export default OcrOverlay;
