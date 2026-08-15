/**
 * Provider Center controller — faithful port of the Solid container.
 *
 * State model: ONE state object mirrored into a ref (`ref.current` is updated
 * synchronously on every write). Async handlers read `ref.current.*` at
 * await-time — the exact semantics Solid signals had — so a completion always
 * sees the freshest list/selection/epoch, never a stale React closure.
 *
 * Preserved invariants (see the Solid rev comments this replaces):
 *  - R7-P1-1 serial operation queue (async mutex) around ALL mutations+refresh
 *  - R9 unified per-UUID configEpoch: Test/Fetch completions are discarded
 *    after ANY config-relevant change (draft edit, model select, key save,
 *    provider update, delete, list refresh)
 *  - per-UUID request counters (Test/Fetch ABA guards)
 *  - fail-closed active selection: no providerSetActive while the cold-load
 *    read is in-flight or failed
 *  - optimistic toggle/reorder with rollback + destructive toast
 *  - R2-E stale_version → save-conflict banner (draft preserved, Reload)
 *  - key never re-readable: input cleared before the await
 *  - delete retry banner (dialog closes so the retry lives in the main area)
 */
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { detectLocale } from "../../i18n";
import { PROVIDER_COPY } from "./copy";
import { validateEndpoint } from "./domain";
import * as ipc from "./ipc";
import type {
  ActiveSelection,
  ConsentRecipient,
  ConnectionResult,
  Preset,
  ProviderDetailState,
  ProviderProfileFE,
  RoleState,
  ToastEntry,
  ToastVariant,
} from "./model";

export type SaveState = "idle" | "saving" | "saved" | "failed";

type PcState = {
  presets: Preset[];
  providers: ProviderProfileFE[];
  selection: ActiveSelection;
  loadError: boolean;
  selectionLoading: boolean;
  selectionError: boolean;
  selectedUuid: string | null;
  keyInput: Record<string, string>;
  endpointDraft: Record<string, string>;
  modelDraft: Record<string, string>;
  nameDraft: Record<string, string>;
  saveByUuid: Record<string, SaveState>;
  keyErrorByUuid: Record<string, string>;
  nameErrorByUuid: Record<string, string>;
  connByUuid: Record<string, ConnectionResult | "testing">;
  balanceByUuid: Record<string, string>;
  connRequestId: Record<string, number>;
  modelOptions: Record<string, { id: string; label: string }[]>;
  modelFetch: Record<string, "idle" | "loading" | "error">;
  modelRequestId: Record<string, number>;
  configEpoch: Record<string, number>;
  deleteConfirmUuid: string | null;
  deleteError: boolean;
  deleteFailedUuid: string | null;
  deletingUuid: string | null;
  saveConflictUuid: string | null;
  reloadingUuid: string | null;
  consentOpen: boolean;
  pendingParallelUuid: string | null;
  consentActualScope: string | null;
  toasts: ToastEntry[];
  exclusiveBusy: boolean;
};

const EMPTY_SELECTION: ActiveSelection = {
  primaryUuid: null,
  parallelUuids: [],
  fallbackUuid: null,
};

const INITIAL: PcState = {
  presets: [],
  providers: [],
  selection: EMPTY_SELECTION,
  loadError: false,
  selectionLoading: true,
  selectionError: false,
  selectedUuid: null,
  keyInput: {},
  endpointDraft: {},
  modelDraft: {},
  nameDraft: {},
  saveByUuid: {},
  keyErrorByUuid: {},
  nameErrorByUuid: {},
  connByUuid: {},
  balanceByUuid: {},
  connRequestId: {},
  modelOptions: {},
  modelFetch: {},
  modelRequestId: {},
  configEpoch: {},
  deleteConfirmUuid: null,
  deleteError: false,
  deleteFailedUuid: null,
  deletingUuid: null,
  saveConflictUuid: null,
  reloadingUuid: null,
  consentOpen: false,
  pendingParallelUuid: null,
  consentActualScope: null,
  toasts: [],
  exclusiveBusy: false,
};

export function useProviderController() {
  const t = PROVIDER_COPY[detectLocale()];
  const [state, setState] = useState<PcState>(INITIAL);
  const ref = useRef<PcState>(state);
  const disposedRef = useRef(false);
  const toastIdRef = useRef(0);

  /** Signal-like write: ref updates synchronously; state drives the render. */
  const set = useCallback(
    (patch: (s: PcState) => Partial<PcState>) => {
      const next = { ...ref.current, ...patch(ref.current) };
      ref.current = next;
      setState(next);
    },
    [],
  );

  const pushToast = useCallback(
    (variant: ToastVariant, message: string) => {
      const id = ++toastIdRef.current;
      set((s) => ({ toasts: [...s.toasts, { id, variant, message }] }));
    },
    [set],
  );

  // --- R9/R10 config epoch -------------------------------------------------

  const bumpConfigEpoch = useCallback(
    (uuid: string) => {
      set((s) => {
        const epoch = { ...s.configEpoch, [uuid]: (s.configEpoch[uuid] ?? 0) + 1 };
        const conn = { ...s.connByUuid };
        const options = { ...s.modelOptions };
        const fetchState = { ...s.modelFetch };
        delete conn[uuid];
        delete options[uuid];
        delete fetchState[uuid];
        return { configEpoch: epoch, connByUuid: conn, modelOptions: options, modelFetch: fetchState };
      });
    },
    [set],
  );

  /** Unified list replacement (R10): bump epoch for ALL old UUIDs + clean
   *  per-UUID request counters of deleted UUIDs. Every backend list goes
   *  through this — never a bare providers write. */
  const applyProviderList = useCallback(
    (newList: ProviderProfileFE[]) => {
      set((s) => {
        let epoch = s.configEpoch;
        for (const p of s.providers) {
          epoch = { ...epoch, [p.uuid]: (epoch[p.uuid] ?? 0) + 1 };
        }
        const conn = { ...s.connByUuid };
        const options = { ...s.modelOptions };
        const fetchState = { ...s.modelFetch };
        let connReq = s.connRequestId;
        let modelReq = s.modelRequestId;
        for (const p of s.providers) {
          delete conn[p.uuid];
          delete options[p.uuid];
          delete fetchState[p.uuid];
          if (!newList.some((n) => n.uuid === p.uuid)) {
            const c = { ...connReq };
            delete c[p.uuid];
            connReq = c;
            const m = { ...modelReq };
            delete m[p.uuid];
            modelReq = m;
          }
        }
        return {
          providers: newList,
          configEpoch: epoch,
          connByUuid: conn,
          modelOptions: options,
          modelFetch: fetchState,
          connRequestId: connReq,
          modelRequestId: modelReq,
        };
      });
    },
    [set],
  );

  // --- R7-P1-1 serial operation queue (async mutex) -------------------------

  const mutexRef = useRef({ inProgress: false, queue: [] as Array<() => void> });
  const runExclusive = useCallback(async <T,>(fn: () => Promise<T>): Promise<T> => {
    const m = mutexRef.current;
    while (m.inProgress) {
      await new Promise<void>((resolve) => m.queue.push(resolve));
    }
    m.inProgress = true;
    set(() => ({ exclusiveBusy: true }));
    try {
      return await fn();
    } finally {
      m.inProgress = false;
      set(() => ({ exclusiveBusy: false }));
      const next = m.queue.shift();
      if (next) next();
    }
  }, [set]);

  // --- Load ----------------------------------------------------------------

  const refreshCore = useCallback(async (): Promise<boolean> => {
    set(() => ({ selectionLoading: true, selectionError: false }));
    try {
      const [list, active] = await Promise.all([
        ipc.loadProviders(),
        ipc.providerGetActiveSelection(),
      ]);
      if (disposedRef.current) return false;
      applyProviderList(list);
      set(() => ({
        selection: {
          primaryUuid: active.primary,
          parallelUuids: active.parallel,
          fallbackUuid: active.fallback,
        },
        loadError: false,
      }));
      return true;
    } catch {
      if (disposedRef.current) return false;
      set(() => ({ loadError: true, selectionError: true }));
      return false;
    } finally {
      if (!disposedRef.current) set(() => ({ selectionLoading: false }));
    }
  }, [applyProviderList, set]);

  const refresh = useCallback(
    (): Promise<boolean> => runExclusive(() => refreshCore()),
    [runExclusive, refreshCore],
  );

  useEffect(() => {
    disposedRef.current = false;
    void ipc
      .providerListPresets()
      .then((presets) => !disposedRef.current && set(() => ({ presets })))
      .catch(() => !disposedRef.current && set(() => ({ presets: [] })));
    void refresh().then((ok) => {
      if (!ok && !disposedRef.current) pushToast("destructive", t.reloadFailed);
    });
    return () => {
      disposedRef.current = true;
    };
  }, []);

  // --- Mutations -------------------------------------------------------------

  const handleToggle = useCallback(
    async (uuid: string, enabled: boolean) => {
      await runExclusive(async () => {
        bumpConfigEpoch(uuid);
        const prev = ref.current.providers;
        const next = prev.map((p) => (p.uuid === uuid ? { ...p, enabled } : p));
        set(() => ({ providers: next }));
        try {
          await ipc.providerToggle(uuid, enabled);
          if (!enabled && !disposedRef.current) {
            const sel = ref.current.selection;
            set(() => ({
              selection: {
                primaryUuid: sel.primaryUuid === uuid ? null : sel.primaryUuid,
                parallelUuids: sel.parallelUuids.filter((u) => u !== uuid),
                fallbackUuid: sel.fallbackUuid === uuid ? null : sel.fallbackUuid,
              },
            }));
          }
        } catch {
          if (!disposedRef.current) {
            set(() => ({ providers: prev }));
            pushToast("destructive", t.saveFailed);
          }
        }
      });
    },
    [runExclusive, bumpConfigEpoch, set, pushToast, t.saveFailed],
  );

  /** Fail-closed gate: role mutations need a successful cold-load. */
  const selectionGate = () => ref.current.selectionLoading || ref.current.selectionError;

  const writeSelection = useCallback(
    async (candidate: ActiveSelection, consentUuid?: string) => {
      const result = await ipc.providerSetActive(
        candidate.primaryUuid ?? "",
        candidate.parallelUuids,
        candidate.fallbackUuid,
      );
      if (result.outcome === "written") {
        set(() => ({ selection: candidate }));
      } else if (result.outcome === "needs_consent") {
        set(() => ({
          pendingParallelUuid: consentUuid ?? null,
          consentActualScope: result.actual_scope,
          consentOpen: true,
        }));
      }
    },
    [set],
  );

  const handleSetPrimary = useCallback(
    async (uuid: string) => {
      if (selectionGate()) return;
      await runExclusive(async () => {
        const sel = ref.current.selection;
        const candidate: ActiveSelection = {
          primaryUuid: uuid,
          parallelUuids: sel.parallelUuids.filter((u) => u !== uuid),
          fallbackUuid: sel.fallbackUuid === uuid ? null : sel.fallbackUuid,
        };
        try {
          await writeSelection(candidate);
        } catch {
          pushToast("destructive", t.saveFailed);
        }
      });
    },
    [runExclusive, writeSelection, pushToast, t.saveFailed],
  );

  const handleAddParallel = useCallback(
    async (uuid: string) => {
      if (selectionGate()) return;
      await runExclusive(async () => {
        const sel = ref.current.selection;
        const candidate: ActiveSelection = {
          ...sel,
          parallelUuids: [...sel.parallelUuids, uuid],
          fallbackUuid: sel.fallbackUuid === uuid ? null : sel.fallbackUuid,
        };
        try {
          await writeSelection(candidate, uuid);
        } catch {
          pushToast("destructive", t.saveFailed);
        }
      });
    },
    [runExclusive, writeSelection, pushToast, t.saveFailed],
  );

  const confirmConsent = useCallback(async () => {
    const uuid = ref.current.pendingParallelUuid;
    if (!uuid || selectionGate()) return;
    await runExclusive(async () => {
      const sel = ref.current.selection;
      const candidate: ActiveSelection = {
        ...sel,
        parallelUuids: [...sel.parallelUuids, uuid],
        fallbackUuid: sel.fallbackUuid === uuid ? null : sel.fallbackUuid,
      };
      const scope = ref.current.consentActualScope;
      try {
        await ipc.providerConfirmAndSetActive(
          candidate.primaryUuid ?? "",
          candidate.parallelUuids,
          candidate.fallbackUuid,
          scope ?? "",
        );
        set(() => ({
          selection: candidate,
          consentOpen: false,
          pendingParallelUuid: null,
          consentActualScope: null,
        }));
      } catch {
        pushToast("destructive", t.saveFailed);
        set(() => ({ consentOpen: false, pendingParallelUuid: null, consentActualScope: null }));
      }
    });
  }, [runExclusive, set, pushToast, t.saveFailed]);

  const cancelConsent = useCallback(() => {
    set(() => ({ consentOpen: false, pendingParallelUuid: null, consentActualScope: null }));
  }, [set]);

  const handleSetFallback = useCallback(
    async (uuid: string) => {
      if (selectionGate()) return;
      await runExclusive(async () => {
        const prev = ref.current.selection;
        const candidate: ActiveSelection = {
          primaryUuid: prev.primaryUuid === uuid ? null : prev.primaryUuid,
          parallelUuids: prev.parallelUuids.filter((u) => u !== uuid),
          fallbackUuid: uuid,
        };
        try {
          await writeSelection(candidate);
        } catch {
          pushToast("destructive", t.saveFailed);
        }
      });
    },
    [runExclusive, writeSelection, pushToast, t.saveFailed],
  );

  const handleRemoveParallel = useCallback(
    async (uuid: string) => {
      if (selectionGate()) return;
      await runExclusive(async () => {
        const sel = ref.current.selection;
        const candidate: ActiveSelection = {
          ...sel,
          parallelUuids: sel.parallelUuids.filter((u) => u !== uuid),
        };
        try {
          await writeSelection(candidate);
        } catch {
          pushToast("destructive", t.saveFailed);
        }
      });
    },
    [runExclusive, writeSelection, pushToast, t.saveFailed],
  );

  const handleAddPreset = useCallback(
    async (preset: Preset) => {
      await runExclusive(async () => {
        const name = preset.name ?? "Ollama";
        try {
          await ipc.providerCreate(preset.templateId, name, preset.endpoint, preset.model ?? undefined);
          const ok = await refreshCore();
          if (ok) pushToast("success", t.profileSaved);
          else pushToast("warning", t.mutationSuccessReloadFailed);
        } catch {
          pushToast("destructive", t.saveFailed);
        }
      });
    },
    [runExclusive, refreshCore, pushToast, t],
  );

  const handleDuplicate = useCallback(
    async (uuid: string) => {
      await runExclusive(async () => {
        try {
          await ipc.providerDuplicate(uuid);
          const ok = await refreshCore();
          if (ok) pushToast("success", t.profileSaved);
          else pushToast("warning", t.mutationSuccessReloadFailed);
        } catch {
          pushToast("destructive", t.saveFailed);
        }
      });
    },
    [runExclusive, refreshCore, pushToast, t],
  );

  const handleToggleCustomAnthropic = useCallback(
    async (uuid: string, anthropic: boolean) => {
      await runExclusive(async () => {
        const provider = ref.current.providers.find((p) => p.uuid === uuid);
        if (!provider || provider.template_id !== "custom") return;
        try {
          const updated = await ipc.providerUpdate(uuid, {
            expected_version: provider.version,
            protocol: anthropic ? "anthropic" : "openai_chat",
          });
          set((s) => ({
            providers: s.providers.map((p) =>
              p.uuid === uuid ? { ...p, ...updated, hasKey: p.hasKey } : p,
            ),
          }));
          bumpConfigEpoch(uuid);
          pushToast("success", t.profileSaved);
        } catch {
          pushToast("destructive", t.saveFailed);
        }
      });
    },
    [runExclusive, set, bumpConfigEpoch, pushToast, t],
  );

  const handleSaveProfile = useCallback(
    async (uuid: string) => {
      await runExclusive(async () => {
        const s = ref.current;
        const provider = s.providers.find((p) => p.uuid === uuid);
        if (!provider) return;
        const effectiveEndpoint = s.endpointDraft[uuid] ?? provider.endpoint;
        const allowEmpty =
          provider.template_id === "custom" || provider.template_id === "azure-openai";
        if (!validateEndpoint(effectiveEndpoint, { allowEmpty }).ok) return;
        const effectiveName = (s.nameDraft[uuid] ?? provider.name).trim();
        if (effectiveName !== provider.name) {
          const conflict = s.providers.some(
            (other) => other.uuid !== uuid && other.name === effectiveName,
          );
          if (conflict) {
            set((cur) => ({ nameErrorByUuid: { ...cur.nameErrorByUuid, [uuid]: t.nameExists } }));
            return;
          }
        }
        set((cur) => {
          const n = { ...cur.nameErrorByUuid };
          delete n[uuid];
          return { nameErrorByUuid: n, saveByUuid: { ...cur.saveByUuid, [uuid]: "saving" } };
        });
        try {
          const updated = await ipc.providerUpdate(uuid, {
            name: effectiveName,
            endpoint: effectiveEndpoint,
            model: s.modelDraft[uuid] ?? provider.model,
            expected_version: provider.version,
          });
          set((cur) => ({
            providers: cur.providers.map((p) =>
              p.uuid === uuid ? { ...p, ...updated, hasKey: p.hasKey } : p,
            ),
            saveByUuid: { ...cur.saveByUuid, [uuid]: "saved" },
            saveConflictUuid: cur.saveConflictUuid === uuid ? null : cur.saveConflictUuid,
          }));
          bumpConfigEpoch(uuid);
          pushToast("success", t.profileSaved);
        } catch (e) {
          const err = e as { error?: string };
          if (err?.error === "stale_version") {
            bumpConfigEpoch(uuid);
            set((cur) => ({ saveByUuid: { ...cur.saveByUuid, [uuid]: "failed" }, saveConflictUuid: uuid }));
          } else {
            set((cur) => ({ saveByUuid: { ...cur.saveByUuid, [uuid]: "failed" } }));
            pushToast("destructive", t.saveFailed);
          }
        }
      });
    },
    [runExclusive, set, bumpConfigEpoch, pushToast, t],
  );

  const handleSaveKey = useCallback(
    async (uuid: string) => {
      const provider = ref.current.providers.find((p) => p.uuid === uuid);
      if (!provider) return;
      // Fail-closed: keyless providers and empty keys never reach the backend.
      if (!provider.needs_key) return;
      const pendingKey = ref.current.keyInput[uuid];
      if (typeof pendingKey !== "string" || pendingKey.trim().length === 0) return;
      bumpConfigEpoch(uuid);
      await runExclusive(async () => {
        const key = ref.current.keyInput[uuid];
        // Clear IMMEDIATELY — never readable back, never in the DOM after submit.
        set((cur) => {
          const k = { ...cur.keyInput };
          delete k[uuid];
          const ke = { ...cur.keyErrorByUuid };
          delete ke[uuid];
          return { keyInput: k, keyErrorByUuid: ke, saveByUuid: { ...cur.saveByUuid, [uuid]: "saving" } };
        });
        try {
          await ipc.providerSetKey(uuid, key);
          set((cur) => ({ saveByUuid: { ...cur.saveByUuid, [uuid]: "saved" } }));
          const list = await ipc.loadProviders();
          applyProviderList(list);
          pushToast("success", t.keySaved);
        } catch (e) {
          set((cur) => ({ saveByUuid: { ...cur.saveByUuid, [uuid]: "failed" } }));
          const msg = (e as { message?: string })?.message ?? "";
          if (/UNIQUE constraint/i.test(String(msg))) {
            set((cur) => ({ keyErrorByUuid: { ...cur.keyErrorByUuid, [uuid]: t.keyAlreadyExists } }));
            pushToast("destructive", t.keyAlreadyExists);
          } else {
            pushToast("destructive", t.saveFailed);
          }
        }
      });
    },
    [bumpConfigEpoch, runExclusive, set, applyProviderList, pushToast, t],
  );

  const handleFetchModels = useCallback(
    async (uuid: string) => {
      if (!ref.current.providers.some((p) => p.uuid === uuid)) return;
      const epoch = ref.current.configEpoch[uuid] ?? 0;
      const requestId = (ref.current.modelRequestId[uuid] ?? 0) + 1;
      set((cur) => ({
        modelRequestId: { ...cur.modelRequestId, [uuid]: requestId },
        modelFetch: { ...cur.modelFetch, [uuid]: "loading" },
      }));
      try {
        const models = await ipc.providerGetModels(uuid);
        if (disposedRef.current) return;
        if ((ref.current.configEpoch[uuid] ?? 0) !== epoch) return;
        if (ref.current.modelRequestId[uuid] !== requestId) return;
        set((cur) => ({
          modelOptions: { ...cur.modelOptions, [uuid]: models },
          modelFetch: { ...cur.modelFetch, [uuid]: "idle" },
        }));
      } catch {
        if (disposedRef.current) return;
        if ((ref.current.configEpoch[uuid] ?? 0) !== epoch) return;
        if (ref.current.modelRequestId[uuid] !== requestId) return;
        set((cur) => ({ modelFetch: { ...cur.modelFetch, [uuid]: "error" } }));
        pushToast("warning", t.modelFetchError);
      }
    },
    [set, pushToast, t.modelFetchError],
  );

  const handleFetchBalance = useCallback(
    async (uuid: string) => {
      const provider = ref.current.providers.find((p) => p.uuid === uuid);
      if (!provider?.capabilities.balance) return;
      set((cur) => ({ balanceByUuid: { ...cur.balanceByUuid, [uuid]: t.balance.loading } }));
      try {
        const result = await ipc.providerGetBalance(uuid);
        if (disposedRef.current) return;
        set((cur) => {
          const value =
            result.kind === "ok"
              ? result.balance + (result.quota ? ` / ${result.quota}` : "")
              : result.kind === "unsupported"
                ? t.balance.unsupportedNote
                : result.message;
          return { balanceByUuid: { ...cur.balanceByUuid, [uuid]: value } };
        });
      } catch (e) {
        if (disposedRef.current) return;
        set((cur) => ({ balanceByUuid: { ...cur.balanceByUuid, [uuid]: String(e) } }));
      }
    },
    [set, t.balance],
  );

  const handleTestConnection = useCallback(
    async (uuid: string) => {
      if (!ref.current.providers.some((p) => p.uuid === uuid)) return;
      const epoch = ref.current.configEpoch[uuid] ?? 0;
      const requestId = (ref.current.connRequestId[uuid] ?? 0) + 1;
      set((cur) => ({
        connRequestId: { ...cur.connRequestId, [uuid]: requestId },
        connByUuid: { ...cur.connByUuid, [uuid]: "testing" },
      }));
      try {
        const result = await ipc.providerTestConnection(uuid);
        if (disposedRef.current) return;
        if ((ref.current.configEpoch[uuid] ?? 0) !== epoch) return;
        if (ref.current.connRequestId[uuid] !== requestId) return;
        set((cur) => ({ connByUuid: { ...cur.connByUuid, [uuid]: result } }));
      } catch {
        if (disposedRef.current) return;
        if ((ref.current.configEpoch[uuid] ?? 0) !== epoch) return;
        if (ref.current.connRequestId[uuid] !== requestId) return;
        set((cur) => ({
          connByUuid: { ...cur.connByUuid, [uuid]: { ok: false, message: t.connectionFailed } },
        }));
      }
    },
    [set, t.connectionFailed],
  );

  const confirmDelete = useCallback(async () => {
    const uuid = ref.current.deleteConfirmUuid ?? ref.current.deleteFailedUuid;
    if (!uuid) return;
    bumpConfigEpoch(uuid);
    await runExclusive(async () => {
      set(() => ({ deletingUuid: uuid }));
      try {
        await ipc.providerDelete(uuid);
        if (disposedRef.current) return;
        set(() => ({ deleteError: false, deleteFailedUuid: null, deleteConfirmUuid: null }));
        const ok = await refreshCore();
        if (!ok) {
          pushToast("warning", t.mutationSuccessReloadFailed);
          return;
        }
        // Post-delete focus restore: the trigger row is gone, so focus the
        // first remaining Edit button or the first preset button.
        setTimeout(() => {
          if (disposedRef.current) return;
          const firstEdit = document.querySelector<HTMLButtonElement>(
            'button[aria-label^="Edit "]',
          );
          if (firstEdit) {
            firstEdit.focus();
            return;
          }
          document.querySelector<HTMLButtonElement>("[data-testid='preset-button']")?.focus();
        });
      } catch {
        if (disposedRef.current) return;
        // Close the dialog; the Retry banner lives in the main area.
        set(() => ({ deleteError: true, deleteFailedUuid: uuid, deleteConfirmUuid: null }));
        pushToast("destructive", t.saveFailed);
      } finally {
        if (!disposedRef.current) set(() => ({ deletingUuid: null }));
      }
    });
  }, [bumpConfigEpoch, runExclusive, set, refreshCore, pushToast, t]);

  const cancelDelete = useCallback(() => {
    set(() => ({ deleteConfirmUuid: null, deleteError: false, deleteFailedUuid: null }));
  }, [set]);

  const dismissDeleteError = useCallback(() => {
    set(() => ({ deleteError: false, deleteFailedUuid: null }));
  }, [set]);

  const handleDelete = useCallback(
    (uuid: string) => {
      if (!ref.current.providers.some((p) => p.uuid === uuid)) return;
      set(() => ({ deleteError: false, deleteFailedUuid: null, deleteConfirmUuid: uuid }));
    },
    [set],
  );

  const moveProvider = useCallback(
    async (uuid: string, dir: "up" | "down") => {
      await runExclusive(async () => {
        const ordered = [...ref.current.providers].sort((a, b) => a.sort_order - b.sort_order);
        const idx = ordered.findIndex((p) => p.uuid === uuid);
        if (idx < 0) return;
        const swap = dir === "up" ? idx - 1 : idx + 1;
        if (swap < 0 || swap >= ordered.length) return;
        const snapshot = [...ordered];
        const newOrder = [...ordered];
        [newOrder[idx], newOrder[swap]] = [newOrder[swap], newOrder[idx]];
        const renumbered = newOrder.map((p, i) => ({ ...p, sort_order: i }));
        set(() => ({ providers: renumbered }));
        try {
          await ipc.providerReorder(renumbered.map((p) => p.uuid));
        } catch {
          if (!disposedRef.current) {
            set(() => ({ providers: snapshot.map((p, i) => ({ ...p, sort_order: i })) }));
            pushToast("destructive", t.reorderReverted);
          }
        }
      });
    },
    [runExclusive, set, pushToast, t.reorderReverted],
  );

  const resolveSaveConflict = useCallback(
    async (uuid: string) => {
      if (ref.current.reloadingUuid) return; // re-entrancy guard
      set(() => ({ reloadingUuid: uuid }));
      await runExclusive(async () => {
        const ok = await refreshCore();
        if (!ok) {
          pushToast("destructive", t.reloadFailed);
          if (!disposedRef.current) set(() => ({ reloadingUuid: null }));
          return;
        }
        set((cur) => {
          const nameDraft = { ...cur.nameDraft };
          const endpointDraft = { ...cur.endpointDraft };
          const modelDraft = { ...cur.modelDraft };
          const nameError = { ...cur.nameErrorByUuid };
          const saveByUuid = { ...cur.saveByUuid };
          delete nameDraft[uuid];
          delete endpointDraft[uuid];
          delete modelDraft[uuid];
          delete nameError[uuid];
          delete saveByUuid[uuid];
          return {
            nameDraft,
            endpointDraft,
            modelDraft,
            nameErrorByUuid: nameError,
            saveByUuid,
            saveConflictUuid: cur.saveConflictUuid === uuid ? null : cur.saveConflictUuid,
            reloadingUuid: null,
          };
        });
      });
    },
    [runExclusive, refreshCore, pushToast, t.reloadFailed],
  );

  // --- Draft inputs ------------------------------------------------------------

  const onNameInput = useCallback(
    (uuid: string, value: string) => {
      set((cur) => {
        const errors = { ...cur.nameErrorByUuid };
        delete errors[uuid];
        return { nameDraft: { ...cur.nameDraft, [uuid]: value }, nameErrorByUuid: errors };
      });
    },
    [set],
  );

  const onEndpointInput = useCallback(
    (uuid: string, value: string) => {
      set((cur) => ({ endpointDraft: { ...cur.endpointDraft, [uuid]: value } }));
      bumpConfigEpoch(uuid);
    },
    [set, bumpConfigEpoch],
  );

  const onModelInput = useCallback(
    (uuid: string, value: string) => {
      set((cur) => ({ modelDraft: { ...cur.modelDraft, [uuid]: value } }));
      bumpConfigEpoch(uuid);
    },
    [set, bumpConfigEpoch],
  );

  const onModelChange = useCallback(
    (uuid: string, value: string) => {
      const s = ref.current;
      const effective = s.modelDraft[uuid] ?? s.providers.find((p) => p.uuid === uuid)?.model ?? "";
      if (effective === value) return; // idempotent re-emission guard
      set((cur) => ({ modelDraft: { ...cur.modelDraft, [uuid]: value } }));
      bumpConfigEpoch(uuid);
    },
    [set, bumpConfigEpoch],
  );

  const onKeyInput = useCallback(
    (uuid: string, value: string) => {
      set((cur) => {
        const errors = { ...cur.keyErrorByUuid };
        delete errors[uuid];
        return { keyInput: { ...cur.keyInput, [uuid]: value }, keyErrorByUuid: errors };
      });
      if (value.length > 0) bumpConfigEpoch(uuid);
    },
    [set, bumpConfigEpoch],
  );

  // --- Derived ------------------------------------------------------------

  const detail = useMemo<ProviderDetailState | null>(() => {
    const p = state.providers.find((x) => x.uuid === state.selectedUuid);
    if (!p) return null;
    const uuid = p.uuid;
    const draftEndpoint = state.endpointDraft[uuid] ?? p.endpoint;
    const draftModel = state.modelDraft[uuid] ?? p.model ?? "";
    const draftName = state.nameDraft[uuid] ?? p.name;
    let endpointError: string | undefined;
    if (draftEndpoint !== p.endpoint) {
      const check = validateEndpoint(draftEndpoint);
      if (!check.ok) endpointError = t.endpoint.errors[check.code];
    }
    return {
      provider: p,
      nameDraft: draftName,
      endpointDraft: draftEndpoint,
      modelDraft: draftModel,
      keyText: state.keyInput[uuid] ?? "",
      nameError: state.nameErrorByUuid[uuid],
      keyError: state.keyErrorByUuid[uuid],
      endpointError,
      saveState: state.saveByUuid[uuid] ?? "idle",
      conn: state.connByUuid[uuid] ?? "idle",
      modelOptions: state.modelOptions[uuid] ?? [],
      modelFetch: state.modelFetch[uuid] ?? "idle",
      saveConflict: state.saveConflictUuid === uuid,
    };
  }, [state, t]);

  const consentRecipients = useMemo<ConsentRecipient[]>(() => {
    const sel = state.pendingParallelUuid
      ? {
          ...state.selection,
          parallelUuids: [...state.selection.parallelUuids, state.pendingParallelUuid],
        }
      : state.selection;
    const recipientUuids = [sel.primaryUuid, ...sel.parallelUuids].filter(
      (u): u is string => u !== null,
    );
    return recipientUuids.map((uuid) => {
      const p = state.providers.find((x) => x.uuid === uuid);
      return { name: p?.name ?? uuid, localLabel: p?.is_local ? t.consent.local : t.consent.remote };
    });
  }, [state, t]);

  const roleFor = useCallback(
    (uuid: string): RoleState => {
      const sel = state.selection;
      if (sel.primaryUuid === uuid) return { kind: "primary" };
      const idx = sel.parallelUuids.indexOf(uuid);
      if (idx >= 0) return { kind: "parallel", index: idx + 1 };
      if (sel.fallbackUuid === uuid) return { kind: "fallback" };
      return { kind: "none" };
    },
    [state.selection],
  );

  return {
    // state
    presets: state.presets,
    providers: state.providers,
    selection: state.selection,
    loadError: state.loadError,
    selectionError: state.selectionError,
    selectionLoading: state.selectionLoading,
    selectedUuid: state.selectedUuid,
    deletingUuid: state.deletingUuid,
    reloadingUuid: state.reloadingUuid,
    exclusiveBusy: state.exclusiveBusy,
    deleteConfirmUuid: state.deleteConfirmUuid,
    deleteError: state.deleteError,
    deleteFailedUuid: state.deleteFailedUuid,
    consentOpen: state.consentOpen,
    consentRecipients,
    toasts: state.toasts,
    balanceByUuid: state.balanceByUuid,
    detail,
    roleFor,
    // callbacks
    select: (uuid: string) => set(() => ({ selectedUuid: uuid })),
    onToggle: (uuid: string, enabled: boolean) => void handleToggle(uuid, enabled),
    onDelete: handleDelete,
    onSetPrimary: (uuid: string) => void handleSetPrimary(uuid),
    onAddParallel: (uuid: string) => void handleAddParallel(uuid),
    onRemoveParallel: (uuid: string) => void handleRemoveParallel(uuid),
    onSetFallback: (uuid: string) => void handleSetFallback(uuid),
    onDuplicate: (uuid: string) => void handleDuplicate(uuid),
    onMoveUp: (uuid: string) => void moveProvider(uuid, "up"),
    onMoveDown: (uuid: string) => void moveProvider(uuid, "down"),
    onAddPreset: (preset: Preset) => void handleAddPreset(preset),
    onNameInput,
    onEndpointInput,
    onModelInput,
    onModelChange,
    onKeyInput,
    onSaveProfile: (uuid: string) => void handleSaveProfile(uuid),
    onToggleCustomAnthropic: (uuid: string, on: boolean) =>
      void handleToggleCustomAnthropic(uuid, on),
    onSaveKey: (uuid: string) => void handleSaveKey(uuid),
    onFetchModels: (uuid: string) => void handleFetchModels(uuid),
    onTestConnection: (uuid: string) => void handleTestConnection(uuid),
    onFetchBalance: (uuid: string) => void handleFetchBalance(uuid),
    onResolveSaveConflict: (uuid: string) => void resolveSaveConflict(uuid),
    onReloadFromError: () => {
      void refresh().then((ok) => {
        if (!ok) pushToast("destructive", t.reloadFailed);
      });
    },
    onRetrySelectionLoad: () => void refresh(),
    onConfirmDelete: () => void confirmDelete(),
    onCancelDelete: cancelDelete,
    onRetryDelete: () => void confirmDelete(),
    onDismissDeleteError: dismissDeleteError,
    onConfirmConsent: () => void confirmConsent(),
    onCancelConsent: cancelConsent,
    onDismissToast: (id: number) =>
      set((cur) => ({ toasts: cur.toasts.filter((x) => x.id !== id) })),
  };
}

export type ProviderController = ReturnType<typeof useProviderController>;
