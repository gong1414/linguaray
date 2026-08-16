/**
 * Onboarding controller — owns ALL side effects (IPC via ./ipc, window +
 * opener via the bridge) and exposes a plain state + callbacks object for the
 * pure view. Mirrors the legacy Solid container behavior 1:1:
 *  - persisted step restored from onboarding_status
 *  - a11y / screen-capture permission status, re-checked on window focus
 *    (the user just toggled the grant in System Settings)
 *  - provider count refresh on focus while on the provider step
 *  - history enable = history_set_enabled THEN advance("continue")
 *  - finish = complete (+ optional settings window) THEN hide this window
 */
import { useCallback, useEffect, useRef, useState } from "react";
import { getCurrentWindow } from "../../bridge/window";
import { openUrl } from "../../bridge/opener";
import * as ipc from "./ipc";
import type {
  AdvanceEvent,
  OnboardingStepName,
  PermissionState,
  ShortcutCombo,
} from "./model";

export type OnboardingController = {
  step: OnboardingStepName;
  a11y: PermissionState;
  screenCapture: PermissionState;
  providerCount: number | null;
  historyBusy: boolean;
  shortcuts: ShortcutCombo[];
  advancing: boolean;
  error: string | null;
  openA11ySettings: () => void;
  openScreenCaptureSettings: () => void;
  recheckPermissions: () => void;
  openProviderSettings: () => void;
  openShortcutsSettings: () => void;
  enableHistory: () => void;
  advance: (event: AdvanceEvent) => void;
  finish: (openSettings: boolean) => void;
};

export function useOnboardingController(): OnboardingController {
  const [step, setStep] = useState<OnboardingStepName>("welcome");
  const [a11y, setA11y] = useState<PermissionState>("checking");
  const [screenCapture, setScreenCapture] = useState<PermissionState>("checking");
  const [providerCount, setProviderCount] = useState<number | null>(null);
  const [historyBusy, setHistoryBusy] = useState(false);
  const [shortcuts, setShortcuts] = useState<ShortcutCombo[]>([]);
  const [advancing, setAdvancing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Like the legacy `cancelled` flag: guards setState after unmount across
  // the async loads below.
  const cancelledRef = useRef(false);
  const stepRef = useRef(step);
  stepRef.current = step;

  const isApple =
    typeof navigator !== "undefined" && /Mac/i.test(navigator.platform || navigator.userAgent);

  const refreshPermissions = useCallback(async () => {
    setA11y("checking");
    setScreenCapture("checking");
    try {
      const granted = await ipc.a11yStatus();
      if (!cancelledRef.current) setA11y(granted ? "granted" : "denied");
    } catch {
      if (!cancelledRef.current) setA11y("error");
    }
    if (!isApple) {
      setScreenCapture("unsupported");
    } else {
      try {
        const granted = await ipc.screenCaptureStatus();
        if (!cancelledRef.current) setScreenCapture(granted ? "granted" : "denied");
      } catch {
        if (!cancelledRef.current) setScreenCapture("error");
      }
    }
  }, [isApple]);

  const refreshProviders = useCallback(async () => {
    try {
      const list = await ipc.listProviders();
      if (!cancelledRef.current) setProviderCount(Array.isArray(list) ? list.length : 0);
    } catch {
      if (!cancelledRef.current) setProviderCount(0);
    }
  }, []);

  const refreshShortcuts = useCallback(async () => {
    try {
      const snap = await ipc.listShortcuts();
      if (!cancelledRef.current) setShortcuts(snap.entries ?? []);
    } catch {
      if (!cancelledRef.current) setShortcuts([]);
    }
  }, []);

  useEffect(() => {
    cancelledRef.current = false;
    void (async () => {
      try {
        const status = await ipc.getOnboardingStatus();
        if (!cancelledRef.current) setStep(status.step);
      } catch (e) {
        if (!cancelledRef.current) setError(String(e));
      }
    })();
    void refreshPermissions();

    // Focus re-checks: the user likely just toggled a grant in System
    // Settings. A failed registration (non-Tauri/test host) is swallowed —
    // onboarding stays usable, just without auto-refresh.
    let unlisten: (() => void) | undefined;
    let unlistenProviders: (() => void) | undefined;
    let done = false;
    getCurrentWindow()
      .onFocusChanged(({ payload: focused }) => {
        if (focused && !cancelledRef.current) void refreshPermissions();
      })
      .then((u) => {
        if (done || cancelledRef.current) u();
        else unlisten = u;
      })
      .catch(() => {});
    getCurrentWindow()
      .onFocusChanged(({ payload: focused }) => {
        if (focused && !cancelledRef.current && stepRef.current === "provider") {
          void refreshProviders();
        }
      })
      .then((u) => {
        if (done || cancelledRef.current) u();
        else unlistenProviders = u;
      })
      .catch(() => {});
    return () => {
      done = true;
      cancelledRef.current = true;
      unlisten?.();
      unlistenProviders?.();
    };
  }, [refreshPermissions, refreshProviders]);

  // Per-step data loads (mirrors the Solid createEffect).
  useEffect(() => {
    if (step === "provider") void refreshProviders();
    if (step === "shortcuts") void refreshShortcuts();
  }, [step, refreshProviders, refreshShortcuts]);

  const advance = useCallback(async (event: AdvanceEvent) => {
    if (advancing) return;
    setAdvancing(true);
    setError(null);
    try {
      const next = await ipc.onboardingNext(stepRef.current, event);
      if (!cancelledRef.current) setStep(next);
      if (next === "done") {
        // Complete only after the step write succeeded.
        await ipc.completeOnboarding();
      }
    } catch (e) {
      if (!cancelledRef.current) setError(String(e));
    } finally {
      if (!cancelledRef.current) setAdvancing(false);
    }
  }, [advancing]);

  const enableHistory = useCallback(async () => {
    if (historyBusy) return;
    setHistoryBusy(true);
    setError(null);
    try {
      await ipc.setHistoryEnabled(true);
      await advance("continue");
    } catch (e) {
      if (!cancelledRef.current) setError(String(e));
    } finally {
      if (!cancelledRef.current) setHistoryBusy(false);
    }
  }, [historyBusy, advance]);

  const openExternal = useCallback((url: string) => {
    openUrl(url).catch(() => {});
  }, []);

  const openSettings = useCallback(async (section: string) => {
    try {
      await ipc.openSettingsSection(section);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  const finish = useCallback(async (openSettingsFirst: boolean) => {
    setError(null);
    try {
      await ipc.completeOnboarding();
      if (openSettingsFirst) {
        await ipc.openSettingsSection("provider-center");
      }
      await getCurrentWindow().hide();
    } catch (e) {
      setError(String(e));
    }
  }, []);

  return {
    step,
    a11y,
    screenCapture,
    providerCount,
    historyBusy,
    shortcuts,
    advancing,
    error,
    openA11ySettings: () =>
      openExternal(
        "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
      ),
    openScreenCaptureSettings: () =>
      openExternal(
        "x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture",
      ),
    recheckPermissions: () => void refreshPermissions(),
    openProviderSettings: () => void openSettings("provider-center"),
    openShortcutsSettings: () => void openSettings("shortcuts"),
    enableHistory: () => void enableHistory(),
    advance: (event) => void advance(event),
    finish: (open) => void finish(open),
  };
}
