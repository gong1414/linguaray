import { useEffect, useRef, useState } from "react";
import { Button, Flex, Result, Spin, Typography } from "antd";
import {
  CheckOutlined,
  CopyOutlined,
  PushpinFilled,
  PushpinOutlined,
  RedoOutlined,
  SettingOutlined,
  SoundFilled,
  SoundOutlined,
  StarFilled,
  StarOutlined,
  TranslationOutlined,
} from "@ant-design/icons";
import { Bubble } from "@ant-design/x";
import type { BubbleItemType } from "@ant-design/x";
import { t } from "../../app/i18n";
import { SurfaceFooter, SurfaceHeader, SurfaceLayout, XActionBar } from "../../ui/x";
import type { PopupController } from "./popupController";
import type { TranslationState } from "./types";

export function headlineFor(state: TranslationState): string {
  switch (state.kind) {
    case "loading": return t("selection.loading");
    case "single-success":
    case "multi-success":
    case "partial": return t("selection.multi.title");
    case "error":
      return state.sub === "network" ? t("selection.error.network")
        : state.sub === "config-key" ? t("selection.error.config.key")
          : state.sub === "config-401" ? t("selection.error.config.auth") : state.message;
    case "offline": return t("selection.error.offline");
    case "no-selection": return t("selection.error.noSelection");
    case "no-permission": return t("selection.error.noPermission");
    case "no-provider": return t("selection.error.noProvider");
    case "keystore-corrupt": return t("selection.error.keystore");
  }
}

const COPIED_FEEDBACK_MS = 1200;

function ResultActions({
  c,
  text,
  copied,
  favorited,
  speaking,
  onCopied,
  onFavorited,
  onSpeaking,
}: {
  c: PopupController;
  text: string;
  copied: boolean;
  favorited: boolean;
  speaking: boolean;
  onCopied: () => void;
  onFavorited: () => void;
  onSpeaking: (on: boolean) => void;
}) {
  const copyLabel = copied ? t("selection.action.copied") : t("selection.action.copy");
  const speakLabel = speaking ? t("selection.action.stop") : t("selection.action.speak");
  const pinLabel = c.pinned ? t("selection.action.unpin") : t("selection.action.pin");
  const favoriteLabel = favorited ? t("selection.action.favorited") : t("selection.action.favorite");

  return (
    <XActionBar
      actions={[
        {
          key: "copy",
          label: copyLabel,
          icon: copied ? <CheckOutlined aria-hidden /> : <CopyOutlined aria-hidden />,
          active: copied,
          onClick: () => void c.copyText(text).then(onCopied).catch(() => {}),
        },
        {
          key: "speak",
          label: speakLabel,
          icon: speaking ? <SoundFilled aria-hidden /> : <SoundOutlined aria-hidden />,
          active: speaking,
          onClick: () => {
            if (speaking) void c.stopSpeaking().finally(() => onSpeaking(false));
            else void c.speak(text).then(() => onSpeaking(true)).catch(() => onSpeaking(false));
          },
        },
        {
          key: "pin",
          label: pinLabel,
          icon: c.pinned ? <PushpinFilled aria-hidden /> : <PushpinOutlined aria-hidden />,
          active: c.pinned,
          onClick: () => c.pinned ? c.unpin() : c.pin(),
        },
        {
          key: "favorite",
          label: favoriteLabel,
          icon: favorited ? <StarFilled aria-hidden /> : <StarOutlined aria-hidden />,
          active: favorited,
          onClick: () => void c.favoriteText(c.lastSource || text, text).then(onFavorited).catch(() => {}),
        },
      ]}
    />
  );
}

/** Compact Ant Design X result surface backed by PopupController. */
export function PopupView({ c }: { c: PopupController }) {
  const single = c.state.kind === "single-success" ? c.state : null;
  const multi = c.state.kind === "multi-success" || c.state.kind === "partial" ? c.state.results : null;
  const errorState = c.state.kind === "error" ? c.state : null;
  const isError = c.state.kind === "error" || c.state.kind === "offline" || c.state.kind === "no-selection" || c.state.kind === "no-permission" || c.state.kind === "no-provider" || c.state.kind === "keystore-corrupt";
  const [copiedKey, setCopiedKey] = useState<string | null>(null);
  const [favoritedKey, setFavoritedKey] = useState<string | null>(null);
  const [speakingKey, setSpeakingKey] = useState<string | null>(null);
  const copiedTimer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  const shellRef = useRef<HTMLElement | null>(null);
  useEffect(() => () => copiedTimer.current && clearTimeout(copiedTimer.current), []);
  useEffect(() => shellRef.current?.focus(), []);

  const markCopied = (key: string) => {
    setCopiedKey(key);
    if (copiedTimer.current) clearTimeout(copiedTimer.current);
    copiedTimer.current = setTimeout(() => setCopiedKey(null), COPIED_FEEDBACK_MS);
  };

  const resultActions = (key: string, text: string) => (
    <ResultActions
      c={c}
      text={text}
      copied={copiedKey === key}
      favorited={favoritedKey === key}
      speaking={speakingKey === key}
      onCopied={() => markCopied(key)}
      onFavorited={() => setFavoritedKey(key)}
      onSpeaking={(on) => setSpeakingKey(on ? key : null)}
    />
  );

  const items: BubbleItemType[] = [];
  if (single) {
    items.push({
      key: "single",
      role: "ai",
      content: <div data-testid="popup-card">{single.text}</div>,
      header: c.engineLabel(single.engine),
      footer: () => resultActions("__single__", single.text),
      variant: "outlined",
    });
  }
  multi?.forEach((result) => {
    items.push({
      key: result.uuid,
      role: "ai",
      content: <div data-testid="popup-card">{result.ok ? result.text : result.errorText}</div>,
      header: c.engineLabel(result.engine),
      footer: result.ok && result.text ? () => resultActions(result.uuid, result.text!) : undefined,
      variant: "outlined",
      status: result.ok ? "success" : "error",
      className: result.ok ? undefined : "lr-x-bubble-error",
    });
  });

  const errorActions = (
    <Flex gap="small" wrap justify="center">
      {errorState?.sub === "network" && c.hasSource ? <Button icon={<RedoOutlined aria-hidden />} onClick={c.retrySelection}>{t("selection.action.retry")}</Button> : null}
      {(errorState?.sub === "config-key" || errorState?.sub === "config-401" || c.state.kind === "no-provider") ? <Button type="primary" icon={<SettingOutlined aria-hidden />} onClick={() => void c.openSettings("provider-center")}>{t("selection.action.openSettings")}</Button> : null}
      {c.state.kind === "keystore-corrupt" ? <Button type="primary" icon={<SettingOutlined aria-hidden />} onClick={() => void c.openSettings("keystore-recovery")}>{t("selection.action.recovery")}</Button> : null}
    </Flex>
  );

  const content = c.state.kind === "loading" ? (
    <div className="lr-x-popup-state" data-testid="popup-loading">
      <Spin size="small" />
      <Typography.Text>{t("selection.loading")}</Typography.Text>
      {c.hasSource ? <Button type="text" size="small" icon={<RedoOutlined aria-hidden />} onClick={c.retrySelection}>{t("selection.action.retry")}</Button> : null}
    </div>
  ) : isError ? (
    <div data-testid={c.state.kind === "keystore-corrupt" ? "popup-keystore" : "popup-error"}>
      <Result status="error" title={headlineFor(c.state)} extra={errorActions} />
    </div>
  ) : (
    <div className="lr-x-popup-results">
      <Bubble.List items={items} autoScroll={false} />
    </div>
  );

  return (
    <section
      ref={shellRef}
      aria-label={headlineFor(c.state)}
      aria-busy={c.state.kind === "loading" || undefined}
      data-testid="popup-shell"
      tabIndex={-1}
      onKeyDown={(event) => { if (event.key === "Escape") { event.preventDefault(); c.dismiss(); } }}
      className="lr-x-popup-shell"
    >
      <SurfaceLayout
        transparent
        header={c.state.kind === "loading" ? undefined : (
          <SurfaceHeader draggable>
            <TranslationOutlined aria-hidden />
            <Typography.Text strong>{headlineFor(c.state)}</Typography.Text>
          </SurfaceHeader>
        )}
        content={content}
        footer={(single || multi) && c.hasSource ? (
          <SurfaceFooter draggable>
            <span />
            <Button className="non-draggable-area" type="text" size="small" icon={<RedoOutlined aria-hidden />} onClick={c.retrySelection}>{t("selection.action.retry")}</Button>
          </SurfaceFooter>
        ) : undefined}
      />
    </section>
  );
}

export default PopupView;
