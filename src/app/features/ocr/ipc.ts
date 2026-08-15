/** OCR overlay IPC + drag-rect helpers. */
import { invoke } from "../../../bridge/invoke";

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

export async function ocrRegionThenTranslate(rect: ScreenRect): Promise<void> {
  const result = await invoke<{ text: string }>("ocr_capture_region", captureArgs(rect));
  await translateOcrText(result.text);
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

export async function fileToBytes(file: File): Promise<Uint8Array> {
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
