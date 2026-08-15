import { Button, FileButton, Group, Text } from "@mantine/core";
import { detectLocale } from "../../i18n";
import { OCR_COPY } from "./copy";
import type { OcrController } from "./controller";
import classes from "./ocr.module.css";

/** Pure presentational OCR overlay (region drag + toolbar + error). */
export function OcrOverlayView({ c }: { c: OcrController }) {
  const t = OCR_COPY[detectLocale()];
  const isToolbar = (target: EventTarget | null) =>
    !!(target as HTMLElement | null)?.closest?.("[data-ocr-toolbar]");

  return (
    <div
      className={classes.overlay}
      role="application"
      aria-label={t.overlayRole}
      data-testid="ocr-overlay"
      onMouseDown={(e) => {
        if (isToolbar(e.target)) return;
        c.startDrag(e.screenX, e.screenY);
      }}
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
      <Group
        gap="xs"
        p="xs"
        className={classes.toolbar}
        data-ocr-toolbar
        wrap="nowrap"
      >
        <FileButton
          accept="image/*"
          disabled={c.busy}
          onChange={(file) => file && c.runFile(file)}
        >
          {(props) => (
            <Button {...props} size="xs" variant="light" aria-label={t.openImage}>
              {t.openImage}
            </Button>
          )}
        </FileButton>
        <Button
          size="xs"
          variant="light"
          disabled={c.busy}
          onClick={c.runClipboard}
        >
          {t.clipboardImage}
        </Button>
        <Button size="xs" variant="subtle" disabled={c.busy} onClick={c.cancel}>
          {t.cancel}
        </Button>
        <Text size="xs" c="dimmed" role="status" className={classes.hint}>
          {c.busy ? t.recognizing : t.hint}
        </Text>
      </Group>

      {c.notice && (
        <Text c="red" size="sm" role="alert" className={classes.error} data-testid="ocr-error">
          {t.errorPrefix}: {c.notice}
        </Text>
      )}

      {c.rect && (
        <div
          className={classes.region}
          style={{
            left: `${c.rect.x - window.screenX}px`,
            top: `${c.rect.y - window.screenY}px`,
            width: `${c.rect.w}px`,
            height: `${c.rect.h}px`,
          }}
          data-testid="ocr-region"
        />
      )}
    </div>
  );
}

export default OcrOverlayView;
