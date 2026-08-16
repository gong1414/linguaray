import {
  Button,
  Card,
  Divider,
  MessageBar,
  MessageBarBody,
  MessageBarTitle,
  ProgressBar,
  Spinner,
  Switch,
  Text,
} from "@fluentui/react-components";
import { ArrowClockwiseRegular, ArrowDownloadRegular } from "@fluentui/react-icons";
import { detectLocale } from "../../app/i18n";
import { useUiStyles } from "../../ui/styles";
import { UPDATER_COPY, updaterErrorText } from "./copy";
import type { UpdaterController } from "./controller";
import type { AvailableUpdate } from "./model";

/** Pure presentational updater panel driven by the phase machine. */
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
        <Text as="h2" size={500} weight="semibold" className={styles.title}>{t.title}</Text>
        {knownVersion && <Text size={300} className={styles.muted} data-testid="updater-current-version">{t.currentVersion}: {knownVersion}</Text>}
      </div>

      <Card appearance="outline" data-testid="updater-status">
        <div className={styles.stack}>
          {phase.kind === "checking" && <div className={styles.row}><Spinner size="tiny" /><Text size={300}>{t.status.checking}</Text></div>}
          {phase.kind === "upToDate" && <Text>{t.status.upToDate}</Text>}
          {phase.kind === "error" && (
            <MessageBar intent="error" data-testid="updater-error">
              <MessageBarBody><MessageBarTitle>{t.status.errorPrefix}</MessageBarTitle>{updaterErrorText(detectLocale(), phase.message)}</MessageBarBody>
            </MessageBar>
          )}

          {availableUpdate && (
            <div className={styles.stackTight}>
              <Text weight="semibold" data-testid="updater-next">{t.status.available} {availableUpdate.next}</Text>
              {availableUpdate.notes && (
                <details>
                  <summary>{t.releaseNotes}</summary>
                  <Text as="pre" size={200} className={`${styles.muted} ${styles.preWrap}`}>{availableUpdate.notes}</Text>
                </details>
              )}
            </div>
          )}

          {phase.kind === "downloading" && (
            <div className={styles.stackTight} data-testid="updater-progress">
              <Text size={300}>{percent !== null ? `${t.progress.downloading} ${percent}%` : t.progress.unknownSize}</Text>
              <ProgressBar value={percent !== null ? percent / 100 : undefined} aria-label={percent !== null ? t.progress.downloading : t.progress.unknownSize} />
            </div>
          )}
          {phase.kind === "installing" && <Text size={300} data-testid="updater-installing">{t.progress.installing}</Text>}
          {phase.kind === "readyToRelaunch" && (
            <div className={styles.rowWrap}>
              <Text size={300}>{t.progress.installedHint}</Text>
              <Button appearance="primary" size="small" data-testid="updater-relaunch" onClick={c.relaunch}>{t.action.relaunch}</Button>
            </div>
          )}

          <div className={styles.rowWrap}>
            <Button appearance="secondary" size="small" icon={<ArrowClockwiseRegular />} disabled={busy} onClick={c.check} data-testid="updater-check-again">{t.action.checkAgain}</Button>
            {phase.kind === "available" && (
              <Button appearance="primary" size="small" icon={<ArrowDownloadRegular />} onClick={c.install} data-testid="updater-download">{t.action.downloadInstall}</Button>
            )}
          </div>
        </div>
      </Card>

      <Divider className={styles.dividerSpace} />
      <Switch
        checked={c.autoCheck}
        label={t.autoCheckLabel}
        aria-description={t.autoCheckHint}
        data-testid="updater-autocheck"
        onChange={(_, data) => c.toggleAutoCheck(data.checked)}
      />
      <Text size={200} className={styles.muted}>{t.autoCheckHint}</Text>
      {c.autoCheckError && <MessageBar intent="error" data-testid="updater-autocheck-error"><MessageBarBody>{c.autoCheckError}</MessageBarBody></MessageBar>}
    </section>
  );
}

export default UpdaterPanelView;
