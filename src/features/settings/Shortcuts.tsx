import {
  For,
  Show,
  createSignal,
  onCleanup,
  onMount,
  type Component,
  type JSX,
} from "solid-js";
import { Keyboard, ScanText, TextCursorInput, Clipboard } from "lucide-solid";
import {
  Button,
  Confirm,
  InlineError,
  ListRow,
  ShortcutChip,
} from "@linguaray/ui";
import { detectLocale } from "../../i18n";
import {
  shortcutCheckConflict,
  shortcutList,
  shortcutRecordingBegin,
  shortcutRecordingEnd,
  shortcutResetDefaults,
  shortcutSave,
} from "./shortcut-ipc";
import {
  DEFAULT_SHORTCUT_MAP,
  SHORTCUT_ACTIONS,
  isRegistrationFailure,
  isStaleShortcutRevision,
  mapFromSnapshot,
  type ShortcutAction,
  type ShortcutEntry,
  type ShortcutSnapshot,
} from "./shortcut-types";
import { SHORTCUTS_COPY, type ShortcutsCopy } from "./shortcuts-copy";
import "./Shortcuts.css";

export type ShortcutConflictState = {
  action: ShortcutAction;
  otherAction: ShortcutAction;
  combo: string;
};

export type ShortcutBusy = "load" | "recording" | "save" | "reset" | null;

export type ShortcutsViewProps = {
  snapshot: ShortcutSnapshot | null;
  loadError: boolean;
  recordingAction: ShortcutAction | null;
  recordedCombo: string;
  conflict: ShortcutConflictState | null;
  busy: ShortcutBusy;
  resetOpen: boolean;
  localRegistrationFailures: Partial<Record<ShortcutAction, boolean>>;
  operationError: "save" | "reset" | null;
  onRetryLoad: () => void;
  onChange: (action: ShortcutAction) => void;
  onCancelRecording: () => void;
  onRecorderKeyDown: (event: KeyboardEvent) => void;
  onOverride: () => void;
  onOpenReset: () => void;
  onCloseReset: () => void;
  onReset: () => void;
  changeRefs?: Partial<Record<ShortcutAction, HTMLButtonElement>>;
  recorderRef?: { current?: HTMLButtonElement };
  resetTriggerRef?: { current?: HTMLElement };
  copy?: ShortcutsCopy;
};

const iconFor = (action: ShortcutAction): JSX.Element => {
  switch (action) {
    case "translate_selection": return <Keyboard size={16} />;
    case "translate_input": return <TextCursorInput size={16} />;
    case "translate_clipboard": return <Clipboard size={16} />;
    case "ocr_translate": return <ScanText size={16} />;
  }
};

const entryFor = (
  snapshot: ShortcutSnapshot,
  action: ShortcutAction,
): ShortcutEntry =>
  snapshot.entries.find((entry) => entry.action === action) ?? {
    action,
    combo: DEFAULT_SHORTCUT_MAP[action],
    available: false,
    registration_state: "unavailable",
    registration_error: null,
  };

const actionLabel = (copy: ShortcutsCopy, action: ShortcutAction): string =>
  copy.actions[action];

/** Pure presentation shared with visual fixtures. */
export const ShortcutsView: Component<ShortcutsViewProps> = (props) => {
  const locale = detectLocale();
  const t = () => props.copy ?? SHORTCUTS_COPY[locale];
  const map = () => props.snapshot && mapFromSnapshot(props.snapshot);
  const differsFromDefaults = () =>
    !!map() &&
    SHORTCUT_ACTIONS.some((action) => map()![action] !== DEFAULT_SHORTCUT_MAP[action]);

  const labelsFor = (action: ShortcutAction) => ({
    recording: t().recordingPrompt,
    conflict: t().conflictTitle,
    clear: t().clearLabel.replace("{action}", actionLabel(t(), action)),
  });

  return (
    <section class="shortcuts" aria-labelledby="shortcuts-title">
      <header class="shortcuts__header">
        <h1 id="shortcuts-title">{t().title}</h1>
      </header>

      <Show when={props.loadError}>
        <div class="shortcuts__load-error">
          <InlineError>{t().loadFailed}</InlineError>
          <Button variant="secondary" size="sm" onClick={props.onRetryLoad}>
            {t().retry}
          </Button>
        </div>
      </Show>

      <Show when={!props.loadError && !props.snapshot}>
        <p class="shortcuts__loading" role="status">{t().loading}</p>
      </Show>

      <Show when={props.snapshot} keyed>
        {(snapshot) => (
          <div class="shortcuts__list" aria-label={t().title}>
            <For each={SHORTCUT_ACTIONS}>
              {(action) => {
                const entry = () => entryFor(snapshot, action);
                const isRecording = () => props.recordingAction === action;
                const conflict = () =>
                  props.conflict?.action === action ? props.conflict : null;
                const registrationFailed = () =>
                  props.localRegistrationFailures[action] ||
                  entry().registration_state === "registration_failed";
                const changeLabel = () =>
                  t().changeLabel.replace("{action}", actionLabel(t(), action));

                return (
                  <div class="shortcuts__item" data-action={action}>
                    <ListRow
                      leading={iconFor(action)}
                      title={actionLabel(t(), action)}
                      subtitle={!entry().available ? t().unavailable : undefined}
                      trailing={
                        <div class="shortcuts__trailing">
                          <Show
                            when={isRecording()}
                            fallback={
                              <>
                                <ShortcutChip
                                  shortcut={entry().combo}
                                  status="clear"
                                  labels={labelsFor(action)}
                                  disabled={!entry().available}
                                />
                                <Button
                                  variant="ghost"
                                  size="sm"
                                  aria-label={changeLabel()}
                                  data-change-action={action}
                                  ref={(el: HTMLButtonElement) => {
                                    if (props.changeRefs) props.changeRefs[action] = el;
                                  }}
                                  disabled={!entry().available || props.busy !== null}
                                  onClick={() => props.onChange(action)}
                                >
                                  {t().change}
                                </Button>
                              </>
                            }
                          >
                            <button
                              type="button"
                              class="shortcuts__recorder lr-focusable"
                              aria-label={t().recordingPrompt}
                              data-recorder-action={action}
                              ref={(el: HTMLButtonElement) => {
                                if (props.recorderRef) props.recorderRef.current = el;
                              }}
                              onKeyDown={props.onRecorderKeyDown}
                            >
                              <ShortcutChip
                                shortcut={props.recordedCombo}
                                status={conflict() ? "conflict" : "recording"}
                                labels={labelsFor(action)}
                              />
                            </button>
                            <Button
                              variant="ghost"
                              size="sm"
                              onClick={props.onCancelRecording}
                            >
                              {t().cancel}
                            </Button>
                          </Show>
                        </div>
                      }
                    />

                    <Show when={conflict()} keyed>
                      {(value) => (
                        <div class="shortcuts__feedback shortcuts__feedback--conflict">
                          <InlineError>
                            {t().conflictMessage.replace(
                              "{action}",
                              actionLabel(t(), value.otherAction),
                            )}
                          </InlineError>
                          <Button
                            variant="secondary"
                            size="sm"
                            loading={props.busy === "save"}
                            loadingLabel={t().loadingLabel}
                            onClick={props.onOverride}
                          >
                            {t().override}
                          </Button>
                          <Button variant="ghost" size="sm" onClick={props.onCancelRecording}>
                            {t().cancel}
                          </Button>
                        </div>
                      )}
                    </Show>

                    <Show when={registrationFailed()}>
                      <div class="shortcuts__feedback">
                        <InlineError variant="warning">{t().registrationFailed}</InlineError>
                      </div>
                    </Show>
                  </div>
                );
              }}
            </For>
          </div>
        )}
      </Show>

      <Show when={props.operationError}>
        <InlineError>
          {props.operationError === "reset" ? t().resetFailed : t().saveFailed}
        </InlineError>
      </Show>

      <Show when={props.snapshot}>
        <footer class="shortcuts__footer">
          <Button
            variant="secondary"
            ref={(el: HTMLButtonElement) => {
              if (props.resetTriggerRef) props.resetTriggerRef.current = el;
            }}
            disabled={!differsFromDefaults() || props.busy !== null}
            onClick={props.onOpenReset}
          >
            {t().resetDefaults}
          </Button>
        </footer>
      </Show>

      <Confirm
        open={props.resetOpen}
        onOpenChange={(open) => (open ? props.onOpenReset() : props.onCloseReset())}
        title={t().resetConfirmTitle}
        message={t().resetConfirmMessage}
        confirmLabel={t().useDefaults}
        cancelLabel={t().cancel}
        onConfirm={props.onReset}
        onCancel={props.onCloseReset}
        triggerRef={props.resetTriggerRef ?? {}}
      />
    </section>
  );
};

const modifierOnly = new Set([
  "Alt",
  "AltGraph",
  "Control",
  "Meta",
  "OS",
  "Shift",
]);

/** Convert a browser keydown into the frozen Ctrl+Alt+Shift+Super+Key format. */
export function canonicalCombo(event: KeyboardEvent): string | null {
  if (event.repeat || event.isComposing || modifierOnly.has(event.key)) return null;
  if (!event.ctrlKey && !event.altKey && !event.shiftKey && !event.metaKey) return null;

  let key: string;
  if (event.code.startsWith("Key")) key = event.code.slice(3);
  else if (event.code.startsWith("Digit")) key = event.code.slice(5);
  else if (event.code === "Space" || event.key === " ") key = "Space";
  else if (/^F(?:[1-9]|1[0-9]|2[0-4])$/.test(event.key)) key = event.key;
  else if (event.key.length === 1) key = event.key.toUpperCase();
  else key = event.key;

  const parts: string[] = [];
  if (event.ctrlKey) parts.push("Ctrl");
  if (event.altKey) parts.push("Alt");
  if (event.shiftKey) parts.push("Shift");
  if (event.metaKey) parts.push("Super");
  parts.push(key);
  return parts.join("+");
}

const Shortcuts: Component = () => {
  const [snapshot, setSnapshot] = createSignal<ShortcutSnapshot | null>(null);
  const [loadError, setLoadError] = createSignal(false);
  const [recordingAction, setRecordingAction] = createSignal<ShortcutAction | null>(null);
  const [recordedCombo, setRecordedCombo] = createSignal("");
  const [conflict, setConflict] = createSignal<ShortcutConflictState | null>(null);
  const [busy, setBusy] = createSignal<ShortcutBusy>("load");
  const [resetOpen, setResetOpen] = createSignal(false);
  const [localFailures, setLocalFailures] = createSignal<
    Partial<Record<ShortcutAction, boolean>>
  >({});
  const [operationError, setOperationError] = createSignal<"save" | "reset" | null>(null);

  const changeRefs: Partial<Record<ShortcutAction, HTMLButtonElement>> = {};
  const recorderRef: { current?: HTMLButtonElement } = {};
  const resetTriggerRef: { current?: HTMLElement } = {};
  let flowEpoch = 0;
  let nativeRecording = false;

  const load = async () => {
    const epoch = ++flowEpoch;
    setBusy("load");
    setLoadError(false);
    setOperationError(null);
    try {
      const value = await shortcutList();
      if (epoch !== flowEpoch) return;
      setSnapshot(value);
    } catch {
      if (epoch !== flowEpoch) return;
      setLoadError(true);
    } finally {
      if (epoch === flowEpoch) setBusy(null);
    }
  };

  onMount(() => void load());
  onCleanup(() => {
    flowEpoch += 1;
    if (nativeRecording) void shortcutRecordingEnd().catch(() => {});
    nativeRecording = false;
  });

  const endNativeRecording = () => {
    if (!nativeRecording) return;
    nativeRecording = false;
    void shortcutRecordingEnd().catch(() => {});
  };

  const restoreChangeFocus = (action: ShortcutAction) => {
    queueMicrotask(() => changeRefs[action]?.focus());
  };

  const closeRecording = (action: ShortcutAction, restoreFocus = true) => {
    endNativeRecording();
    setRecordingAction(null);
    setRecordedCombo("");
    setConflict(null);
    setBusy(null);
    if (restoreFocus) restoreChangeFocus(action);
  };

  const beginRecording = async (action: ShortcutAction) => {
    const entry = snapshot()?.entries.find((item) => item.action === action);
    if (!entry?.available || busy() !== null) return;
    const epoch = ++flowEpoch;
    setOperationError(null);
    setLocalFailures((prev) => ({ ...prev, [action]: false }));
    setBusy("recording");
    try {
      await shortcutRecordingBegin(action);
      if (epoch !== flowEpoch) {
        void shortcutRecordingEnd().catch(() => {});
        return;
      }
      nativeRecording = true;
      setRecordingAction(action);
      setRecordedCombo("");
      setConflict(null);
      setBusy(null);
      queueMicrotask(() => recorderRef.current?.focus());
    } catch {
      if (epoch !== flowEpoch) return;
      setBusy(null);
      setOperationError("save");
      restoreChangeFocus(action);
    }
  };

  const cancelRecording = () => {
    const action = recordingAction();
    if (!action) return;
    flowEpoch += 1;
    closeRecording(action);
  };

  const saveShortcut = async (
    action: ShortcutAction,
    combo: string,
    epoch: number,
    overrideAction?: ShortcutAction,
  ) => {
    const current = snapshot();
    if (!current) return;
    setBusy("save");
    try {
      const next = await shortcutSave(action, combo, current.revision, overrideAction);
      if (epoch !== flowEpoch) return;
      setSnapshot(next);
      setLocalFailures((prev) => ({ ...prev, [action]: false }));
      closeRecording(action);
    } catch (error) {
      if (epoch !== flowEpoch) return;
      if (isRegistrationFailure(error)) {
        setLocalFailures((prev) => ({ ...prev, [action]: true }));
      } else if (isStaleShortcutRevision(error)) {
        closeRecording(action);
        void load();
        return;
      } else {
        setOperationError("save");
      }
      // The backend rollback contract guarantees the snapshot's old mapping is
      // still authoritative after any failure.
      closeRecording(action);
    }
  };

  const handleRecorderKeyDown = (event: KeyboardEvent) => {
    const action = recordingAction();
    const current = snapshot();
    if (!action || !current || busy() !== null) return;
    event.preventDefault();
    event.stopPropagation();
    if (event.key === "Escape") {
      cancelRecording();
      return;
    }
    const combo = canonicalCombo(event);
    if (!combo) return;

    const currentMap = mapFromSnapshot(current);
    if (currentMap[action] === combo) {
      cancelRecording();
      return;
    }
    setRecordedCombo(combo);
    setConflict(null);
    const epoch = ++flowEpoch;
    setBusy("recording");
    void shortcutCheckConflict(action, combo, current.revision)
      .then((otherAction) => {
        if (epoch !== flowEpoch) return;
        setBusy(null);
        if (otherAction && otherAction !== action) {
          setConflict({ action, otherAction, combo });
          return;
        }
        return saveShortcut(action, combo, epoch);
      })
      .catch(() => {
        if (epoch !== flowEpoch) return;
        setOperationError("save");
        closeRecording(action);
      });
  };

  const overrideConflict = () => {
    const value = conflict();
    const current = snapshot();
    if (!value || !current || busy() !== null) return;
    const epoch = ++flowEpoch;
    void saveShortcut(value.action, value.combo, epoch, value.otherAction);
  };

  const resetDefaults = async () => {
    const current = snapshot();
    if (!current || busy() !== null) return;
    const epoch = ++flowEpoch;
    setBusy("reset");
    setOperationError(null);
    try {
      const next = await shortcutResetDefaults(current.revision);
      if (epoch !== flowEpoch) return;
      setSnapshot(next);
      setLocalFailures({});
      setResetOpen(false);
      queueMicrotask(() => resetTriggerRef.current?.focus());
    } catch (error) {
      if (epoch !== flowEpoch) return;
      setResetOpen(false);
      if (isStaleShortcutRevision(error)) {
        setBusy(null);
        void load();
        return;
      }
      setOperationError("reset");
    } finally {
      if (epoch === flowEpoch) setBusy(null);
    }
  };

  return (
    <ShortcutsView
      snapshot={snapshot()}
      loadError={loadError()}
      recordingAction={recordingAction()}
      recordedCombo={recordedCombo()}
      conflict={conflict()}
      busy={busy()}
      resetOpen={resetOpen()}
      localRegistrationFailures={localFailures()}
      operationError={operationError()}
      onRetryLoad={() => void load()}
      onChange={(action) => void beginRecording(action)}
      onCancelRecording={cancelRecording}
      onRecorderKeyDown={handleRecorderKeyDown}
      onOverride={overrideConflict}
      onOpenReset={() => setResetOpen(true)}
      onCloseReset={() => setResetOpen(false)}
      onReset={() => void resetDefaults()}
      changeRefs={changeRefs}
      recorderRef={recorderRef}
      resetTriggerRef={resetTriggerRef}
    />
  );
};

export default Shortcuts;
