import type { GetRef } from "antd";
import { Alert, Button, Typography } from "antd";
import { DeleteOutlined, SendOutlined, StarFilled, StarOutlined, TranslationOutlined } from "@ant-design/icons";
import { Bubble, Sender, Welcome } from "@ant-design/x";
import type { BubbleItemType } from "@ant-design/x";
import { t } from "../../app/i18n";
import { SurfaceFooter, SurfaceHeader, SurfaceLayout, XActionBar } from "../../ui/x";
import { engineLabel } from "./providerNames";
import type { InputController } from "./inputController";
import type { TranslationState } from "./types";

export function errorMessageFor(state: TranslationState): string | null {
  if (state.kind === "error") {
    return state.sub === "network"
      ? t("selection.error.network")
      : state.sub === "config-key"
        ? t("selection.error.config.key")
        : state.sub === "config-401"
          ? t("selection.error.config.auth")
          : state.message;
  }
  if (state.kind === "offline") return t("input.error.offline");
  if (state.kind === "no-permission") return t("selection.error.noPermission");
  if (state.kind === "no-provider") return t("selection.error.noProvider");
  if (state.kind === "keystore-corrupt") return t("selection.error.keystore");
  return null;
}

/** Ant Design X conversation surface backed by the existing InputController. */
export function InputPanelView({ c }: { c: InputController }) {
  const single = c.state.kind === "single-success" ? c.state : null;
  const multi = c.state.kind === "multi-success" || c.state.kind === "partial" ? c.state.results : null;
  const errorMessage = errorMessageFor(c.state);
  const showClear = c.hasResult || c.text.trim().length > 0;

  const favoriteActions = (key: string, translation: string) => {
    const favorited = c.favoritedKey === key;
    const label = favorited ? t("selection.action.favorited") : t("selection.action.favorite");
    return (
      <XActionBar
        actions={[{
          key: "favorite",
          label,
          icon: favorited ? <StarFilled aria-hidden /> : <StarOutlined aria-hidden />,
          active: favorited,
          onClick: () => void c.favorite(c.text, translation, key),
        }]}
      />
    );
  };

  const items: BubbleItemType[] = [];
  if (c.hasResult && c.text.trim()) {
    items.push({ key: "source", role: "user", placement: "end", content: c.text, variant: "filled" });
  }
  if (!c.idle) {
    items.push({ key: "loading", role: "ai", content: t("selection.loading"), loading: true, header: t("input.result.label") });
  }
  if (single) {
    items.push({
      key: "single",
      role: "ai",
      content: <div data-testid="input-result">{single.text}</div>,
      header: engineLabel(single.engine),
      footer: () => favoriteActions("__single__", single.text),
      variant: "outlined",
    });
  }
  multi?.forEach((result) => {
    items.push({
      key: result.uuid,
      role: "ai",
      content: <div data-testid="input-result">{result.ok ? result.text : result.errorText}</div>,
      header: engineLabel(result.engine),
      footer: result.ok && result.text ? () => favoriteActions(result.uuid, result.text!) : undefined,
      variant: "outlined",
      status: result.ok ? "success" : "error",
      className: result.ok ? undefined : "lr-x-bubble-error",
    });
  });

  const bindSenderRef = (ref: GetRef<typeof Sender> | null) => {
    c.textareaRef.current = (ref?.inputElement as HTMLTextAreaElement | undefined) ?? null;
    ref?.inputElement?.setAttribute("aria-label", t("input.title"));
  };

  return (
    <main className="lr-x-translation-window" data-testid="input-panel">
      <SurfaceLayout
        header={
          <SurfaceHeader draggable>
            <TranslationOutlined aria-hidden />
            <Typography.Text strong>{t("input.title")}</Typography.Text>
          </SurfaceHeader>
        }
        content={
          <div className="lr-x-conversation-content">
            {items.length > 0 ? (
              <Bubble.List items={items} autoScroll={false} />
            ) : (
              <Welcome icon={<TranslationOutlined aria-hidden />} title={t("input.result.label")} description={t("input.placeholder")} variant="borderless" />
            )}
            {errorMessage ? <Alert type="error" showIcon title={errorMessage} data-testid="input-error" /> : null}
          </div>
        }
        footer={
          <SurfaceFooter draggable>
            <Sender
              ref={bindSenderRef}
              rootClassName="lr-x-sender non-draggable-area"
              value={c.text}
              placeholder={t("input.placeholder")}
              loading={!c.idle}
              disabled={!c.idle}
              autoSize={{ minRows: 2, maxRows: 5 }}
              submitType="enter"
              onChange={c.setText}
              onSubmit={() => void c.translate()}
              prefix={
                <Button type="text" size="small" icon={<DeleteOutlined aria-hidden />} onClick={c.clear} disabled={!showClear}>
                  {t("input.action.clear")}
                </Button>
              }
              suffix={(_, { components }) => {
                const SendButton = components.SendButton;
                return (
                  <SendButton
                    type="primary"
                    shape="default"
                    size="small"
                    icon={<SendOutlined aria-hidden />}
                    disabled={!c.idle || !c.text.trim()}
                  >
                    {t("input.action.translate")}
                  </SendButton>
                );
              }}
            />
          </SurfaceFooter>
        }
      />
    </main>
  );
}

export default InputPanelView;
