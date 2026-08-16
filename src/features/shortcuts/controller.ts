/**
 * Shortcuts controller — the full record → conflict-check → save / override
 * flow, ported from the Solid container with its epoch discipline:
 *  - `flowEpoch` invalidates every in-flight response after any new flow start
 *  - native recording (recording_begin/end) is ALWAYS paired (cancel + unmount)
 *  - stale_revision reloads; registration_failed marks the row locally;
 *    the backend rollback contract keeps the old snapshot authoritative
 */
import { useCallback, useEffect, useRef, useState } from "react";
import * as ipc from "./ipc";
import {
  DEFAULT_SHORTCUT_MAP,
  SHORTCUT_ACTIONS,
  canonicalCombo,
  isRegistrationFailure,
  isStaleShortcutRevision,
  mapFromSnapshot,
  type ShortcutAction,
  type ShortcutConflictState,
  type ShortcutSnapshot,
} from "./model";

export type ShortcutBusy = "load" | "recording" | "save" | "reset" | null;

export function useShortcutsController() {
  const [snapshot, setSnapshot] = useState<ShortcutSnapshot | null>(null);
  const [loadError, setLoadError] = useState(false);
  const [recordingAction, setRecordingAction] = useState<ShortcutAction | null>(null);
  const [recordedCombo, setRecordedCombo] = useState("");
  const [conflict, setConflict] = useState<ShortcutConflictState | null>(null);
  const [busy, setBusy] = useState<ShortcutBusy>("load");
  const [resetOpen, setResetOpen] = useState(false);
  const [localFailures, setLocalFailures] = useState<Partial<Record<ShortcutAction, boolean>>>({});
  const [operationError, setOperationError] = useState<"save" | "reset" | null>(null);

  const ref = useRef({
    snapshot,
    busy,
    recordingAction,
    conflict,
    epoch: 0,
    nativeRecording: false,
    cancelled: false,
  });
  ref.current.snapshot = snapshot;
  ref.current.busy = busy;
  ref.current.recordingAction = recordingAction;
  ref.current.conflict = conflict;

  const endNativeRecording = useCallback(() => {
    if (!ref.current.nativeRecording) return;
    ref.current.nativeRecording = false;
    void ipc.shortcutRecordingEnd().catch(() => {});
  }, []);

  const load = useCallback(async () => {
    const epoch = ++ref.current.epoch;
    setBusy("load");
    setLoadError(false);
    setOperationError(null);
    try {
      const value = await ipc.shortcutList();
      if (epoch !== ref.current.epoch || ref.current.cancelled) return;
      setSnapshot(value);
    } catch {
      if (epoch !== ref.current.epoch || ref.current.cancelled) return;
      setLoadError(true);
    } finally {
      if (epoch === ref.current.epoch && !ref.current.cancelled) setBusy(null);
    }
  }, []);

  useEffect(() => {
    ref.current.cancelled = false;
    void load();
    return () => {
      ref.current.cancelled = true;
      ref.current.epoch += 1;
      if (ref.current.nativeRecording) void ipc.shortcutRecordingEnd().catch(() => {});
      ref.current.nativeRecording = false;
    };
  }, [load]);

  const closeRecording = useCallback(
    (action: ShortcutAction) => {
      endNativeRecording();
      setRecordingAction(null);
      setRecordedCombo("");
      setConflict(null);
      setBusy(null);
      // Focus restore: the Change button for this action (testid keyed).
      queueMicrotask(() => {
        document.querySelector<HTMLButtonElement>(`[data-change-action="${action}"]`)?.focus();
      });
    },
    [endNativeRecording],
  );

  const beginRecording = useCallback(
    async (action: ShortcutAction) => {
      const entry = ref.current.snapshot?.entries.find((item) => item.action === action);
      if (!entry?.available || ref.current.busy !== null) return;
      const epoch = ++ref.current.epoch;
      setOperationError(null);
      setLocalFailures((prev) => ({ ...prev, [action]: false }));
      setBusy("recording");
      try {
        await ipc.shortcutRecordingBegin(action);
        if (epoch !== ref.current.epoch) {
          void ipc.shortcutRecordingEnd().catch(() => {});
          return;
        }
        ref.current.nativeRecording = true;
        setRecordingAction(action);
        setRecordedCombo("");
        setConflict(null);
        setBusy(null);
        queueMicrotask(() => {
          document.querySelector<HTMLButtonElement>(`[data-recorder-action="${action}"]`)?.focus();
        });
      } catch {
        if (epoch !== ref.current.epoch) return;
        setBusy(null);
        setOperationError("save");
      }
    },
    [],
  );

  const cancelRecording = useCallback(() => {
    const action = ref.current.recordingAction;
    if (!action) return;
    ref.current.epoch += 1;
    closeRecording(action);
  }, [closeRecording]);

  const saveShortcut = useCallback(
    async (action: ShortcutAction, combo: string, epoch: number, overrideAction?: ShortcutAction) => {
      const current = ref.current.snapshot;
      if (!current) return;
      setBusy("save");
      try {
        const next = await ipc.shortcutSave(action, combo, current.revision, overrideAction);
        if (epoch !== ref.current.epoch || ref.current.cancelled) return;
        setSnapshot(next);
        setLocalFailures((prev) => ({ ...prev, [action]: false }));
        closeRecording(action);
      } catch (error) {
        if (epoch !== ref.current.epoch || ref.current.cancelled) return;
        if (isRegistrationFailure(error)) {
          setLocalFailures((prev) => ({ ...prev, [action]: true }));
        } else if (isStaleShortcutRevision(error)) {
          closeRecording(action);
          void load();
          return;
        } else {
          setOperationError("save");
        }
        // The backend rollback contract keeps the old snapshot authoritative.
        closeRecording(action);
      }
    },
    [closeRecording, load],
  );

  const handleRecorderKeyDown = useCallback(
    (event: React.KeyboardEvent<HTMLButtonElement>) => {
      const action = ref.current.recordingAction;
      const current = ref.current.snapshot;
      if (!action || !current || ref.current.busy !== null) return;
      event.preventDefault();
      event.stopPropagation();
      if (event.key === "Escape") {
        cancelRecording();
        return;
      }
      const combo = canonicalCombo(event.nativeEvent);
      if (!combo) return;

      const currentMap = mapFromSnapshot(current);
      if (currentMap[action] === combo) {
        cancelRecording();
        return;
      }
      setRecordedCombo(combo);
      setConflict(null);
      const epoch = ++ref.current.epoch;
      setBusy("recording");
      void ipc
        .shortcutCheckConflict(action, combo, current.revision)
        .then((otherAction) => {
          if (epoch !== ref.current.epoch || ref.current.cancelled) return;
          setBusy(null);
          if (otherAction && otherAction !== action) {
            setConflict({ action, otherAction, combo });
            return;
          }
          return saveShortcut(action, combo, epoch);
        })
        .catch(() => {
          if (epoch !== ref.current.epoch || ref.current.cancelled) return;
          setOperationError("save");
          closeRecording(action);
        });
    },
    [cancelRecording, closeRecording, saveShortcut],
  );

  const overrideConflict = useCallback(() => {
    const value = ref.current.conflict;
    const current = ref.current.snapshot;
    if (!value || !current || ref.current.busy !== null) return;
    const epoch = ++ref.current.epoch;
    void saveShortcut(value.action, value.combo, epoch, value.otherAction);
  }, [saveShortcut]);

  const resetDefaults = useCallback(async () => {
    const current = ref.current.snapshot;
    if (!current || ref.current.busy !== null) return;
    const epoch = ++ref.current.epoch;
    setBusy("reset");
    setOperationError(null);
    try {
      const next = await ipc.shortcutResetDefaults(current.revision);
      if (epoch !== ref.current.epoch || ref.current.cancelled) return;
      setSnapshot(next);
      setLocalFailures({});
      setResetOpen(false);
    } catch (error) {
      if (epoch !== ref.current.epoch || ref.current.cancelled) return;
      setResetOpen(false);
      if (isStaleShortcutRevision(error)) {
        setBusy(null);
        void load();
        return;
      }
      setOperationError("reset");
    } finally {
      if (epoch === ref.current.epoch && !ref.current.cancelled) setBusy(null);
    }
  }, [load]);

  const map = snapshot ? mapFromSnapshot(snapshot) : null;
  const differsFromDefaults =
    !!map && SHORTCUT_ACTIONS.some((action) => map[action] !== DEFAULT_SHORTCUT_MAP[action]);

  return {
    snapshot,
    loadError,
    recordingAction,
    recordedCombo,
    conflict,
    busy,
    resetOpen,
    localFailures,
    operationError,
    differsFromDefaults,
    retryLoad: () => void load(),
    change: (action: ShortcutAction) => void beginRecording(action),
    cancelRecording,
    onRecorderKeyDown: handleRecorderKeyDown,
    overrideConflict: () => overrideConflict(),
    openReset: () => setResetOpen(true),
    closeReset: () => setResetOpen(false),
    reset: () => void resetDefaults(),
  };
}

export type ShortcutsController = ReturnType<typeof useShortcutsController>;
