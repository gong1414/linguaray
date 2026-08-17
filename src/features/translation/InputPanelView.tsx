import {
  Button,
  MessageBar,
  MessageBarBody,
  ProgressBar,
  Text,
  Textarea,
  Tooltip,
  tokens,
} from "@fluentui/react-components";
import { DeleteRegular, SendRegular, StarFilled, StarRegular } from "@fluentui/react-icons";
import { t } from "../../app/i18n";
import { BaseLayout, Footer, Header, SearchResultItem, SearchResultList } from "../../ui/ueli";
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

/**
 * LinguaRay adapter for Ueli's DeeplTranslator renderer. Ueli owns the
 * header/content/footer and two-pane translator structure; this view only
 * maps InputController state to those slots.
 */
export function InputPanelView({ c }: { c: InputController }) {
  const single = c.state.kind === "single-success" ? c.state : null;
  const multi = c.state.kind === "multi-success" || c.state.kind === "partial" ? c.state.results : null;
  const errorMessage = errorMessageFor(c.state);
  const showClear = c.hasResult || c.text.trim().length > 0;

  const favoriteButton = (key: string, translation: string) => {
    const favorited = c.favoritedKey === key;
    const label = favorited ? t("selection.action.favorited") : t("selection.action.favorite");
    return (
      <Tooltip content={label} relationship="label">
        <Button
          appearance={favorited ? "primary" : "subtle"}
          size="small"
          icon={favorited ? <StarFilled /> : <StarRegular />}
          aria-label={label}
          onClick={() => void c.favorite(c.text, translation, key)}
        />
      </Tooltip>
    );
  };

  const resultContent = (
    <div style={{ display: "flex", flexDirection: "column", gap: 10, minHeight: "100%" }}>
      {!c.hasResult && !errorMessage ? (
        <div
          style={{
            minHeight: 160,
            height: "100%",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            color: tokens.colorNeutralForeground3,
          }}
        >
          <Text size={300}>{t("input.result.label")}</Text>
        </div>
      ) : null}

      {single ? (
        <SearchResultList>
          <div data-testid="input-result">
            <SearchResultItem
              name={engineLabel(single.engine)}
              actions={favoriteButton("__single__", single.text)}
            >
              <Text style={{ whiteSpace: "pre-wrap", userSelect: "text", marginTop: 6 }}>{single.text}</Text>
            </SearchResultItem>
          </div>
        </SearchResultList>
      ) : null}

      {multi ? (
        <SearchResultList>
          {multi.map((result) => (
            <div key={result.uuid} data-testid="input-result">
              <SearchResultItem
                name={engineLabel(result.engine)}
                actions={result.ok && result.text ? favoriteButton(result.uuid, result.text) : undefined}
              >
                <Text
                  style={{
                    whiteSpace: "pre-wrap",
                    userSelect: "text",
                    marginTop: 6,
                    color: result.ok ? undefined : tokens.colorPaletteRedForeground1,
                  }}
                >
                  {result.ok ? result.text : result.errorText}
                </Text>
              </SearchResultItem>
            </div>
          ))}
        </SearchResultList>
      ) : null}

      {errorMessage ? <MessageBar intent="error" data-testid="input-error"><MessageBarBody>{errorMessage}</MessageBarBody></MessageBar> : null}
    </div>
  );

  return (
    <main style={{ height: "100vh" }} data-testid="input-panel">
      <BaseLayout
        header={<Header draggable><Text weight="semibold">{t("input.title")}</Text></Header>}
        content={
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              padding: 10,
              boxSizing: "border-box",
              gap: 10,
              minHeight: "100%",
            }}
          >
            <div style={{ display: "flex", flexDirection: "row", gap: 10, flexGrow: 1, minHeight: 0 }}>
              <div style={{ width: "50%", display: "flex", flexDirection: "column", gap: 10 }}>
                <Textarea
                  ref={c.textareaRef}
                  autoFocus
                  className="non-draggable-area"
                  aria-label={t("input.title")}
                  placeholder={t("input.placeholder")}
                  value={c.text}
                  disabled={!c.idle}
                  resize="none"
                  style={{ flexGrow: 1, width: "100%", height: "100%" }}
                  onChange={(_, data) => c.setText(data.value)}
                  onKeyDown={(event) => {
                    if (event.key === "Enter" && !event.shiftKey) {
                      event.preventDefault();
                      void c.translate();
                    }
                  }}
                />
              </div>
              <div
                style={{
                  width: "50%",
                  minWidth: 0,
                  overflowY: "auto",
                  borderRadius: tokens.borderRadiusMedium,
                  backgroundColor: tokens.colorNeutralBackground2,
                  padding: 5,
                  boxSizing: "border-box",
                }}
              >
                {resultContent}
              </div>
            </div>
            {!c.idle ? <ProgressBar aria-label={t("selection.loading")} /> : <div style={{ minHeight: 2 }} />}
          </div>
        }
        footer={
          <Footer draggable>
            <Button className="non-draggable-area" appearance="subtle" size="small" icon={<DeleteRegular />} onClick={c.clear} disabled={!showClear}>
              {t("input.action.clear")}
            </Button>
            <Button className="non-draggable-area" appearance="primary" size="small" icon={<SendRegular />} onClick={() => void c.translate()} disabled={!c.idle || !c.text.trim()}>
              {t("input.action.translate")}
            </Button>
          </Footer>
        }
      />
    </main>
  );
}

export default InputPanelView;
