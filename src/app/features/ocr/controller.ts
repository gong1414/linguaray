/**
 * OCR overlay controller — on-demand window that must not linger: every
 * terminal path (success, cancel, unrecoverable error) destroys it.
 * Recognition runs BEFORE destroying so failures stay visible with a retry.
 */
import { useCallback, useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "../../../bridge/window";
import {
  fileToBytes,
  ocrBytesThenTranslate,
  ocrClipboardThenTranslate,
  ocrRegionThenTranslate,
  type ScreenRect,
} from "./ipc";

export function useOcrController() {
  const [origin, setOrigin] = useState<{ x: number; y: number } | null>(null);
  const [rect, setRect] = useState<ScreenRect | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const busyRef = useRef(false);
  busyRef.current = busy;

  const endSession = useCallback(async () => {
    setOrigin(null);
    setRect(null);
    await getCurrentWindow().destroy();
  }, []);

  const cancel = useCallback(() => {
    if (!busyRef.current) void endSession();
  }, [endSession]);

  const runOcr = useCallback(
    async (fn: () => Promise<void>) => {
      if (busyRef.current) return;
      busyRef.current = true;
      setBusy(true);
      setNotice(null);
      try {
        await fn();
        await endSession();
      } catch (e) {
        // Keep the overlay open: the error is actionable here (retry/cancel).
        setNotice(String(e));
      } finally {
        // Reset even on success — if destroy() failed on a dying window
        // bridge, the lingering overlay must stay actionable.
        busyRef.current = false;
        setBusy(false);
      }
    },
    [endSession],
  );

  const startDrag = useCallback((x: number, y: number) => {
    setOrigin({ x, y });
    setRect({ x, y, w: 0, h: 0 });
  }, []);

  const moveDrag = useCallback(
    (x: number, y: number) => {
      setOrigin((o) => {
        if (!o) return o;
        setRect({
          x: Math.min(o.x, x),
          y: Math.min(o.y, y),
          w: Math.abs(x - o.x),
          h: Math.abs(y - o.y),
        });
        return o;
      });
    },
    [],
  );

  /** Returns the completed rect when it is a real selection (>=4px). */
  const finishDrag = useCallback(
    (x: number, y: number): ScreenRect | null => {
      const o = origin;
      setOrigin(null);
      if (!o) return null;
      const r: ScreenRect = {
        x: Math.min(o.x, x),
        y: Math.min(o.y, y),
        w: Math.abs(x - o.x),
        h: Math.abs(y - o.y),
      };
      setRect(null);
      return r.w >= 4 && r.h >= 4 ? r : null;
    },
    [origin],
  );

  const runRegion = useCallback(
    (r: ScreenRect) => void runOcr(() => ocrRegionThenTranslate(r)),
    [runOcr],
  );

  const runFile = useCallback(
    (file: File) =>
      void runOcr(async () => {
        await ocrBytesThenTranslate(await fileToBytes(file));
      }),
    [runOcr],
  );

  const runClipboard = useCallback(
    () => void runOcr(() => ocrClipboardThenTranslate()),
    [runOcr],
  );

  useEffect(() => {
    // The window is built hidden (ocr_capture) — show only once this DOM
    // exists so a cold WebView never flashes gray. The catch keeps browser
    // contexts (Storybook) quiet.
    void getCurrentWindow()
      .show()
      .catch(() => {});
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        e.preventDefault();
        cancel();
      }
    };
    const onContextMenu = (e: MouseEvent) => {
      e.preventDefault();
      cancel();
    };
    window.addEventListener("keydown", onKey);
    window.addEventListener("contextmenu", onContextMenu);
    return () => {
      window.removeEventListener("keydown", onKey);
      window.removeEventListener("contextmenu", onContextMenu);
    };
  }, [cancel]);

  return {
    rect,
    notice,
    busy,
    startDrag,
    moveDrag,
    finishDrag,
    runRegion,
    runFile,
    runClipboard,
    cancel,
  };
}

export type OcrController = ReturnType<typeof useOcrController>;
