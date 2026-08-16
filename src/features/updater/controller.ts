/** Updater controller — real check on mount, download/install, progress events. */
import { useCallback, useEffect, useRef, useState } from "react";
import { detectLocale } from "../../app/i18n";
import * as ipc from "./ipc";
import { updaterErrorText } from "./copy";
import { applyCheck, applyFailure, applyInstallDone, applyProgress, type UpdaterPhase } from "./model";

export function useUpdaterController() {
  const locale = detectLocale();
  const [phase, setPhase] = useState<UpdaterPhase>({ kind: "checking" });
  const [autoCheck, setAutoCheck] = useState(true);
  const [autoCheckError, setAutoCheckError] = useState<string | null>(null);
  const phaseRef = useRef(phase);
  phaseRef.current = phase;
  const cancelledRef = useRef(false);

  const runCheck = useCallback(async () => {
    setPhase({ kind: "checking" });
    phaseRef.current = { kind: "checking" };
    try {
      const check = await ipc.updaterCheck();
      if (cancelledRef.current) return;
      setPhase((p) => applyCheck(p, check));
    } catch (e) {
      if (cancelledRef.current) return;
      setPhase((p) => applyFailure(p, String(e)));
    }
  }, []);

  const startInstall = useCallback(async () => {
    setPhase((p) =>
      p.kind === "available"
        ? { kind: "downloading", update: p.update, percent: null, downloaded: 0 }
        : p,
    );
    try {
      // On Windows this promise never resolves — the NSIS installer exits the
      // process mid-call; only the updater-progress events arrive.
      const check = await ipc.updaterDownloadInstall();
      if (cancelledRef.current) return;
      setPhase((p) => applyInstallDone(p, check));
    } catch (e) {
      if (cancelledRef.current) return;
      setPhase((p) => applyFailure(p, String(e)));
    }
  }, []);

  const toggleAutoCheck = useCallback(async (enabled: boolean) => {
    setAutoCheck((prev) => {
      // Remember prev inside the updater for the revert path below.
      autoCheckPrevRef.current = prev;
      return enabled;
    });
    setAutoCheckError(null);
    try {
      await ipc.setUpdaterStartupCheck(enabled);
    } catch (e) {
      // Revert so the checkbox never shows a state the store rejected.
      setAutoCheck(autoCheckPrevRef.current);
      setAutoCheckError(updaterErrorText(locale, e));
    }
  }, [locale]);
  const autoCheckPrevRef = useRef(true);

  useEffect(() => {
    cancelledRef.current = false;
    void runCheck();
    void ipc.getUpdaterStartupCheck().then((v) => {
      if (!cancelledRef.current) setAutoCheck(v);
    });
    let unlisten: (() => void) | undefined;
    let done = false;
    ipc
      .onUpdaterProgress((p) => {
        if (cancelledRef.current) return;
        setPhase((prev) => applyProgress(prev, p));
      })
      .then((u) => {
        if (done || cancelledRef.current) u();
        else unlisten = u;
      });
    return () => {
      done = true;
      cancelledRef.current = true;
      unlisten?.();
    };
  }, [runCheck]);

  return {
    phase,
    autoCheck,
    autoCheckError,
    check: () => void runCheck(),
    install: () => void startInstall(),
    relaunch: () => void ipc.relaunchApp(),
    toggleAutoCheck: (enabled: boolean) => void toggleAutoCheck(enabled),
  };
}

export type UpdaterController = ReturnType<typeof useUpdaterController>;
