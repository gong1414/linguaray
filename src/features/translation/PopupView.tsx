import { useEffect, useRef } from "react";
import { Button, Flex, Tag, Typography } from "antd";
import { RedoOutlined, TranslationOutlined } from "@ant-design/icons";
import { t } from "../../app/i18n";
import {
  TranslationResultSurface,
  headlineForTranslation,
} from "./TranslationResultSurface";
import type { PopupController } from "./popupController";
import type { TranslationState } from "./types";

export function headlineFor(state: TranslationState): string {
  return headlineForTranslation(state);
}

/** Do-first Hybrid UI quick bar rendered by the shared A2UI result surface. */
export function PopupView({ c }: { c: PopupController }) {
  const shellRef = useRef<HTMLElement | null>(null);
  useEffect(() => shellRef.current?.focus(), []);
  const errorTestId = c.state.kind === "keystore-corrupt" ? "popup-keystore" : "popup-error";
  const showHeaderRetry = c.hasSource && (
    c.state.kind === "loading"
    || c.state.kind === "single-success"
    || c.state.kind === "multi-success"
    || c.state.kind === "partial"
  );

  return (
    <section
      ref={shellRef}
      aria-label={headlineFor(c.state)}
      aria-busy={c.state.kind === "loading" || undefined}
      data-testid="popup-shell"
      tabIndex={-1}
      onKeyDown={(event) => {
        if (event.key === "Escape") {
          event.preventDefault();
          c.dismiss();
        }
      }}
      className="lr-hui-quickbar"
    >
      <header className="lr-hui-quickbar-header" data-tauri-drag-region>
        <Flex align="center" gap="small">
          <TranslationOutlined aria-hidden />
          <Typography.Text strong>{headlineFor(c.state)}</Typography.Text>
          {c.pinned ? <Tag color="processing">{t("selection.action.pin")}</Tag> : null}
        </Flex>
        {showHeaderRetry ? (
          <Button
            className="non-draggable-area"
            type="text"
            size="small"
            icon={<RedoOutlined aria-hidden />}
            onClick={c.retrySelection}
          >
            {t("selection.action.retry")}
          </Button>
        ) : null}
      </header>

      <div
        className="lr-hui-quickbar-body"
        data-testid={c.state.kind === "loading" ? "popup-loading" : undefined}
      >
        <TranslationResultSurface
          state={c.state}
          source={c.lastSource}
          testId="popup-card"
          errorTestId={errorTestId}
          surfaceId="selection-result"
          engineLabel={c.engineLabel}
          pinned={c.pinned}
          actions={{
            copy: c.copyText,
            favorite: (source, text) => c.favoriteText(source || c.lastSource || text, text),
            speak: c.speak,
            stopSpeaking: c.stopSpeaking,
            togglePin: () => c.pinned ? c.unpin() : c.pin(),
            retry: c.retrySelection,
            openSettings: c.openSettings,
          }}
        />
      </div>
    </section>
  );
}

export default PopupView;
