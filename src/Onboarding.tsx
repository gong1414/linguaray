/**
 * First-launch onboarding. The container owns ALL IPC + real state
 * (persisted step, a11y / Screen-Recording permission status, provider
 * count, default shortcuts) and hands it to the presentational
 * `OnboardingView` (ui-lab + direct unit tests render the view with
 * fixtures). Design system: token CSS + the shared `Button` — the same
 * product language as SettingsShell.
 */
import {
  createEffect,
  createSignal,
  onCleanup,
  onMount,
  Show,
  type Component,
} from "solid-js";
import { invoke } from "./bridge/invoke";
import { getCurrentWindow } from "./bridge/window";
import { Button } from "@linguaray/ui";
import { detectLocale, type Locale } from "./i18n";
import { ONBOARDING_COPY, type OnboardingStepName } from "./onboarding-copy";
import "./Onboarding.css";

export type { OnboardingStepName } from "./onboarding-copy";

/** null = still loading; "unsupported" = platform has no such permission. */
export type PermissionState = "checking" | "granted" | "denied" | "error" | "unsupported";

export type ShortcutCombo = { action: string; combo: string };

export type OnboardingViewProps = {
  step: OnboardingStepName;
  locale: Locale;
  a11y: PermissionState;
  screenCapture: PermissionState;
  /** null = still checking. */
  providerCount: number | null;
  historyBusy: boolean;
  shortcuts: ShortcutCombo[];
  advancing: boolean;
  error: string | null;
  onOpenA11ySettings: () => void;
  onOpenScreenCaptureSettings: () => void;
  onRecheckPermissions: () => void;
  onOpenProviderSettings: () => void;
  onOpenShortcutsSettings: () => void;
  onEnableHistory: () => void;
  onAdvance: (event: "start" | "continue" | "skip" | "complete") => void;
  onFinish: (openSettings: boolean) => void;
};

const STEP_ORDER: OnboardingStepName[] = [
  "welcome",
  "accessibility",
  "provider",
  "history",
  "shortcuts",
  "done",
];

const StatusBadge: Component<{ state: PermissionState; labels: Record<string, string> }> = (
  props,
) => (
  <span class="onboarding__badge" data-state={props.state} role="status">
    {props.labels[props.state] ?? props.state}
  </span>
);

export const OnboardingView: Component<OnboardingViewProps> = (props) => {
  const t = ONBOARDING_COPY[props.locale];
  let titleRef: HTMLHeadingElement | undefined;
  // Keyboard + screen-reader order: each step change moves focus to the new
  // step's heading (the container drives step; the view owns the ref).
  createEffect(() => {
    props.step;
    queueMicrotask(() => titleRef?.focus());
  });

  const stepIndex = () => STEP_ORDER.indexOf(props.step);

  return (
    <main class="onboarding" data-step={props.step} data-testid="onboarding">
      <header class="onboarding__brand">
        <span class="onboarding__logo" aria-hidden="true" />
        <span class="onboarding__brand-name">{t.brand}</span>
      </header>

      <div class="onboarding__content">
        <h1 class="onboarding__title" ref={titleRef} tabindex="-1" data-testid="onboarding-title">
          {props.step === "welcome"
            ? t.welcome.title
            : props.step === "done"
              ? t.done.title
              : props.step === "accessibility"
                ? t.a11y.title
                : props.step === "provider"
                  ? t.provider.title
                  : props.step === "history"
                    ? t.history.title
                    : t.shortcuts.title}
        </h1>

        <Show when={props.step === "welcome"}>
          <p class="onboarding__desc">{t.welcome.body}</p>
        </Show>

        <Show when={props.step === "accessibility"}>
          <p class="onboarding__desc">{t.a11y.body}</p>
          <div class="onboarding__card">
            <div class="onboarding__card-row">
              <span>{t.a11y.title}</span>
              <StatusBadge state={props.a11y} labels={t.a11y.status as Record<string, string>} />
            </div>
            <div class="onboarding__card-actions">
              <Button
                variant="ghost"
                onClick={props.onOpenA11ySettings}
                disabled={props.a11y === "unsupported"}
              >
                {t.a11y.openSettings}
              </Button>
            </div>
          </div>
          <div class="onboarding__card">
            <div class="onboarding__card-row">
              <span>{t.a11y.screenTitle}</span>
              <StatusBadge
                state={props.screenCapture}
                labels={t.a11y.status as Record<string, string>}
              />
            </div>
            <p class="onboarding__card-hint">{t.a11y.screenBody}</p>
            <div class="onboarding__card-actions">
              <Button
                variant="ghost"
                onClick={props.onOpenScreenCaptureSettings}
                disabled={props.screenCapture === "unsupported"}
              >
                {t.a11y.openScreenSettings}
              </Button>
            </div>
          </div>
          <button
            type="button"
            class="onboarding__link"
            onClick={props.onRecheckPermissions}
            disabled={props.advancing}
          >
            {t.a11y.recheck}
          </button>
        </Show>

        <Show when={props.step === "provider"}>
          <p class="onboarding__desc">{t.provider.body}</p>
          <div class="onboarding__card">
            <p class="onboarding__card-hint">
              {props.providerCount === null
                ? t.provider.checking
                : props.providerCount === 0
                  ? t.provider.noneBody
                  : t.provider.count(props.providerCount)}
            </p>
            <div class="onboarding__card-actions">
              <Button variant="ghost" onClick={props.onOpenProviderSettings}>
                {t.provider.openSettings}
              </Button>
            </div>
          </div>
        </Show>

        <Show when={props.step === "history"}>
          <p class="onboarding__desc">{t.history.body}</p>
        </Show>

        <Show when={props.step === "shortcuts"}>
          <p class="onboarding__desc">{t.shortcuts.body}</p>
          <ul class="onboarding__shortcuts" data-testid="onboarding-shortcuts">
            {props.shortcuts.map((s) => (
              <li>
                <span>{t.shortcuts.combos[s.action] ?? s.action}</span>
                <kbd>{s.combo}</kbd>
              </li>
            ))}
          </ul>
          <button type="button" class="onboarding__link" onClick={props.onOpenShortcutsSettings}>
            {t.shortcuts.openSettings}
          </button>
        </Show>

        <Show when={props.step === "done"}>
          <p class="onboarding__desc">{t.done.body}</p>
        </Show>

        <Show when={props.error}>
          {(msg) => (
            <p class="onboarding__error" role="alert" data-testid="onboarding-error">
              {t.errorPrefix}: {msg()}
            </p>
          )}
        </Show>
      </div>

      <footer class="onboarding__footer">
        <ol class="onboarding__progress" aria-label="Progress">
          {STEP_ORDER.map((s, i) => (
            <li
              class="onboarding__progress-dot"
              data-active={i === stepIndex()}
              data-past={i < stepIndex()}
              aria-current={i === stepIndex() ? "step" : undefined}
              title={t.stepLabels[s]}
            />
          ))}
        </ol>
        <div class="onboarding__actions">
          <Show when={props.step === "welcome"}>
            <Button onClick={() => props.onAdvance("start")}>{t.welcome.start}</Button>
          </Show>
          <Show when={props.step === "accessibility"}>
            <Button
              variant="ghost"
              onClick={() => props.onAdvance("skip")}
              disabled={props.advancing}
            >
              {t.a11y.later}
            </Button>
            <Button
              onClick={() => props.onAdvance("continue")}
              disabled={props.advancing || props.a11y === "checking"}
            >
              {t.a11y.continue}
            </Button>
          </Show>
          <Show when={props.step === "provider"}>
            <Button
              onClick={() => props.onAdvance("continue")}
              disabled={props.advancing}
            >
              {t.provider.continue}
            </Button>
          </Show>
          <Show when={props.step === "history"}>
            <Button
              variant="ghost"
              onClick={() => props.onAdvance("skip")}
              disabled={props.advancing || props.historyBusy}
            >
              {t.history.skip}
            </Button>
            <Button
              onClick={props.onEnableHistory}
              disabled={props.advancing || props.historyBusy}
            >
              {props.historyBusy ? t.history.enabling : t.history.enable}
            </Button>
          </Show>
          <Show when={props.step === "shortcuts"}>
            <Button onClick={() => props.onAdvance("complete")} disabled={props.advancing}>
              {t.shortcuts.continue}
            </Button>
          </Show>
          <Show when={props.step === "done"}>
            <Button variant="ghost" onClick={() => props.onFinish(false)}>
              {t.done.tray}
            </Button>
            <Button onClick={() => props.onFinish(true)}>{t.done.openApp}</Button>
          </Show>
        </div>
      </footer>
    </main>
  );
};

const Onboarding: Component = () => {
  const locale = detectLocale();
  const [step, setStep] = createSignal<OnboardingStepName>("welcome");
  const [a11y, setA11y] = createSignal<PermissionState>("checking");
  const [screenCapture, setScreenCapture] = createSignal<PermissionState>("checking");
  const [providerCount, setProviderCount] = createSignal<number | null>(null);
  const [historyBusy, setHistoryBusy] = createSignal(false);
  const [shortcuts, setShortcuts] = createSignal<ShortcutCombo[]>([]);
  const [advancing, setAdvancing] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  let cancelled = false;

  const isApple = typeof navigator !== "undefined" && /Mac/i.test(navigator.platform || navigator.userAgent);

  const refreshPermissions = async () => {
    setA11y("checking");
    setScreenCapture("checking");
    try {
      const granted = await invoke<boolean>("a11y_status");
      if (!cancelled) setA11y(granted ? "granted" : "denied");
    } catch {
      if (!cancelled) setA11y("error");
    }
    if (!isApple) {
      setScreenCapture("unsupported");
    } else {
      try {
        const granted = await invoke<boolean>("screen_capture_status");
        if (!cancelled) setScreenCapture(granted ? "granted" : "denied");
      } catch {
        if (!cancelled) setScreenCapture("error");
      }
    }
  };

  const refreshProviders = async () => {
    try {
      const list = await invoke<unknown[]>("provider_list");
      if (!cancelled) setProviderCount(Array.isArray(list) ? list.length : 0);
    } catch {
      if (!cancelled) setProviderCount(0);
    }
  };

  const refreshShortcuts = async () => {
    try {
      const snap = await invoke<{ entries?: { action: string; combo: string }[] }>(
        "shortcut_list",
      );
      if (!cancelled) setShortcuts(snap.entries ?? []);
    } catch {
      if (!cancelled) setShortcuts([]);
    }
  };

  onMount(() => {
    void (async () => {
      try {
        const status = await invoke<{ complete: boolean; step: OnboardingStepName }>(
          "onboarding_status",
        );
        if (!cancelled) setStep(status.step);
      } catch (e) {
        if (!cancelled) setError(String(e));
      }
    })();
    void refreshPermissions();
    // Re-check when the window regains focus: the user likely just toggled
    // the grants in System Settings (SettingsShell pattern).
    let unlisten: (() => void) | undefined;
    import("./bridge/window")
      .then(({ getCurrentWindow: gw }) =>
        gw().onFocusChanged(({ payload: focused }) => {
          if (focused && !cancelled) void refreshPermissions();
        }),
      )
      .then((u) => {
        if (cancelled) u();
        else unlisten = u;
      })
      .catch(() => {});
    onCleanup(() => {
      cancelled = true;
      unlisten?.();
    });
  });

  const advance = async (event: "start" | "continue" | "skip" | "complete") => {
    if (advancing()) return;
    setAdvancing(true);
    setError(null);
    try {
      const next = await invoke<OnboardingStepName>("onboarding_next", {
        step: step(),
        event,
      });
      if (!cancelled) setStep(next);
      if (next === "done") {
        // Complete only after the step write succeeded.
        await invoke("onboarding_complete");
      }
    } catch (e) {
      if (!cancelled) setError(String(e));
    } finally {
      if (!cancelled) setAdvancing(false);
    }
  };

  const enableHistory = async () => {
    if (historyBusy()) return;
    setHistoryBusy(true);
    setError(null);
    try {
      await invoke("history_set_enabled", { enabled: true });
      await advance("continue");
    } catch (e) {
      if (!cancelled) setError(String(e));
    } finally {
      if (!cancelled) setHistoryBusy(false);
    }
  };

  const openUrl = (url: string) =>
    import("./bridge/opener")
      .then(({ openUrl: open }) => open(url))
      .catch(() => {});

  const finish = async (openSettings: boolean) => {
    setError(null);
    try {
      await invoke("onboarding_complete");
      if (openSettings) {
        await invoke("open_settings_window", { section: "provider-center" });
      }
      await getCurrentWindow().hide();
    } catch (e) {
      setError(String(e));
    }
  };

  // Per-step data loads.
  createEffect(() => {
    if (step() === "provider") void refreshProviders();
    if (step() === "shortcuts") void refreshShortcuts();
  });
  // Provider count also refreshes when returning from the settings window.
  let providersUnlisten: (() => void) | undefined;
  import("./bridge/window")
    .then(({ getCurrentWindow: gw }) =>
      gw().onFocusChanged(({ payload: focused }) => {
        if (focused && !cancelled && step() === "provider") void refreshProviders();
      }),
    )
    .then((u) => {
      if (cancelled) u();
      else providersUnlisten = u;
    })
    .catch(() => {});
  onCleanup(() => providersUnlisten?.());

  return (
    <OnboardingView
      step={step()}
      locale={locale}
      a11y={a11y()}
      screenCapture={screenCapture()}
      providerCount={providerCount()}
      historyBusy={historyBusy()}
      shortcuts={shortcuts()}
      advancing={advancing()}
      error={error()}
      onOpenA11ySettings={() =>
        void openUrl(
          "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
        )
      }
      onOpenScreenCaptureSettings={() =>
        void openUrl(
          "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture",
        )
      }
      onRecheckPermissions={() => void refreshPermissions()}
      onOpenProviderSettings={() =>
        void invoke("open_settings_window", { section: "provider-center" }).catch((e) =>
          setError(String(e)),
        )
      }
      onOpenShortcutsSettings={() =>
        void invoke("open_settings_window", { section: "shortcuts" }).catch((e) =>
          setError(String(e)),
        )
      }
      onEnableHistory={() => void enableHistory()}
      onAdvance={(event) => void advance(event)}
      onFinish={(openSettings) => void finish(openSettings)}
    />
  );
};

export default Onboarding;
