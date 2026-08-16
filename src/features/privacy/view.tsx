import {
  Badge,
  Button,
  Card,
  Dialog,
  DialogActions,
  DialogBody,
  DialogContent,
  DialogSurface,
  DialogTitle,
  Field,
  MessageBar,
  MessageBarActions,
  MessageBarBody,
  MessageBarTitle,
  Select,
  Spinner,
  Switch,
  Text,
} from "@fluentui/react-components";
import { CopyRegular, DismissRegular } from "@fluentui/react-icons";
import { useUiStyles } from "../../ui/styles";
import { PRIVACY_COPY } from "./copy";
import type { PrivacyToast } from "./controller";
import type { ExternalApiStatus, HistoryPrivacyStatus, HistoryRetentionDays, PrivacyBusy } from "./model";

export type PrivacyViewProps = {
  locale: "zh" | "en";
  status: HistoryPrivacyStatus | null;
  loading: boolean;
  error: string | null;
  busy: PrivacyBusy;
  clearOpen: boolean;
  toasts: PrivacyToast[];
  external: ExternalApiStatus | null;
  externalBusy: boolean;
  tokenOnce: string | null;
  tokenCopied: boolean;
  onRetry: () => void;
  onEnabledChange: (enabled: boolean) => void;
  onRetentionChange: (days: HistoryRetentionDays) => void;
  onOpenClear: () => void;
  onCloseClear: () => void;
  onConfirmClear: () => void;
  onEnableExternal: () => void;
  onDisableExternal: () => void;
  onRegenToken: () => void;
  onCopyToken: () => void;
  onDismissToast: (id: number) => void;
};

/** Pure presentational Privacy & Data page. */
export function PrivacyView(props: PrivacyViewProps) {
  const t = PRIVACY_COPY[props.locale];
  const styles = useUiStyles();
  const countCopy = t.records.replace("{count}", String(props.status?.record_count ?? 0));
  const externalLine = props.external?.state === "enabled"
    ? t.externalOn.replace("{port}", String(props.external.port ?? ""))
    : t.externalOff;

  return (
    <section className={styles.page} aria-label={t.title} aria-busy={props.loading || undefined} data-testid="privacy-page">
      <Text as="h2" size={500} weight="semibold" className={styles.title}>{t.title}</Text>

      {props.error && (
        <MessageBar intent="error" data-testid="privacy-error">
          <MessageBarBody><MessageBarTitle>{t.loadFailed}</MessageBarTitle>{props.error}</MessageBarBody>
          <MessageBarActions><Button appearance="secondary" size="small" onClick={props.onRetry}>{t.retry}</Button></MessageBarActions>
        </MessageBar>
      )}

      {props.loading && <div className={styles.row} data-testid="privacy-loading"><Spinner size="tiny" /><Text size={300}>{t.loading}</Text></div>}

      {!props.error && !props.loading && props.status && (
        <>
          <Card appearance="outline" data-testid="history-panel">
            <div className={styles.stack}>
              <div className={styles.rowBetween}>
                <div className={styles.stackTight}>
                  <Text as="h3" size={400} weight="semibold" className={styles.title}>{t.historyTitle}</Text>
                  <Text size={300} className={styles.muted}>{props.status.enabled ? t.historyEnabledNotice : t.historyDisabledNotice}</Text>
                </div>
                <Switch
                  label={t.historyEnable}
                  checked={props.status.enabled}
                  disabled={props.busy !== null}
                  onChange={(_, data) => props.onEnabledChange(data.checked)}
                />
              </div>
              <div className={styles.rowBetween}>
                <Field label={t.retention} className={styles.fieldSmall}>
                  <Select
                    aria-label={t.retention}
                    value={String(props.status.retention_days)}
                    disabled={!props.status.enabled || props.busy !== null}
                    onChange={(e) => props.onRetentionChange(Number(e.currentTarget.value) as HistoryRetentionDays)}
                  >
                    <option value="30">{t.retention30}</option>
                    <option value="90">{t.retention90}</option>
                  </Select>
                </Field>
                <div className={styles.rowWrap}>
                  <Badge appearance="tint" color={props.status.record_count > 0 ? "brand" : "subtle"}>{countCopy}</Badge>
                  <Button
                    appearance="primary"
                    icon={props.busy === "clear" ? <Spinner size="tiny" /> : undefined}
                    disabled={props.status.record_count === 0 || props.busy !== null}
                    onClick={props.onOpenClear}
                  >
                    {t.clearAll}
                  </Button>
                </div>
              </div>
            </div>
          </Card>

          <Card appearance="outline" data-testid="external-panel">
            <div className={styles.stack}>
              <div className={styles.rowBetween}>
                <div className={styles.stackTight}>
                  <Text as="h3" size={400} weight="semibold" className={styles.title}>{t.externalTitle}</Text>
                  <Text size={300} className={styles.muted}>{t.externalHint}</Text>
                </div>
                <Badge appearance="tint" color={props.external?.state === "enabled" ? "success" : "subtle"}>{externalLine}</Badge>
              </div>
              <div className={styles.rowWrap}>
                <Button appearance="primary" size="small" icon={props.externalBusy && props.external?.state !== "enabled" ? <Spinner size="tiny" /> : undefined} disabled={props.external?.state === "enabled" || props.externalBusy} onClick={props.onEnableExternal}>{t.externalEnable}</Button>
                <Button appearance="secondary" size="small" icon={props.externalBusy && props.external?.state === "enabled" ? <Spinner size="tiny" /> : undefined} disabled={props.external?.state !== "enabled" || props.externalBusy} onClick={props.onDisableExternal}>{t.externalDisable}</Button>
                <Button appearance="secondary" size="small" disabled={props.external?.state !== "enabled" || props.externalBusy} onClick={props.onRegenToken}>{t.externalRegen}</Button>
              </div>
              {props.tokenOnce && (
                <MessageBar intent="warning" data-testid="external-token">
                  <MessageBarBody>
                    <MessageBarTitle>{t.externalTokenOnce}</MessageBarTitle>
                    <div className={styles.stackTight}>
                      <Text size={300}>{t.externalTokenHint}</Text>
                      <div className={styles.rowWrap}>
                        <code className={styles.code}>{props.tokenOnce}</code>
                        <Button appearance="secondary" size="small" icon={<CopyRegular />} onClick={props.onCopyToken}>{props.tokenCopied ? t.copied : t.copyToken}</Button>
                      </div>
                    </div>
                  </MessageBarBody>
                </MessageBar>
              )}
            </div>
          </Card>
        </>
      )}

      <Dialog open={props.clearOpen} onOpenChange={(_, data) => !data.open && props.onCloseClear()}>
        <DialogSurface data-testid="privacy-clear-modal">
          <DialogBody>
            <DialogTitle>{t.clearConfirmTitle}</DialogTitle>
            <DialogContent><Text>{t.clearConfirmMessage}</Text></DialogContent>
            <DialogActions>
              <Button appearance="secondary" onClick={props.onCloseClear}>{t.cancel}</Button>
              <Button appearance="primary" onClick={props.onConfirmClear}>{t.clearAll}</Button>
            </DialogActions>
          </DialogBody>
        </DialogSurface>
      </Dialog>

      {props.toasts.length > 0 && (
        <div className={styles.stack} aria-live="polite" data-testid="privacy-toasts">
          {props.toasts.map((entry) => (
            <MessageBar key={entry.id} intent={entry.variant === "success" ? "success" : "error"}>
              <MessageBarBody>{entry.message}</MessageBarBody>
              <MessageBarActions containerAction={<Button appearance="transparent" icon={<DismissRegular />} aria-label={t.dismissToast} onClick={() => props.onDismissToast(entry.id)} />} />
            </MessageBar>
          ))}
        </div>
      )}
    </section>
  );
}

export default PrivacyView;
