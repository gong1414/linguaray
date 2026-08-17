import { useEffect, useRef, useState } from "react";
import {
  Button,
  Spinner,
  Text,
  Tooltip,
  makeStyles,
  tokens,
} from "@fluentui/react-components";
import {
  ArrowClockwiseRegular,
  CheckmarkRegular,
  CopyRegular,
  ErrorCircleRegular,
  PinOffRegular,
  PinRegular,
  SettingsRegular,
  Speaker2Regular,
  SpeakerOffRegular,
  StarFilled,
  StarRegular,
} from "@fluentui/react-icons";
import { t } from "../../app/i18n";
import { BaseLayout, Footer, Header, SearchResultItem, SearchResultList } from "../../ui/ueli";
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

const usePopupStyles = makeStyles({
  surface: {
    boxSizing: "border-box",
    height: "calc(100vh - 12px)",
    margin: "6px",
    overflow: "hidden",
    backgroundColor: tokens.colorNeutralBackground1,
    border: `${tokens.strokeWidthThin} solid ${tokens.colorNeutralStroke2}`,
    borderRadius: tokens.borderRadiusLarge,
    boxShadow: tokens.shadow16,
  },
  loading: {
    minHeight: "100%",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    gap: tokens.spacingHorizontalS,
  },
  actions: {
    display: "flex",
    alignItems: "center",
    gap: tokens.spacingHorizontalXXS,
  },
  resultText: {
    whiteSpace: "pre-wrap",
    userSelect: "text",
    marginTop: tokens.spacingVerticalXS,
  },
  error: {
    minHeight: "100%",
    display: "flex",
    flexDirection: "column",
    alignItems: "center",
    justifyContent: "center",
    gap: tokens.spacingVerticalS,
    textAlign: "center",
  },
});

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
  const styles = usePopupStyles();
  const copyLabel = copied ? t("selection.action.copied") : t("selection.action.copy");
  const speakLabel = speaking ? t("selection.action.stop") : t("selection.action.speak");
  const pinLabel = c.pinned ? t("selection.action.unpin") : t("selection.action.pin");
  const favoriteLabel = favorited ? t("selection.action.favorited") : t("selection.action.favorite");

  return (
    <div className={styles.actions}>
      <Tooltip content={copyLabel} relationship="label">
        <Button appearance={copied ? "primary" : "subtle"} size="small" icon={copied ? <CheckmarkRegular /> : <CopyRegular />} aria-label={copyLabel} onClick={() => void c.copyText(text).then(onCopied).catch(() => {})} />
      </Tooltip>
      <Tooltip content={speakLabel} relationship="label">
        <Button
          appearance={speaking ? "primary" : "subtle"}
          size="small"
          icon={speaking ? <SpeakerOffRegular /> : <Speaker2Regular />}
          aria-label={speakLabel}
          onClick={() => {
            if (speaking) void c.stopSpeaking().finally(() => onSpeaking(false));
            else void c.speak(text).then(() => onSpeaking(true)).catch(() => onSpeaking(false));
          }}
        />
      </Tooltip>
      <Tooltip content={pinLabel} relationship="label">
        <Button appearance={c.pinned ? "primary" : "subtle"} size="small" icon={c.pinned ? <PinOffRegular /> : <PinRegular />} aria-label={pinLabel} onClick={() => c.pinned ? c.unpin() : c.pin()} />
      </Tooltip>
      <Tooltip content={favoriteLabel} relationship="label">
        <Button appearance={favorited ? "primary" : "subtle"} size="small" icon={favorited ? <StarFilled /> : <StarRegular />} aria-label={favoriteLabel} onClick={() => void c.favoriteText(c.lastSource || text, text).then(onFavorited).catch(() => {})} />
      </Tooltip>
    </div>
  );
}

/** LinguaRay state adapter around Ueli's search/result window renderer. */
export function PopupView({ c }: { c: PopupController }) {
  const styles = usePopupStyles();
  const single = c.state.kind === "single-success" ? c.state : null;
  const multi = c.state.kind === "multi-success" || c.state.kind === "partial" ? c.state.results : null;
  const errorState = c.state.kind === "error" ? c.state : null;
  const isError = c.state.kind === "error" || c.state.kind === "offline" || c.state.kind === "no-selection" || c.state.kind === "no-permission" || c.state.kind === "no-provider" || c.state.kind === "keystore-corrupt";
  const [copiedKey, setCopiedKey] = useState<string | null>(null);
  const [favoritedKey, setFavoritedKey] = useState<string | null>(null);
  const [speakingKey, setSpeakingKey] = useState<string | null>(null);
  const copiedTimer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  useEffect(() => () => copiedTimer.current && clearTimeout(copiedTimer.current), []);

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

  const content = c.state.kind === "loading" ? (
    <div className={styles.loading} data-testid="popup-loading">
      <Spinner size="tiny" />
      <Text size={300}>{t("selection.loading")}</Text>
      {c.hasSource ? <Button appearance="subtle" size="small" icon={<ArrowClockwiseRegular />} onClick={c.retrySelection}>{t("selection.action.retry")}</Button> : null}
    </div>
  ) : isError ? (
    <div className={styles.error} role="alert" data-testid={c.state.kind === "keystore-corrupt" ? "popup-keystore" : "popup-error"}>
      <ErrorCircleRegular fontSize={24} aria-hidden />
      <Text weight="semibold">{headlineFor(c.state)}</Text>
      {errorState?.sub === "network" && c.hasSource ? <Button appearance="secondary" size="small" icon={<ArrowClockwiseRegular />} onClick={c.retrySelection}>{t("selection.action.retry")}</Button> : null}
      {(errorState?.sub === "config-key" || errorState?.sub === "config-401" || c.state.kind === "no-provider") ? <Button appearance="subtle" size="small" icon={<SettingsRegular />} onClick={() => void c.openSettings("provider-center")}>{t("selection.action.openSettings")}</Button> : null}
      {c.state.kind === "keystore-corrupt" ? <Button appearance="subtle" size="small" icon={<SettingsRegular />} onClick={() => void c.openSettings("keystore-recovery")}>{t("selection.action.recovery")}</Button> : null}
    </div>
  ) : (
    <div style={{ padding: 5 }}>
      <SearchResultList>
        {single ? (
          <div data-testid="popup-card">
            <SearchResultItem selected name={c.engineLabel(single.engine)} actions={resultActions("__single__", single.text)}>
              <Text className={styles.resultText}>{single.text}</Text>
            </SearchResultItem>
          </div>
        ) : null}
        {multi?.map((result, index) => (
          <div key={result.uuid} data-testid="popup-card">
            <SearchResultItem selected={index === 0} name={c.engineLabel(result.engine)} actions={result.ok && result.text ? resultActions(result.uuid, result.text) : undefined}>
              <Text className={styles.resultText} style={{ color: result.ok ? undefined : tokens.colorPaletteRedForeground1 }}>
                {result.ok ? result.text : result.errorText}
              </Text>
            </SearchResultItem>
          </div>
        ))}
      </SearchResultList>
    </div>
  );

  return (
    <section
      aria-label={headlineFor(c.state)}
      aria-busy={c.state.kind === "loading" || undefined}
      data-testid="popup-shell"
      tabIndex={-1}
      onKeyDown={(event) => { if (event.key === "Escape") { event.preventDefault(); c.dismiss(); } }}
      className={styles.surface}
      style={{ outline: "none" }}
    >
      <BaseLayout
        transparent
        header={c.state.kind === "loading" ? undefined : <Header draggable><Text weight="semibold">{headlineFor(c.state)}</Text></Header>}
        content={content}
        footer={(single || multi) && c.hasSource ? <Footer draggable><div /><Button className="non-draggable-area" appearance="subtle" size="small" icon={<ArrowClockwiseRegular />} onClick={c.retrySelection}>{t("selection.action.retry")}</Button></Footer> : undefined}
      />
    </section>
  );
}

export default PopupView;
