import {
  Badge,
  Button,
  Card,
  Field,
  MessageBar,
  MessageBarBody,
  Spinner,
  Text,
  Textarea,
  Tooltip,
} from "@fluentui/react-components";
import { DeleteRegular, SendRegular, StarFilled, StarRegular } from "@fluentui/react-icons";
import { t } from "../../app/i18n";
import { useUiStyles } from "../../ui/styles";
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

/** Pure presentational input window (props/callbacks only). */
export function InputPanelView({ c }: { c: InputController }) {
  const styles = useUiStyles();
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

  return (
    <main className={styles.windowPage} data-testid="input-panel">
      <Text as="h1" size={400} weight="semibold" className={styles.title}>{t("input.title")}</Text>
      <Field>
        <Textarea
          ref={c.textareaRef}
          aria-label={t("input.title")}
          placeholder={t("input.placeholder")}
          rows={4}
          value={c.text}
          disabled={!c.idle}
          resize="vertical"
          onChange={(e) => c.setText(e.currentTarget.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && !e.shiftKey) {
              e.preventDefault();
              void c.translate();
            }
          }}
        />
      </Field>
      <div className={styles.rowWrap}>
        <Button appearance="secondary" icon={<DeleteRegular />} onClick={c.clear} disabled={!showClear}>{t("input.action.clear")}</Button>
        <Button appearance="primary" icon={!c.idle ? <Spinner size="tiny" aria-label={t("selection.loading")} /> : <SendRegular />} onClick={() => void c.translate()} disabled={!c.idle || !c.text.trim()}>{t("input.action.translate")}</Button>
      </div>

      {single && (
        <Card appearance="outline" size="small" data-testid="input-result">
          <div className={styles.stackTight}>
            <div className={styles.rowBetween}>
              <Badge appearance="tint" color="brand">{engineLabel(single.engine)}</Badge>
              {favoriteButton("__single__", single.text)}
            </div>
            <Text className={styles.preWrap}>{single.text}</Text>
          </div>
        </Card>
      )}

      {multi && (
        <div className={styles.list} data-multi="true">
          {multi.map((result) => (
            <Card key={result.uuid} appearance="outline" size="small" data-testid="input-result">
              <div className={styles.stackTight}>
                <div className={styles.rowBetween}>
                  <Badge appearance="tint" color={result.ok ? "brand" : "subtle"}>{engineLabel(result.engine)}</Badge>
                  {result.ok && result.text && favoriteButton(result.uuid, result.text)}
                </div>
                {result.ok ? <Text className={styles.preWrap}>{result.text}</Text> : <Text className={styles.danger}>{result.errorText}</Text>}
              </div>
            </Card>
          ))}
        </div>
      )}

      {errorMessage && <MessageBar intent="error" data-testid="input-error"><MessageBarBody>{errorMessage}</MessageBarBody></MessageBar>}
    </main>
  );
}

export default InputPanelView;
