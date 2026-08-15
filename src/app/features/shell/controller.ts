/**
 * Settings shell controller — owns active-section state (uncontrolled or
 * parent-controlled like the Solid rev-9-2 shell), the macOS Accessibility
 * banner state, and its focus re-check (the user just toggled the grant in
 * System Settings, so regaining focus re-runs a11y_status).
 */
import { useCallback, useEffect, useRef, useState } from "react";
import { openUrl } from "../../../bridge/opener";
import { a11yStatus } from "./ipc";
import type { SettingsSection } from "./model";

export type ShellController = {
  active: SettingsSection;
  setActive: (section: SettingsSection) => void;
  a11yGranted: boolean | null;
  recheckA11y: () => void;
  openSystemSettings: () => void;
};

export function useShellController(options?: {
  initialSection?: SettingsSection;
  activePage?: SettingsSection;
  onNavigate?: (section: SettingsSection) => void;
}): ShellController {
  const [internalActive, setInternalActive] = useState<SettingsSection>(
    options?.initialSection ?? "provider-center",
  );
  const [a11yGranted, setA11yGranted] = useState<boolean | null>(null);
  const cancelledRef = useRef(false);

  const active = options?.activePage ?? internalActive;

  const recheckA11y = useCallback(async () => {
    try {
      const granted = await a11yStatus();
      if (!cancelledRef.current) setA11yGranted(granted);
    } catch {
      // Non-Tauri host or missing command: keep the banner hidden rather
      // than blocking the shell (legacy behavior).
      if (!cancelledRef.current) setA11yGranted(true);
    }
  }, []);

  useEffect(() => {
    cancelledRef.current = false;
    void recheckA11y();
    // Focus re-check. Registration failures (tests/non-Tauri) are swallowed;
    // a listener arriving after unmount is torn down via the done flag.
    let unlisten: (() => void) | undefined;
    let done = false;
    let cancelled = false;
    import("../../../bridge/window")
      .then(({ getCurrentWindow }) =>
        getCurrentWindow().onFocusChanged(({ payload: focused }) => {
          if (focused && !cancelled) void recheckA11y();
        }),
      )
      .then((u) => {
        if (done || cancelled) u();
        else unlisten = u;
      })
      .catch(() => {});
    return () => {
      done = true;
      cancelled = true;
      cancelledRef.current = true;
      unlisten?.();
    };
  }, [recheckA11y]);

  const setActive = useCallback(
    (section: SettingsSection) => {
      if (options?.activePage === undefined) setInternalActive(section);
      options?.onNavigate?.(section);
    },
    [options],
  );

  const openSystemSettings = useCallback(() => {
    openUrl(
      "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
    ).catch(() => {});
  }, []);

  return { active, setActive, a11yGranted, recheckA11y: () => void recheckA11y(), openSystemSettings };
}
