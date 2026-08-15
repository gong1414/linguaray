/**
 * Settings → Updater panel. The container runs a real check on mount (the
 * panel IS the "Check for Updates" tray destination), drives download/install
 * via the backend commands, and feeds the pure phase machine into the
 * exported presentational `UpdaterPanelView` (ui-lab renders the same view
 * with fixture phases).
 */
import { createSignal, onCleanup, onMount, Show, type Component } from "solid-js";
import { detectLocale } from "../../i18n";
import { UPDATER_COPY } from "./updater-copy";
import {
  getUpdaterStartupCheck,
  onUpdaterProgress,
  relaunchApp,
  setUpdaterStartupCheck,
  updaterCheck,
  updaterDownloadInstall,
} from "./updater-ipc";
import {
  applyCheck,
  applyFailure,
  applyInstallDone,
  applyProgress,
  type AvailableUpdate,
  type UpdaterPhase,
} from "./updater-types";
import "./UpdaterPanel.css";

export type UpdaterPanelViewProps = {
  phase: UpdaterPhase;
  autoCheck: boolean;
  autoCheckError: string | null;
  onCheck: () => void;
  onInstall: () => void;
  onRelaunch: () => void;
  onToggleAutoCheck: (enabled: boolean) => void;
};

export const UpdaterPanelView: Component<UpdaterPanelViewProps> = (props) => {
  const t = UPDATER_COPY[detectLocale()];

  /** The available update, when any install-related phase is active. Solid's
   *  reactive JSX re-runs these getters; the kind checks double as TS
   *  narrowing so each branch reads a typed `update`. */
  const availableUpdate = (): AvailableUpdate | null => {
    const p = props.phase;
    return p.kind === "available" ||
      p.kind === "downloading" ||
      p.kind === "installing" ||
      p.kind === "readyToRelaunch"
      ? p.update
      : null;
  };

  const knownVersion = (): string | null => {
    const p = props.phase;
    if (p.kind === "upToDate") return p.version;
    return availableUpdate()?.current ?? null;
  };

  const percent = (): number | null => {
    const p = props.phase;
    return p.kind === "downloading" ? p.percent : null;
  };

  const errorMessage = (): string => {
    const p = props.phase;
    return p.kind === "error" ? p.message : "";
  };

  const busy = (): boolean =>
    props.phase.kind === "downloading" || props.phase.kind === "installing";

  return (
    <section class="updater-panel" data-testid="updater-panel">
      <header class="updater-panel__header">
        <h1>{t.title}</h1>
        <Show when={knownVersion()}>
          {(v) => (
            <p class="updater-panel__version" data-testid="updater-current-version">
              {t.currentVersion}: {v()}
            </p>
          )}
        </Show>
      </header>

      <div class="updater-panel__panel" data-testid="updater-status">
        <Show when={props.phase.kind === "checking"}>{t.status.checking}</Show>
        <Show when={props.phase.kind === "upToDate"}>{t.status.upToDate}</Show>
        <Show when={props.phase.kind === "error"}>
          <p class="updater-panel__error" role="alert">
            {t.status.errorPrefix}: {errorMessage()}
          </p>
        </Show>

        <Show when={availableUpdate()}>
          {(u) => (
            <div class="updater-panel__available">
              <strong data-testid="updater-next">
                {t.status.available} {u().next}
              </strong>
              <Show when={u().notes}>
                <details class="updater-panel__notes">
                  <summary>{t.releaseNotes}</summary>
                  <pre>{u().notes}</pre>
                </details>
              </Show>
            </div>
          )}
        </Show>

        <Show when={props.phase.kind === "downloading"}>
          <div class="updater-panel__progress" data-testid="updater-progress">
            <Show when={percent() !== null} fallback={<span>{t.progress.unknownSize}</span>}>
              <span>
                {t.progress.downloading} {percent()}%
              </span>
            </Show>
            {/* No `value` attr while the total is unknown → the browser renders
                an indeterminate bar instead of a bogus 0%. */}
            <Show when={percent() !== null} fallback={<progress max="100" />}>
              <progress max="100" value={percent() ?? 0} />
            </Show>
          </div>
        </Show>
        <Show when={props.phase.kind === "installing"}>
          <p data-testid="updater-installing">{t.progress.installing}</p>
        </Show>
        <Show when={props.phase.kind === "readyToRelaunch"}>
          <p>{t.progress.installedHint}</p>
          <button
            type="button"
            class="updater-panel__primary"
            data-testid="updater-relaunch"
            onClick={() => props.onRelaunch()}
          >
            {t.action.relaunch}
          </button>
        </Show>

        <div class="updater-panel__actions">
          <button
            type="button"
            data-testid="updater-check-again"
            disabled={busy()}
            onClick={() => props.onCheck()}
          >
            {t.action.checkAgain}
          </button>
          <Show when={props.phase.kind === "available"}>
            <button
              type="button"
              class="updater-panel__primary"
              data-testid="updater-download"
              onClick={() => props.onInstall()}
            >
              {t.action.downloadInstall}
            </button>
          </Show>
        </div>
      </div>

      <div class="updater-panel__panel updater-panel__pref">
        <label class="updater-panel__toggle">
          <input
            type="checkbox"
            data-testid="updater-autocheck"
            checked={props.autoCheck}
            onChange={(e) => props.onToggleAutoCheck(e.currentTarget.checked)}
          />
          <span>
            <strong>{t.autoCheckLabel}</strong>
            <small>{t.autoCheckHint}</small>
          </span>
        </label>
        <Show when={props.autoCheckError}>
          {(msg) => (
            <p class="updater-panel__error" role="alert" data-testid="updater-autocheck-error">
              {msg()}
            </p>
          )}
        </Show>
      </div>
    </section>
  );
};

const UpdaterPanel: Component = () => {
  const [phase, setPhase] = createSignal<UpdaterPhase>({ kind: "checking" });
  const [autoCheck, setAutoCheck] = createSignal(true);
  const [autoCheckError, setAutoCheckError] = createSignal<string | null>(null);
  let unlisten: (() => void) | undefined;
  let cancelled = false;

  const runCheck = async () => {
    setPhase({ kind: "checking" });
    try {
      const check = await updaterCheck();
      if (cancelled) return;
      setPhase((p) => applyCheck(p, check));
    } catch (e) {
      if (cancelled) return;
      setPhase((p) => applyFailure(p, String(e)));
    }
  };

  const startInstall = async () => {
    setPhase((p) =>
      p.kind === "available"
        ? { kind: "downloading", update: p.update, percent: null, downloaded: 0 }
        : p,
    );
    try {
      // On Windows this promise never resolves — the NSIS installer exits the
      // process mid-call; only the updater-progress events arrive.
      const check = await updaterDownloadInstall();
      if (cancelled) return;
      setPhase((p) => applyInstallDone(p, check));
    } catch (e) {
      if (cancelled) return;
      setPhase((p) => applyFailure(p, String(e)));
    }
  };

  const toggleAutoCheck = async (enabled: boolean) => {
    const prev = autoCheck();
    setAutoCheck(enabled);
    setAutoCheckError(null);
    try {
      await setUpdaterStartupCheck(enabled);
    } catch (e) {
      // Revert on failure so the checkbox never shows a state the store
      // did not accept.
      setAutoCheck(prev);
      setAutoCheckError(String(e));
    }
  };

  onMount(() => {
    void runCheck();
    void getUpdaterStartupCheck().then((v) => {
      if (!cancelled) setAutoCheck(v);
    });
    // Race guard (SettingsShell pattern): a late-resolving listener after
    // teardown must be torn down immediately, not leaked.
    void onUpdaterProgress((p) => {
      setPhase((prev) => applyProgress(prev, p));
    }).then((u) => {
      if (cancelled) u();
      else unlisten = u;
    });
  });
  onCleanup(() => {
    cancelled = true;
    unlisten?.();
  });

  return (
    <UpdaterPanelView
      phase={phase()}
      autoCheck={autoCheck()}
      autoCheckError={autoCheckError()}
      onCheck={() => void runCheck()}
      onInstall={() => void startInstall()}
      onRelaunch={() => void relaunchApp()}
      onToggleAutoCheck={(enabled) => void toggleAutoCheck(enabled)}
    />
  );
};

export default UpdaterPanel;
