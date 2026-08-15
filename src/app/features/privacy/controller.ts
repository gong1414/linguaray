/**
 * Privacy controller — owns IPC + external-API lifecycle + toasts. All
 * mutations are epoch-guarded (a stale response never overwrites newer
 * state) and single-flight (busy lock), mirroring the Solid container.
 * Toast text is localized here (from ./copy) so no raw error codes or Rust
 * strings reach the view.
 */
import { useCallback, useEffect, useRef, useState } from "react";
import { writeText } from "../../../bridge/clipboard";
import { detectLocale } from "../../i18n";
import { PRIVACY_COPY } from "./copy";
import * as ipc from "./ipc";
import type {
  ExternalApiStatus,
  HistoryPrivacyStatus,
  HistoryRetentionDays,
  PrivacyBusy,
} from "./model";

export type PrivacyToast = { id: number; variant: "success" | "danger"; message: string };

export type PrivacyController = {
  status: HistoryPrivacyStatus | null;
  loading: boolean;
  error: string | null;
  busy: PrivacyBusy;
  clearOpen: boolean;
  toasts: PrivacyToast[];
  external: ExternalApiStatus | null;
  externalBusy: boolean;
  /** Non-empty only immediately after enable/regenerate — shown ONCE. */
  tokenOnce: string | null;
  tokenCopied: boolean;
  retry: () => void;
  setEnabled: (enabled: boolean) => void;
  setRetention: (days: HistoryRetentionDays) => void;
  openClear: () => void;
  closeClear: () => void;
  confirmClear: () => void;
  enableExternal: () => void;
  disableExternal: () => void;
  regenToken: () => void;
  copyToken: () => void;
  dismissToast: (id: number) => void;
};

export function usePrivacyController(): PrivacyController {
  const t = PRIVACY_COPY[detectLocale()];
  const [status, setStatus] = useState<HistoryPrivacyStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState<PrivacyBusy>(null);
  const [clearOpen, setClearOpen] = useState(false);
  const [toasts, setToasts] = useState<PrivacyToast[]>([]);
  const [external, setExternal] = useState<ExternalApiStatus | null>(null);
  const [externalBusy, setExternalBusy] = useState(false);
  const [tokenOnce, setTokenOnce] = useState<string | null>(null);
  const [tokenCopied, setTokenCopied] = useState(false);

  const epochRef = useRef(0);
  const busyRef = useRef<PrivacyBusy>(null);
  const toastIdRef = useRef(0);
  const cancelledRef = useRef(false);

  const pushToast = useCallback((variant: PrivacyToast["variant"], message: string) => {
    setToasts((items) => [...items, { id: ++toastIdRef.current, variant, message }]);
  }, []);

  const load = useCallback(async () => {
    const epoch = ++epochRef.current;
    setLoading(true);
    setError(null);
    try {
      const next = await ipc.historyPrivacyStatus();
      if (epoch === epochRef.current && !cancelledRef.current) setStatus(next);
    } catch (reason) {
      if (epoch === epochRef.current && !cancelledRef.current) {
        setError(reason instanceof Error ? reason.message : String(reason));
      }
    } finally {
      if (epoch === epochRef.current && !cancelledRef.current) setLoading(false);
    }
  }, []);

  const refreshExternal = useCallback(async () => {
    try {
      const s = await ipc.externalApiStatus();
      if (!cancelledRef.current) setExternal(s);
    } catch {
      if (!cancelledRef.current) setExternal({ state: "disabled" });
    }
  }, []);

  useEffect(() => {
    cancelledRef.current = false;
    void load();
    void refreshExternal();
    return () => {
      cancelledRef.current = true;
    };
  }, [load, refreshExternal]);

  const mutate = useCallback(
    async (kind: Exclude<PrivacyBusy, null>, operation: () => Promise<HistoryPrivacyStatus>) => {
      if (busyRef.current !== null) return;
      const epoch = ++epochRef.current;
      busyRef.current = kind;
      setBusy(kind);
      try {
        const next = await operation();
        if (epoch === epochRef.current && !cancelledRef.current) setStatus(next);
      } catch {
        if (epoch === epochRef.current && !cancelledRef.current) {
          pushToast("danger", t.updateFailed);
        }
      } finally {
        if (epoch === epochRef.current && !cancelledRef.current) {
          busyRef.current = null;
          setBusy(null);
        }
      }
    },
    [pushToast, t.updateFailed],
  );

  const runExternal = useCallback(
    async (operation: () => Promise<string | void>) => {
      if (externalBusy) return;
      setExternalBusy(true);
      try {
        const token = await operation();
        if (typeof token === "string" && token && !cancelledRef.current) {
          setTokenOnce(token);
          setTokenCopied(false);
        }
        await refreshExternal();
      } catch {
        await refreshExternal();
      } finally {
        if (!cancelledRef.current) setExternalBusy(false);
      }
    },
    [externalBusy, refreshExternal],
  );

  return {
    status,
    loading,
    error,
    busy,
    clearOpen,
    toasts,
    external,
    externalBusy,
    tokenOnce,
    tokenCopied,
    retry: () => void load(),
    setEnabled: (enabled) => void mutate("enabled", () => ipc.historySetEnabled(enabled)),
    setRetention: (days) => void mutate("retention", () => ipc.historySetRetention(days)),
    openClear: () => setClearOpen(true),
    closeClear: () => setClearOpen(false),
    confirmClear: () => {
      setClearOpen(false);
      void mutate("clear", async () => {
        const next = await ipc.historyClearAll();
        pushToast("success", t.cleared);
        return next;
      });
    },
    enableExternal: () => void runExternal(() => ipc.externalApiEnable()),
    disableExternal: () => void runExternal(() => ipc.externalApiDisable()),
    regenToken: () => void runExternal(() => ipc.externalApiRegenerateToken()),
    copyToken: () => {
      if (!tokenOnce) return;
      writeText(tokenOnce)
        .then(() => setTokenCopied(true))
        .catch(() => setTokenCopied(false));
    },
    dismissToast: (id) => setToasts((items) => items.filter((item) => item.id !== id)),
  };
}
