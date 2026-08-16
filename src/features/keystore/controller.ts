/**
 * Keystore Recovery controller. Four visual states:
 * healthy (no banner) / corrupt (destructive banner: archive + reset) /
 * archived (info banner) / reset-confirm (transient dialog state).
 * A thrown keystore_health is itself a corrupt signal (fail-closed).
 */
import { useCallback, useEffect, useRef, useState } from "react";
import { detectLocale } from "../../app/i18n";
import { KEYSTORE_COPY } from "./copy";
import type { KsState, KsToastEntry } from "./copy";
import * as ipc from "./ipc";

export function useKeystoreController() {
  const t = KEYSTORE_COPY[detectLocale()];
  const [state, setState] = useState<KsState>("healthy");
  const [reason, setReason] = useState("");
  const [resetOpen, setResetOpen] = useState(false);
  const [busy, setBusy] = useState<"archive" | "reset" | null>(null);
  const [toasts, setToasts] = useState<KsToastEntry[]>([]);
  const cancelledRef = useRef(false);
  const toastSeqRef = useRef(0);

  useEffect(() => {
    cancelledRef.current = false;
    ipc
      .keystoreHealth()
      .then((h) => {
        if (cancelledRef.current) return;
        if (h === "" || h == null) setState("healthy");
        else {
          setState("corrupt");
          setReason(h);
        }
      })
      .catch((e: unknown) => {
        if (cancelledRef.current) return;
        setState("corrupt");
        setReason(String(e));
      });
    return () => {
      cancelledRef.current = true;
      setToasts([]);
    };
  }, []);

  const pushToast = useCallback((variant: KsToastEntry["variant"], message: string) => {
    setToasts((prev) => [...prev, { id: ++toastSeqRef.current, variant, message }]);
  }, []);

  const onArchive = useCallback(async () => {
    setBusy("archive");
    try {
      await ipc.archiveKeystore();
      if (!cancelledRef.current) setState("archived");
    } catch (e) {
      pushToast("destructive", `${t.archiveFailed}: ${String(e)}`);
    } finally {
      if (!cancelledRef.current) setBusy(null);
    }
  }, [pushToast, t.archiveFailed]);

  const onReset = useCallback(async () => {
    setBusy("reset");
    try {
      await ipc.resetKeystore();
      if (cancelledRef.current) return;
      setState("archived");
      setResetOpen(false);
    } catch (e) {
      pushToast("destructive", `${t.resetFailed}: ${String(e)}`);
    } finally {
      if (!cancelledRef.current) setBusy(null);
    }
  }, [pushToast, t.resetFailed]);

  return {
    state,
    reason,
    resetOpen,
    busy,
    toasts,
    archive: () => void onArchive(),
    reset: () => void onReset(),
    openReset: () => setResetOpen(true),
    closeReset: () => setResetOpen(false),
    dismissToast: (id: number) => setToasts((prev) => prev.filter((x) => x.id !== id)),
  };
}

export type KeystoreController = ReturnType<typeof useKeystoreController>;
