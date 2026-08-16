/** OCR overlay IPC + drag-rect helpers. */
import { commands } from "../../bridge/invoke";
import { translateSelection } from "../translation/popup-ipc";

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

/** Tray/shortcut OCR entry: opens the capture overlay (source labels the trigger). */
export function startOcrCapture(source: "tray" | "shortcut"): Promise<void> {
  return commands.ocrCapture(source).then(() => undefined);
}

export async function translateOcrText(text: string): Promise<void> {
  const trimmed = text.trim();
  if (!trimmed) return;
  await translateSelection(trimmed);
}

export async function ocrRegionThenTranslate(rect: ScreenRect): Promise<void> {
  const args = captureArgs(rect);
  const result = await commands.ocrCaptureRegion(args.x, args.y, args.width, args.height);
  await translateOcrText(result.text);
}

export async function ocrBytesThenTranslate(bytes: Uint8Array): Promise<void> {
  const result = await commands.ocrRecognizeBytes(Array.from(bytes));
  await translateOcrText(result.text);
}

export async function ocrPathThenTranslate(path: string): Promise<void> {
  const result = await commands.ocrFromImage(path);
  await translateOcrText(result.text);
}

export async function ocrClipboardThenTranslate(): Promise<void> {
  const result = await commands.ocrFromClipboard();
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
