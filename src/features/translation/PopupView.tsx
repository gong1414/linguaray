import { useEffect, useRef, useState } from "react";
import {
  Badge,
  Button,
  Card,
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
import { useUiStyles } from "../../ui/styles";
import type { PopupController } from "./popupController";
import type { TranslationState } from "./types";

export function headlineFor(s: TranslationState): string {
  switch (s.kind) {
    case "loading": return t("selection.loading");
    case "single-success":
    case "multi-success":
    case "partial": return t("selection.multi.title");
    case "error":
      return s.sub === "network" ? t("selection.error.network")
        : s.sub === "config-key" ? t("selection.error.config.key")
          : s.sub === "config-401" ? t("selection.error.config.auth") : s.message;
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
    minHeight: "calc(100vh - 12px)",
    maxHeight: "calc(100vh - 12px)",
    margin: "6px",
    padding: tokens.spacingVerticalS,
    overflowY: "auto",
    backgroundColor: tokens.colorNeutralBackground1,
    border: `${tokens.strokeWidthThin} solid ${tokens.colorNeutralStroke2}`,
    borderRadius: tokens.borderRadiusLarge,
    boxShadow: tokens.shadow16,
  },
  loading: {
    minHeight: "8rem",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    gap: tokens.spacingHorizontalS,
  },
});

function CardActions({
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
  const styles = useUiStyles();
  const copyLabel = copied ? t("selection.action.copied") : t("selection.action.copy");
  const speakLabel = speaking ? t("selection.action.stop") : t("selection.action.speak");
  const pinLabel = c.pinned ? t("selection.action.unpin") : t("selection.action.pin");
  const favoriteLabel = favorited ? t("selection.action.favorited") : t("selection.action.favorite");
  return (
    <div className={styles.row}>
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
        <Button
          appearance={favorited ? "primary" : "subtle"}
          size="small"
          icon={favorited ? <StarFilled /> : <StarRegular />}
          aria-label={favoriteLabel}
          onClick={() => void c.favoriteText(c.lastSource || text, text).then(onFavorited).catch(() => {})}
        />
      </Tooltip>
    </div>
  );
}

/** Pure presentational selection popup. All capability calls arrive via the controller. */
export function PopupView({ c }: { c: PopupController }) {
  const styles = useUiStyles();
  const popupStyles = usePopupStyles();
  const single = c.state.kind === "single-success" ? c.state : null;
  const multi = c.state.kind === "multi-success" || c.state.kind === "partial" ? c.state.results : null;
  const errorState = c.state.kind === "error" ? c.state : null;
  const isErrorShell = c.state.kind === "error" || c.state.kind === "offline" || c.state.kind === "no-selection" || c.state.kind === "no-permission" || c.state.kind === "no-provider";
  const [copiedKey, setCopiedKey] = useState<string | null>(null);
  const [favoritedKey, setFavoritedKey] = useState<string | null>(null);
  const [speakingKey, setSpeakingKey] = useState<string | null>(null);
  const copiedTimer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  useEffect(() => () => copiedTimer.current && clearTimeout(copiedTimer.current), []);

  const markCopied = (uuid: string) => {
    setCopiedKey(uuid);
    if (copiedTimer.current) clearTimeout(copiedTimer.current);
    copiedTimer.current = setTimeout(() => setCopiedKey(null), COPIED_FEEDBACK_MS);
  };

  return (
    <section
      aria-label={headlineFor(c.state)}
      aria-busy={c.state.kind === "loading" || undefined}
      data-testid="popup-shell"
      tabIndex={-1}
      onKeyDown={(e) => { if (e.key === "Escape") { e.preventDefault(); c.dismiss(); } }}
      className={popupStyles.surface}
      style={{ outline: "none" }}
    >
      {c.state.kind === "loading" && (
        <div className={popupStyles.loading} data-testid="popup-loading">
          <Spinner size="tiny" />
          <Text size={300}>{t("selection.loading")}</Text>
          {c.hasSource && <Button appearance="subtle" size="small" icon={<ArrowClockwiseRegular />} onClick={c.retrySelection}>{t("selection.action.retry")}</Button>}
        </div>
      )}

      {single && (
        <Card appearance="outline" size="small" data-testid="popup-card">
          <div className={styles.stackTight}>
            <div className={styles.rowBetween}>
              <Badge appearance="tint" color="brand">{c.engineLabel(single.engine)}</Badge>
              <CardActions c={c} text={single.text} copied={copiedKey === "__single__"} favorited={favoritedKey === "__single__"} speaking={speakingKey === "__single__"} onCopied={() => markCopied("__single__")} onFavorited={() => setFavoritedKey("__single__")} onSpeaking={(on) => setSpeakingKey(on ? "__single__" : null)} />
            </div>
            <Text className={styles.preWrap}>{single.text}</Text>
          </div>
        </Card>
      )}

      {multi && (
        <div className={styles.list} data-multi="true">
          {multi.map((result) => (
            <Card key={result.uuid} appearance="outline" size="small" data-testid="popup-card">
              <div className={styles.stackTight}>
                <div className={styles.rowBetween}>
                  <Badge appearance="tint" color={result.ok ? "brand" : "subtle"}>{c.engineLabel(result.engine)}</Badge>
                  {result.ok && result.text && <CardActions c={c} text={result.text} copied={copiedKey === result.uuid} favorited={favoritedKey === result.uuid} speaking={speakingKey === result.uuid} onCopied={() => markCopied(result.uuid)} onFavorited={() => setFavoritedKey(result.uuid)} onSpeaking={(on) => setSpeakingKey(on ? result.uuid : null)} />}
                </div>
                {result.ok ? <Text className={styles.preWrap}>{result.text}</Text> : <Text className={styles.danger}>{result.errorText}</Text>}
              </div>
            </Card>
          ))}
        </div>
      )}

      {(single || multi) && c.hasSource && <div className={styles.end}><Button appearance="subtle" size="small" icon={<ArrowClockwiseRegular />} onClick={c.retrySelection}>{t("selection.action.retry")}</Button></div>}

      {isErrorShell && (
        <div className={styles.empty} role="alert" data-testid="popup-error">
          <ErrorCircleRegular fontSize={24} aria-hidden />
          <Text weight="semibold">{headlineFor(c.state)}</Text>
          {errorState?.sub === "network" && c.hasSource && <Button appearance="secondary" size="small" icon={<ArrowClockwiseRegular />} onClick={c.retrySelection}>{t("selection.action.retry")}</Button>}
          {(errorState?.sub === "config-key" || errorState?.sub === "config-401" || c.state.kind === "no-provider") && <Button appearance="subtle" size="small" icon={<SettingsRegular />} onClick={() => void c.openSettings("provider-center")}>{t("selection.action.openSettings")}</Button>}
        </div>
      )}

      {c.state.kind === "keystore-corrupt" && (
        <div className={styles.empty} role="alert" data-testid="popup-keystore">
          <ErrorCircleRegular fontSize={24} aria-hidden />
          <Text weight="semibold">{t("selection.error.keystore")}</Text>
          <Button appearance="subtle" size="small" icon={<SettingsRegular />} onClick={() => void c.openSettings("keystore-recovery")}>{t("selection.action.recovery")}</Button>
        </div>
      )}
    </section>
  );
}

export default PopupView;
