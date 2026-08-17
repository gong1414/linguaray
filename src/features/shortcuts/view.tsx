import {
  Badge,
  Button,
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
import { Setting, SettingGroup, SettingGroupList } from "../../ui/ueli";
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

/** LinguaRay shortcuts mapped onto Ueli setting rows. */
export function ShortcutsView({ c }: { c: ShortcutsController }) {
  const t = SHORTCUTS_COPY[detectLocale()];
  const styles = useUiStyles();

  return (
    <section aria-label={t.title} data-testid="shortcuts-page">
      <SettingGroupList>
        {c.loadError ? (
          <MessageBar intent="error" data-testid="shortcuts-load-error">
            <MessageBarBody><MessageBarTitle>{t.loadFailed}</MessageBarTitle></MessageBarBody>
            <MessageBarActions><Button size="small" appearance="secondary" onClick={c.retryLoad}>{t.retry}</Button></MessageBarActions>
          </MessageBar>
        ) : null}

        {!c.loadError && !c.snapshot ? <div className={styles.row} role="status"><Spinner size="tiny" /><Text size={300}>{t.loading}</Text></div> : null}

        {c.snapshot ? (
          <SettingGroup title={t.title}>
            {SHORTCUT_ACTIONS.map((action) => {
              const entry = entryFor(c.snapshot!, action);
              const isRecording = c.recordingAction === action;
              const conflict = c.conflict?.action === action ? c.conflict : null;
              const registrationFailed = c.localFailures[action] || entry.registration_state === "registration_failed";
              const changeLabel = t.changeLabel.replace("{action}", t.actions[action]);
              const description = !entry.available ? t.unavailable : registrationFailed ? t.registrationFailed : undefined;

              return (
                <div key={action} data-action={action}>
                  <Setting
                    label={t.actions[action]}
                    description={description}
                    control={
                      <div className={styles.rowWrap}>
                        {isRecording ? (
                          <>
                            <Button appearance="secondary" size="small" aria-label={t.recordingPrompt} data-recorder-action={action} onKeyDown={c.onRecorderKeyDown}>
                              {t.recordingPrompt} <span className={styles.monospace}>{c.recordedCombo || "…"}</span>
                            </Button>
                            <Button appearance="subtle" size="small" onClick={c.cancelRecording}>{t.cancel}</Button>
                          </>
                        ) : (
                          <>
                            <Badge appearance="tint" size="large" data-testid={`shortcut-chip-${action}`}><span className={styles.monospace}>{entry.combo}</span></Badge>
                            <Button appearance="subtle" size="small" aria-label={changeLabel} data-change-action={action} data-testid={`shortcuts-change-${action}`} disabled={!entry.available || c.busy !== null} onClick={() => c.change(action)}>{t.change}</Button>
                          </>
                        )}
                      </div>
                    }
                  />
                  {registrationFailed ? <Text size={200} className={styles.warning} data-testid={`shortcut-regfail-${action}`}>{t.registrationFailed}</Text> : null}
                  {conflict ? (
                    <MessageBar intent="warning" data-testid={`shortcut-conflict-${action}`}>
                      <MessageBarBody>{t.conflictMessage.replace("{action}", t.actions[conflict.otherAction])}</MessageBarBody>
                      <MessageBarActions>
                        <Button appearance="primary" size="small" icon={c.busy === "save" ? <Spinner size="tiny" /> : undefined} disabled={c.busy === "save"} onClick={c.overrideConflict}>{t.override}</Button>
                        <Button appearance="subtle" size="small" onClick={c.cancelRecording}>{t.cancel}</Button>
                      </MessageBarActions>
                    </MessageBar>
                  ) : null}
                </div>
              );
            })}
          </SettingGroup>
        ) : null}

        {c.operationError ? <MessageBar intent="error" data-testid="shortcuts-operation-error"><MessageBarBody>{c.operationError === "reset" ? t.resetFailed : t.saveFailed}</MessageBarBody></MessageBar> : null}

        {c.snapshot ? (
          <SettingGroup>
            <Setting
              label={t.resetDefaults}
              control={<Button appearance="secondary" disabled={!c.differsFromDefaults || c.busy !== null} onClick={c.openReset} data-testid="shortcuts-reset-trigger">{t.resetDefaults}</Button>}
            />
          </SettingGroup>
        ) : null}
      </SettingGroupList>

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
