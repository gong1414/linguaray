import { useRef } from "react";
import { Button, MessageBar, MessageBarBody, Text, makeStyles, tokens } from "@fluentui/react-components";
import { ClipboardRegular, DismissRegular, ImageRegular } from "@fluentui/react-icons";
import { detectLocale } from "../../app/i18n";
import { OCR_COPY } from "./copy";
import type { OcrController } from "./controller";

const useStyles = makeStyles({
  overlay: {
    position: "fixed",
    inset: 0,
    backgroundColor: "rgba(0, 0, 0, 0.25)",
    cursor: "crosshair",
  },
  toolbar: {
    position: "absolute",
    top: tokens.spacingVerticalS,
    left: "50%",
    transform: "translateX(-50%)",
    display: "flex",
    alignItems: "center",
    gap: tokens.spacingHorizontalXS,
    padding: tokens.spacingVerticalXS,
    backgroundColor: tokens.colorNeutralBackground1,
    borderRadius: tokens.borderRadiusMedium,
    boxShadow: tokens.shadow8,
  },
  hint: { whiteSpace: "nowrap", color: tokens.colorNeutralForeground2 },
  error: {
    position: "absolute",
    top: "64px",
    left: "50%",
    transform: "translateX(-50%)",
    maxWidth: "min(32rem, calc(100vw - 2rem))",
  },
  region: {
    position: "absolute",
    border: `${tokens.strokeWidthThick} solid ${tokens.colorBrandStroke1}`,
    backgroundColor: "rgba(15, 108, 189, 0.15)",
    pointerEvents: "none",
  },
  hiddenInput: { display: "none" },
});

/** Pure presentational OCR overlay (region drag + toolbar + error). */
export function OcrOverlayView({ c }: { c: OcrController }) {
  const t = OCR_COPY[detectLocale()];
  const styles = useStyles();
  const fileInput = useRef<HTMLInputElement>(null);
  const isToolbar = (target: EventTarget | null) => !!(target as HTMLElement | null)?.closest?.("[data-ocr-toolbar]");

  return (
    <div
      className={styles.overlay}
      role="application"
      aria-label={t.overlayRole}
      data-testid="ocr-overlay"
      onMouseDown={(e) => { if (!isToolbar(e.target)) c.startDrag(e.screenX, e.screenY); }}
      onMouseMove={(e) => c.moveDrag(e.screenX, e.screenY)}
      onMouseUp={(e) => {
        if (isToolbar(e.target)) return;
        const rect = c.finishDrag(e.screenX, e.screenY);
        if (rect) c.runRegion(rect);
      }}
      onDragOver={(e) => e.preventDefault()}
      onDrop={(e) => {
        e.preventDefault();
        const file = e.dataTransfer?.files?.[0];
        if (file) c.runFile(file);
      }}
    >
      <div className={styles.toolbar} data-ocr-toolbar>
        <input
          ref={fileInput}
          className={styles.hiddenInput}
          hidden
          type="file"
          accept="image/*"
          aria-label={t.openImage}
          onChange={(e) => { const file = e.currentTarget.files?.[0]; if (file) c.runFile(file); }}
        />
        <Button appearance="secondary" size="small" icon={<ImageRegular />} disabled={c.busy} onClick={() => fileInput.current?.click()}>{t.openImage}</Button>
        <Button appearance="secondary" size="small" icon={<ClipboardRegular />} disabled={c.busy} onClick={c.runClipboard}>{t.clipboardImage}</Button>
        <Button appearance="subtle" size="small" icon={<DismissRegular />} disabled={c.busy} onClick={c.cancel}>{t.cancel}</Button>
        <Text size={200} role="status" className={styles.hint}>{c.busy ? t.recognizing : t.hint}</Text>
      </div>

      {c.notice && <MessageBar intent="error" className={styles.error} data-testid="ocr-error"><MessageBarBody>{t.errorPrefix}: {c.notice}</MessageBarBody></MessageBar>}

      {c.rect && (
        <div
          className={styles.region}
          style={{ left: `${c.rect.x - window.screenX}px`, top: `${c.rect.y - window.screenY}px`, width: `${c.rect.w}px`, height: `${c.rect.h}px` }}
          data-testid="ocr-region"
        />
      )}
    </div>
  );
}

export default OcrOverlayView;
