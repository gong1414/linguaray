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
  MessageBar,
  MessageBarActions,
  MessageBarBody,
  MessageBarTitle,
  Spinner,
  Text,
} from "@fluentui/react-components";
import { KeyboardRegular } from "@fluentui/react-icons";
import { detectLocale } from "../../app/i18n";
import { useUiStyles } from "../../ui/styles";
import { SHORTCUTS_COPY } from "./copy";
import { SHORTCUT_ACTIONS, DEFAULT_SHORTCUT_MAP, type ShortcutAction, type ShortcutSnapshot } from "./model";
import type { ShortcutsController } from "./controller";

function entryFor(snapshot: ShortcutSnapshot, action: ShortcutAction) {
  return snapshot.entries.find((entry) => entry.action === action) ?? {
    action,
    combo: DEFAULT_SHORTCUT_MAP[action],
    available: false,
    registration_state: "unavailable" as const,
    registration_error: null,
  };
}

/** Pure presentational shortcuts page. */
export function ShortcutsView({ c }: { c: ShortcutsController }) {
  const t = SHORTCUTS_COPY[detectLocale()];
  const styles = useUiStyles();

  return (
    <section className={styles.page} aria-label={t.title} data-testid="shortcuts-page">
      <Text as="h2" size={500} weight="semibold" className={styles.title}>{t.title}</Text>

      {c.loadError && (
        <MessageBar intent="error" data-testid="shortcuts-load-error">
          <MessageBarBody><MessageBarTitle>{t.loadFailed}</MessageBarTitle></MessageBarBody>
          <MessageBarActions><Button size="small" appearance="secondary" onClick={c.retryLoad}>{t.retry}</Button></MessageBarActions>
        </MessageBar>
      )}

      {!c.loadError && !c.snapshot && <div className={styles.row} role="status"><Spinner size="tiny" /><Text size={300}>{t.loading}</Text></div>}

      {c.snapshot && SHORTCUT_ACTIONS.map((action) => {
        const entry = entryFor(c.snapshot!, action);
        const isRecording = c.recordingAction === action;
        const conflict = c.conflict?.action === action ? c.conflict : null;
        const registrationFailed = c.localFailures[action] || entry.registration_state === "registration_failed";
        const changeLabel = t.changeLabel.replace("{action}", t.actions[action]);

        return (
          <Card key={action} appearance="outline" size="small" data-action={action}>
            <div className={styles.stackTight}>
              <div className={styles.rowBetween}>
                <div className={styles.row}>
                  <KeyboardRegular fontSize={20} aria-hidden />
                  <div className={styles.stackTight}>
                    <Text weight="semibold">{t.actions[action]}</Text>
                    {!entry.available && <Text size={200} className={styles.muted}>{t.unavailable}</Text>}
                  </div>
                </div>
                <div className={styles.row}>
                  {isRecording ? (
                    <>
                      <Button
                        appearance="secondary"
                        size="small"
                        aria-label={t.recordingPrompt}
                        data-recorder-action={action}
                        onKeyDown={c.onRecorderKeyDown}
                      >
                        {t.recordingPrompt} <span className={styles.monospace}>{c.recordedCombo || "…"}</span>
                      </Button>
                      <Button appearance="subtle" size="small" onClick={c.cancelRecording}>{t.cancel}</Button>
                    </>
                  ) : (
                    <>
                      <Badge appearance="tint" size="large" data-testid={`shortcut-chip-${action}`}>
                        <span className={styles.monospace}>{entry.combo}</span>
                      </Badge>
                      <Button
                        appearance="subtle"
                        size="small"
                        aria-label={changeLabel}
                        data-change-action={action}
                        data-testid={`shortcuts-change-${action}`}
                        disabled={!entry.available || c.busy !== null}
                        onClick={() => c.change(action)}
                      >
                        {t.change}
                      </Button>
                    </>
                  )}
                </div>
              </div>

              {conflict && (
                <div className={styles.rowWrap} data-testid={`shortcut-conflict-${action}`}>
                  <Text className={styles.danger}>{t.conflictMessage.replace("{action}", t.actions[conflict.otherAction])}</Text>
                  <Button appearance="primary" size="small" icon={c.busy === "save" ? <Spinner size="tiny" /> : undefined} disabled={c.busy === "save"} onClick={c.overrideConflict}>{t.override}</Button>
                  <Button appearance="subtle" size="small" onClick={c.cancelRecording}>{t.cancel}</Button>
                </div>
              )}
              {registrationFailed && <Text size={200} className={styles.warning} data-testid={`shortcut-regfail-${action}`}>{t.registrationFailed}</Text>}
            </div>
          </Card>
        );
      })}

      {c.operationError && <MessageBar intent="error" data-testid="shortcuts-operation-error"><MessageBarBody>{c.operationError === "reset" ? t.resetFailed : t.saveFailed}</MessageBarBody></MessageBar>}

      {c.snapshot && <div className={styles.end}><Button appearance="secondary" disabled={!c.differsFromDefaults || c.busy !== null} onClick={c.openReset} data-testid="shortcuts-reset-trigger">{t.resetDefaults}</Button></div>}

      <Dialog open={c.resetOpen} onOpenChange={(_, data) => !data.open && c.closeReset()}>
        <DialogSurface data-testid="shortcuts-reset-modal">
          <DialogBody>
            <DialogTitle>{t.resetConfirmTitle}</DialogTitle>
            <DialogContent><Text>{t.resetConfirmMessage}</Text></DialogContent>
            <DialogActions>
              <Button appearance="secondary" onClick={c.closeReset}>{t.cancel}</Button>
              <Button appearance="primary" onClick={c.reset} data-testid="shortcuts-reset-confirm">{t.useDefaults}</Button>
            </DialogActions>
          </DialogBody>
        </DialogSurface>
      </Dialog>
    </section>
  );
}

export default ShortcutsView;
