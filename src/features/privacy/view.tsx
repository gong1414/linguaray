import { Alert, Button, Modal, Select, Spin, Switch, Tag, Typography } from "antd";
import { CloseOutlined, CopyOutlined } from "@ant-design/icons";
import { Setting, SettingGroup, SettingGroupList } from "../../ui/x";
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

/** LinguaRay privacy state mapped onto Ant Design setting groups. */
export function PrivacyView(props: PrivacyViewProps) {
  const t = PRIVACY_COPY[props.locale];
  const styles = useUiStyles();
  const countCopy = t.records.replace("{count}", String(props.status?.record_count ?? 0));
  const externalLine = props.external?.state === "enabled" ? t.externalOn.replace("{port}", String(props.external.port ?? "")) : t.externalOff;

  return (
    <section aria-label={t.title} aria-busy={props.loading || undefined} data-testid="privacy-page">
      <SettingGroupList>
        {props.error ? <Alert type="error" showIcon title={t.loadFailed} description={props.error} action={<Button onClick={props.onRetry}>{t.retry}</Button>} data-testid="privacy-error" /> : null}
        {props.loading ? <div className={styles.row} data-testid="privacy-loading"><Spin size="small" /><Typography.Text>{t.loading}</Typography.Text></div> : null}
        {!props.error && !props.loading && props.status ? (
          <>
            <div data-testid="history-panel">
              <SettingGroup title={t.historyTitle}>
                <Setting label={t.historyEnable} description={props.status.enabled ? t.historyEnabledNotice : t.historyDisabledNotice} control={<Switch aria-label={t.historyEnable} checked={props.status.enabled} disabled={props.busy !== null} onChange={props.onEnabledChange} />} />
                <Setting label={t.retention} control={<Select aria-label={t.retention} value={String(props.status.retention_days)} disabled={!props.status.enabled || props.busy !== null} options={[{ value: "30", label: t.retention30 }, { value: "90", label: t.retention90 }]} onChange={(value) => props.onRetentionChange(Number(value) as HistoryRetentionDays)} />} />
                <Setting label={countCopy} control={<Button type="primary" danger icon={props.busy === "clear" ? <Spin size="small" /> : undefined} disabled={props.status.record_count === 0 || props.busy !== null} onClick={props.onOpenClear}>{t.clearAll}</Button>} />
              </SettingGroup>
            </div>
            <div data-testid="external-panel">
              <SettingGroup title={t.externalTitle}>
                <Setting
                  label={t.externalTitle}
                  description={t.externalHint}
                  control={
                    <div className={styles.rowWrap}>
                      <Tag color={props.external?.state === "enabled" ? "success" : "default"}>{externalLine}</Tag>
                      <Button type="primary" size="small" icon={props.externalBusy && props.external?.state !== "enabled" ? <Spin size="small" /> : undefined} disabled={props.external?.state === "enabled" || props.externalBusy} onClick={props.onEnableExternal}>{t.externalEnable}</Button>
                      <Button size="small" icon={props.externalBusy && props.external?.state === "enabled" ? <Spin size="small" /> : undefined} disabled={props.external?.state !== "enabled" || props.externalBusy} onClick={props.onDisableExternal}>{t.externalDisable}</Button>
                      <Button size="small" disabled={props.external?.state !== "enabled" || props.externalBusy} onClick={props.onRegenToken}>{t.externalRegen}</Button>
                    </div>
                  }
                />
                {props.tokenOnce ? (
                  <Alert
                    type="warning"
                    showIcon
                    title={t.externalTokenOnce}
                    description={<div className={styles.stackTight}><Typography.Text>{t.externalTokenHint}</Typography.Text><div className={styles.rowWrap}><code className={styles.code}>{props.tokenOnce}</code><Button icon={<CopyOutlined aria-hidden />} onClick={props.onCopyToken}>{props.tokenCopied ? t.copied : t.copyToken}</Button></div></div>}
                    data-testid="external-token"
                  />
                ) : null}
              </SettingGroup>
            </div>
          </>
        ) : null}
        {props.toasts.length > 0 ? (
          <div className={styles.stack} aria-live="polite" data-testid="privacy-toasts">
            {props.toasts.map((entry) => <Alert key={entry.id} type={entry.variant === "success" ? "success" : "error"} showIcon title={entry.message} action={<Button type="text" size="small" icon={<CloseOutlined aria-hidden />} aria-label={t.dismissToast} onClick={() => props.onDismissToast(entry.id)} />} />)}
          </div>
        ) : null}
      </SettingGroupList>
      <Modal
        open={props.clearOpen}
        title={t.clearConfirmTitle}
        onCancel={props.onCloseClear}
        data-testid="privacy-clear-modal"
        footer={[<Button key="cancel" onClick={props.onCloseClear}>{t.cancel}</Button>, <Button key="clear" type="primary" danger onClick={props.onConfirmClear}>{t.clearAll}</Button>]}
      >
        <Typography.Paragraph>{t.clearConfirmMessage}</Typography.Paragraph>
      </Modal>
    </section>
  );
}

export default PrivacyView;
