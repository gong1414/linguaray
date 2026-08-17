import { Alert, Button, Card, Divider, Progress, Spin, Switch, Typography } from "antd";
import { DownloadOutlined, ReloadOutlined } from "@ant-design/icons";
import { detectLocale } from "../../app/i18n";
import { useUiStyles } from "../../ui/styles";
import { UPDATER_COPY, updaterErrorText } from "./copy";
import type { UpdaterController } from "./controller";
import type { AvailableUpdate } from "./model";

/** Pure presentational Ant Design updater panel. */
export function UpdaterPanelView({ c }: { c: UpdaterController }) {
  const t = UPDATER_COPY[detectLocale()];
  const styles = useUiStyles();
  const phase = c.phase;
  const availableUpdate: AvailableUpdate | null =
    phase.kind === "available" || phase.kind === "downloading" || phase.kind === "installing" || phase.kind === "readyToRelaunch"
      ? phase.update
      : null;
  const knownVersion = phase.kind === "upToDate" ? phase.version : availableUpdate?.current ?? null;
  const percent = phase.kind === "downloading" ? phase.percent : null;
  const busy = phase.kind === "downloading" || phase.kind === "installing";

  return (
    <section className={styles.page} data-testid="updater-panel" aria-label={t.title}>
      <div className={styles.rowBetween}>
        <Typography.Title level={4} className={styles.title}>{t.title}</Typography.Title>
        {knownVersion ? <Typography.Text type="secondary" data-testid="updater-current-version">{t.currentVersion}: {knownVersion}</Typography.Text> : null}
      </div>
      <Card data-testid="updater-status">
        <div className={styles.stack}>
          {phase.kind === "checking" ? <div className={styles.row}><Spin size="small" /><Typography.Text>{t.status.checking}</Typography.Text></div> : null}
          {phase.kind === "upToDate" ? <Typography.Text>{t.status.upToDate}</Typography.Text> : null}
          {phase.kind === "error" ? <Alert type="error" showIcon title={t.status.errorPrefix} description={updaterErrorText(detectLocale(), phase.message)} data-testid="updater-error" /> : null}
          {availableUpdate ? (
            <div className={styles.stackTight}>
              <Typography.Text strong data-testid="updater-next">{t.status.available} {availableUpdate.next}</Typography.Text>
              {availableUpdate.notes ? <details><summary>{t.releaseNotes}</summary><Typography.Paragraph type="secondary" className={styles.preWrap}>{availableUpdate.notes}</Typography.Paragraph></details> : null}
            </div>
          ) : null}
          {phase.kind === "downloading" ? (
            <div className={styles.stackTight} data-testid="updater-progress">
              <Typography.Text>{percent !== null ? `${t.progress.downloading} ${percent}%` : t.progress.unknownSize}</Typography.Text>
              <Progress percent={percent ?? undefined} status="active" aria-label={percent !== null ? t.progress.downloading : t.progress.unknownSize} />
            </div>
          ) : null}
          {phase.kind === "installing" ? <Typography.Text data-testid="updater-installing">{t.progress.installing}</Typography.Text> : null}
          {phase.kind === "readyToRelaunch" ? <div className={styles.rowWrap}><Typography.Text>{t.progress.installedHint}</Typography.Text><Button type="primary" size="small" data-testid="updater-relaunch" onClick={c.relaunch}>{t.action.relaunch}</Button></div> : null}
          <div className={styles.rowWrap}>
            <Button icon={<ReloadOutlined aria-hidden />} disabled={busy} onClick={c.check} data-testid="updater-check-again">{t.action.checkAgain}</Button>
            {phase.kind === "available" ? <Button type="primary" icon={<DownloadOutlined aria-hidden />} onClick={c.install} data-testid="updater-download">{t.action.downloadInstall}</Button> : null}
          </div>
        </div>
      </Card>
      <Divider className={styles.dividerSpace} />
      <div className={styles.rowBetween}>
        <div className={styles.stackTight}>
          <Typography.Text>{t.autoCheckLabel}</Typography.Text>
          <Typography.Text type="secondary">{t.autoCheckHint}</Typography.Text>
        </div>
        <Switch checked={c.autoCheck} aria-label={t.autoCheckLabel} data-testid="updater-autocheck" onChange={c.toggleAutoCheck} />
      </div>
      {c.autoCheckError ? <Alert type="error" showIcon title={c.autoCheckError} data-testid="updater-autocheck-error" /> : null}
    </section>
  );
}

export default UpdaterPanelView;
