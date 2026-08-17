import { useRef } from "react";
import { Alert, Button, Typography } from "antd";
import { CloseOutlined, CopyOutlined, FileImageOutlined } from "@ant-design/icons";
import { detectLocale } from "../../app/i18n";
import { OCR_COPY } from "./copy";
import type { OcrController } from "./controller";

/** Pure presentational Ant Design OCR overlay. */
export function OcrOverlayView({ c }: { c: OcrController }) {
  const t = OCR_COPY[detectLocale()];
  const fileInput = useRef<HTMLInputElement>(null);
  const isToolbar = (target: EventTarget | null) => !!(target as HTMLElement | null)?.closest?.("[data-ocr-toolbar]");

  return (
    <div
      className="lr-ocr-overlay"
      role="application"
      aria-label={t.overlayRole}
      data-testid="ocr-overlay"
      onMouseDown={(e) => { if (!isToolbar(e.target)) c.startDrag(e.screenX, e.screenY); }}
      onMouseMove={(e) => c.moveDrag(e.screenX, e.screenY)}
      onMouseUp={(e) => { if (isToolbar(e.target)) return; const rect = c.finishDrag(e.screenX, e.screenY); if (rect) c.runRegion(rect); }}
      onDragOver={(e) => e.preventDefault()}
      onDrop={(e) => { e.preventDefault(); const file = e.dataTransfer?.files?.[0]; if (file) c.runFile(file); }}
    >
      <div className="lr-ocr-toolbar" data-ocr-toolbar>
        <input ref={fileInput} hidden type="file" accept="image/*" aria-label={t.openImage} onChange={(e) => { const file = e.currentTarget.files?.[0]; if (file) c.runFile(file); }} />
        <Button size="small" icon={<FileImageOutlined aria-hidden />} disabled={c.busy} onClick={() => fileInput.current?.click()}>{t.openImage}</Button>
        <Button size="small" icon={<CopyOutlined aria-hidden />} disabled={c.busy} onClick={c.runClipboard}>{t.clipboardImage}</Button>
        <Button type="text" size="small" icon={<CloseOutlined aria-hidden />} disabled={c.busy} onClick={c.cancel}>{t.cancel}</Button>
        <Typography.Text type="secondary" role="status" className="lr-ocr-hint">{c.busy ? t.recognizing : t.hint}</Typography.Text>
      </div>
      {c.notice ? <Alert type="error" showIcon className="lr-ocr-error" title={`${t.errorPrefix}: ${c.notice}`} data-testid="ocr-error" /> : null}
      {c.rect ? <div className="lr-ocr-region" style={{ left: `${c.rect.x - window.screenX}px`, top: `${c.rect.y - window.screenY}px`, width: `${c.rect.w}px`, height: `${c.rect.h}px` }} data-testid="ocr-region" /> : null}
    </div>
  );
}

export default OcrOverlayView;
