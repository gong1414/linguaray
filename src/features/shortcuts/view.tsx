import { Alert, Button, Modal, Spin, Tag, Typography } from "antd";
import { Setting, SettingGroup, SettingGroupList } from "../../ui/x";
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

/** LinguaRay shortcuts mapped onto Ant Design setting rows. */
export function ShortcutsView({ c }: { c: ShortcutsController }) {
  const t = SHORTCUTS_COPY[detectLocale()];
  const styles = useUiStyles();

  return (
    <section aria-label={t.title} data-testid="shortcuts-page">
      <SettingGroupList>
        {c.loadError ? <Alert type="error" showIcon title={t.loadFailed} action={<Button onClick={c.retryLoad}>{t.retry}</Button>} data-testid="shortcuts-load-error" /> : null}
        {!c.loadError && !c.snapshot ? <div className={styles.row} role="status"><Spin size="small" /><Typography.Text>{t.loading}</Typography.Text></div> : null}
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
                            <Button aria-label={t.recordingPrompt} data-recorder-action={action} onKeyDown={c.onRecorderKeyDown}>{t.recordingPrompt} <span className={styles.monospace}>{c.recordedCombo || "…"}</span></Button>
                            <Button type="text" onClick={c.cancelRecording}>{t.cancel}</Button>
                          </>
                        ) : (
                          <>
                            <Tag data-testid={`shortcut-chip-${action}`}><span className={styles.monospace}>{entry.combo}</span></Tag>
                            <Button type="text" aria-label={changeLabel} data-change-action={action} data-testid={`shortcuts-change-${action}`} disabled={!entry.available || c.busy !== null} onClick={() => c.change(action)}>{t.change}</Button>
                          </>
                        )}
                      </div>
                    }
                  />
                  {registrationFailed ? <Typography.Text type="warning" data-testid={`shortcut-regfail-${action}`}>{t.registrationFailed}</Typography.Text> : null}
                  {conflict ? (
                    <Alert
                      type="warning"
                      showIcon
                      title={t.conflictMessage.replace("{action}", t.actions[conflict.otherAction])}
                      data-testid={`shortcut-conflict-${action}`}
                      action={<div className={styles.row}><Button type="primary" icon={c.busy === "save" ? <Spin size="small" /> : undefined} disabled={c.busy === "save"} onClick={c.overrideConflict}>{t.override}</Button><Button type="text" onClick={c.cancelRecording}>{t.cancel}</Button></div>}
                    />
                  ) : null}
                </div>
              );
            })}
          </SettingGroup>
        ) : null}
        {c.operationError ? <Alert type="error" showIcon title={c.operationError === "reset" ? t.resetFailed : t.saveFailed} data-testid="shortcuts-operation-error" /> : null}
        {c.snapshot ? <SettingGroup><Setting label={t.resetDefaults} control={<Button disabled={!c.differsFromDefaults || c.busy !== null} onClick={c.openReset} data-testid="shortcuts-reset-trigger">{t.resetDefaults}</Button>} /></SettingGroup> : null}
      </SettingGroupList>
      <Modal
        open={c.resetOpen}
        title={t.resetConfirmTitle}
        onCancel={c.closeReset}
        data-testid="shortcuts-reset-modal"
        footer={[<Button key="cancel" onClick={c.closeReset}>{t.cancel}</Button>, <Button key="confirm" type="primary" onClick={c.reset} data-testid="shortcuts-reset-confirm">{t.useDefaults}</Button>]}
      >
        <Typography.Paragraph>{t.resetConfirmMessage}</Typography.Paragraph>
      </Modal>
    </section>
  );
}

export default ShortcutsView;
