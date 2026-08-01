import {
  For,
  Show,
  Switch as FlowSwitch,
  Match,
  createSignal,
  createMemo,
  createEffect,
  onCleanup,
  on,
  untrack,
  batch,
  type Component,
} from "solid-js";
import { Server, Plus, Copy, ArrowUp, ArrowDown, Check, X, Globe, GripVertical, Keyboard, Shield } from "lucide-solid";
import {
  ProviderCard,
  Button,
  TextField,
  Select,
  Confirm,
  Toast,
  Banner,
  EmptyState,
  Spinner,
  type ProviderRole,
  type SelectOption,
  type ProviderCardLabels,
} from "@linguaray/ui";
import type { Locale, LabStrings, ProviderState } from "../i18n";
import {
  type MockProvider,
  type ActiveSelection,
  validateActiveSelection,
  buildConsentScope,
  consentScopeKey,
  validateEndpoint,
  TRADITIONAL_TEMPLATES,
} from "./provider-domain";
import { OpRegistry, type OpKind } from "./op-registry";
import "./ProviderCenter.css";

export type ProviderCenterProps = {
  state: ProviderState;
  locale: Locale;
  t: LabStrings["provider"];
};

// --- Mock provider fixtures (NO real keys) --------------------------------

let uuidCounter = 100;
function mockUuid(): string {
  uuidCounter += 1;
  return `mock-${uuidCounter}`;
}

function initialProviders(): MockProvider[] {
  return [
    { uuid: "mock-openai-1", template: "openai", name: "OpenAI #1", endpoint: "https://api.openai.com/v1/chat/completions", model: "gpt-4o", enabled: true, isLocal: false, hasKey: true, status: "active", sortOrder: 0 },
    { uuid: "mock-openai-2", template: "openai", name: "OpenAI #2", endpoint: "https://api.openai.com/v1/chat/completions", model: "gpt-4o-mini", enabled: true, isLocal: false, hasKey: false, status: "active", sortOrder: 1 },
    { uuid: "mock-deepseek", template: "deepseek", name: "DeepSeek", endpoint: "https://api.deepseek.com/v1/chat/completions", model: "deepseek-chat", enabled: true, isLocal: false, hasKey: true, status: "active", sortOrder: 2 },
    { uuid: "mock-google", template: "google", name: "Google Translate", endpoint: "https://translation.googleapis.com/", model: null, enabled: true, isLocal: false, hasKey: true, status: "active", sortOrder: 3 },
    { uuid: "mock-ollama", template: "ollama", name: "Ollama", endpoint: "http://localhost:11434/v1/chat/completions", model: "llama3", enabled: false, isLocal: true, hasKey: false, status: "active", sortOrder: 4 },
  ];
}

const DEFAULT_SELECTION: ActiveSelection = {
  primaryUuid: "mock-openai-1",
  parallelUuids: ["mock-deepseek"],
  fallbackUuid: "mock-google",
};

const MODEL_OPTIONS: SelectOption[] = [
  { value: "gpt-4o", label: "GPT-4o", disabled: false },
  { value: "gpt-4o-mini", label: "GPT-4o mini", disabled: false },
  { value: "gpt-4-turbo", label: "GPT-4 Turbo", disabled: false },
];

const PRESETS = [
  { template: "openai", name: "OpenAI" },
  { template: "anthropic", name: "Anthropic" },
  { template: "gemini", name: "Gemini" },
  { template: "deepseek", name: "DeepSeek" },
  { template: "google", name: "Google Translate" },
  { template: "deepl", name: "DeepL" },
  { template: "ollama", name: null as string | null }, // translated at render time
];

// --- Component ------------------------------------------------------------

const ProviderCenter: Component<ProviderCenterProps> = (props) => {
  const [providers, setProviders] = createSignal<MockProvider[]>(initialProviders());
  const [selection, setSelection] = createSignal<ActiveSelection>({ ...DEFAULT_SELECTION });
  const [consentKey, setConsentKey] = createSignal<string | null>(
    consentScopeKey(buildConsentScope(DEFAULT_SELECTION, initialProviders())),
  );
  const [consentOpen, setConsentOpen] = createSignal(false);
  const [pendingParallel, setPendingParallel] = createSignal<string | null>(null);

  // Detail panel state — all per-UUID to prevent cross-provider leakage
  const [selectedUuid, setSelectedUuid] = createSignal<string | null>("mock-openai-1");
  const [keyInputByUuid, setKeyInputByUuid] = createSignal<Record<string, string>>({});
  // Per-UUID profile drafts (endpoint + model). Not committed to provider
  // fixture until handleSaveProfile. Unsaved drafts NEVER participate in
  // consent scope — only committed providers do.
  const [endpointDraft, setEndpointDraft] = createSignal<Record<string, string>>({});
  const [modelDraftByUuid, setModelDraftByUuid] = createSignal<Record<string, string>>({});
  // Per-UUID busy states (no globals)
  const [balanceByUuid, setBalanceByUuid] = createSignal<Record<string, "idle" | "loading" | "done" | "unsupported" | "rate-limited" | "error">>({});
  const [conflictResolved, setConflictResolved] = createSignal(false);
  const [retryTargetUuid, setRetryTargetUuid] = createSignal<string | null>(null);
  const [modelFetchByUuid, setModelFetchByUuid] = createSignal<Record<string, "idle" | "loading" | "error">>({});
  const [connStatus, setConnStatus] = createSignal<Record<string, "idle" | "testing" | "ok" | "failed">>({});
  const [connLatency, setConnLatency] = createSignal<Record<string, number>>({});
  const [saveByUuid, setSaveByUuid] = createSignal<Record<string, "idle" | "saving" | "saved" | "failed">>({});
  const [deleteConfirmUuid, setDeleteConfirmUuid] = createSignal<string | null>(null);
  // Trigger refs for focus-restore on dialog close
  const deleteTriggerRef: { current?: HTMLElement } = {};
  const consentTriggerRef: { current?: HTMLElement } = {};
  const sidebarFallbackRef: { current?: HTMLElement } = {};
  const [toasts, setToasts] = createSignal<{ id: number; variant: "info" | "success" | "warning" | "destructive"; message: string }[]>([]);
  const [reorderAnnouncement, setReorderAnnouncement] = createSignal("");
  const [showPresetGrid, setShowPresetGrid] = createSignal(false);
  const [deleteRetryPending, setDeleteRetryPending] = createSignal(false);
  // Reorder persist pending — disables further reorder/drag until resolved
  const [reorderPending, setReorderPending] = createSignal(false);
  // Drag state
  const [draggedUuid, setDraggedUuid] = createSignal<string | null>(null);
  const [dragOverUuid, setDragOverUuid] = createSignal<string | null>(null);
  const [dragOverPos, setDragOverPos] = createSignal<"before" | "after">("before");

  // --- CAS operation registry ---
  const opRegistry = new OpRegistry();

  // --- Tracked timers for non-op delayed callbacks (delete, retry, rollback).
  // These are cancelled on state-change reset and onCleanup to prevent
  // cross-state pollution (old delete callback removing new fixture's provider).
  const trackedTimers = new Set<number>();
  const scheduleTracked = (fn: () => void, ms: number): void => {
    const id = window.setTimeout(() => {
      trackedTimers.delete(id);
      fn();
    }, ms);
    trackedTimers.add(id);
  };
  const clearTrackedTimers = (): void => {
    for (const id of trackedTimers) window.clearTimeout(id);
    trackedTimers.clear();
  };

  // selectProvider cancels ALL selection-scoped ops for the OLD provider
  // synchronously (not waiting for timers), then sets the new selection.
  const selectProvider = (uuid: string | null): void => {
    const old = selectedUuid();
    if (old && old !== uuid) {
      opRegistry.cancelOpsForUuid(old);
    }
    setSelectedUuid(uuid);
  };

  // --- commitProviderState: atomic transaction (batch) ---
  // Validates nextSelection against nextProviders, commits all three in batch.
  // approvedConsentKey: if provided (from consent Confirm), written in the
  // same batch. If not provided, consent is preserved only if scope matches;
  // otherwise set to null (invalidated). Never auto-approves.
  const commitProviderState = (
    nextProviders: MockProvider[],
    nextSelection: ActiveSelection,
    approvedConsentKey?: string,
  ): boolean => {
    const result = validateActiveSelection(nextSelection, nextProviders);
    if (!result.ok) {
      pushToast("destructive", result.errors[0]!.message);
      return false;
    }
    const oldScopeKey = consentScopeKey(buildConsentScope(selection(), providers()));
    const newScope = buildConsentScope(nextSelection, nextProviders);
    const newKey = consentScopeKey(newScope);
    const previousConsent = consentKey();
    // If an approved key is provided, use it. Otherwise preserve only if
    // the scope hasn't changed and consent was already valid.
    const nextConsent = approvedConsentKey !== undefined
      ? approvedConsentKey
      : (previousConsent !== null && previousConsent === oldScopeKey && newKey === oldScopeKey
          ? previousConsent
          : null);
    batch(() => {
      setProviders(nextProviders);
      setSelection(nextSelection);
      setConsentKey(nextConsent);
    });
    return true;
  };

  // --- toast helper ---
  let toastId = 0;
  const pushToast = (variant: "info" | "success" | "warning" | "destructive", message: string) => {
    const id = ++toastId;
    setToasts((prev) => [...prev, { id, variant, message }]);
  };
  const dismissToast = (id: number) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  };

  // Reset transient mock state on ProviderState change ONLY.
  // Uses on(() => props.state) + untrack so selecting a provider does NOT
  // re-trigger the fixture reset (which would wipe user interactions).
  createEffect(
    on(
      () => props.state,
      (state) => untrack(() => {
    // Cancel ALL async operations (CAS registry: delete → clearBusy per op)
    opRegistry.cancelAll();
    clearTrackedTimers();
    setKeyInputByUuid({});
    setModelDraftByUuid({});
    setModelFetchByUuid({});
    setConnStatus({});
    setConnLatency({});
    setSaveByUuid({});
    setProfileSaveByUuid({});
    setEndpointErrorByUuid({});
    setBalanceByUuid({});
    setEndpointDraft({});
    setConflictResolved(false);
    setRetryTargetUuid(null);
    setDeleteConfirmUuid(null);
    setDeleteRetryPending(false);
    setReorderAnnouncement("");
    setShowPresetGrid(false);
    setReorderPending(false);
    setDraggedUuid(null);
    setDragOverUuid(null);

    // State-specific fixtures
    let provs = initialProviders();
    let sel: ActiveSelection = { ...DEFAULT_SELECTION };

    if (state === "empty") {
      provs = [];
      sel = { primaryUuid: null, parallelUuids: [], fallbackUuid: null };
    } else if (state === "duplicate") {
      const orig = provs.find((p) => p.uuid === "mock-openai-1")!;
      provs = [...provs, { ...orig, uuid: "mock-openai-dup", name: "OpenAI #1 (copy)", hasKey: false, sortOrder: provs.length }];
    } else if (state === "deleting" || state === "delete-retry") {
      provs = provs.map((p) => (p.uuid === "mock-openai-1" ? { ...p, status: "deleting" as const, enabled: false } : p));
      sel = {
        primaryUuid: null,
        parallelUuids: sel.parallelUuids.filter((u) => u !== "mock-openai-1"),
        fallbackUuid: sel.fallbackUuid,
      };
      if (state === "delete-retry") setRetryTargetUuid("mock-openai-1");
    } else if (state === "delete-confirm") {
      // Open the delete dialog on the first provider (via signal, not prop)
      setDeleteConfirmUuid("mock-openai-1");
    }

    // Validate fixture — THROW on invalid (not silent clear)
    const selResult = validateActiveSelection(sel, provs);
    if (!selResult.ok) {
      throw new Error(
        `Invalid fixture for state "${state}": ${selResult.errors.map((e) => e.message).join(", ")}`,
      );
    }

    batch(() => {
      setProviders(provs);
      setSelection(sel);
    });

    // State-specific status fixtures (per-UUID)
    if (state === "loading-models") {
      setModelFetchByUuid({ "mock-openai-1": "loading" });
    }
    if (state === "saving") {
      setSaveByUuid({ "mock-openai-2": "saving" });
      selectProvider("mock-openai-2");
    } else if (state === "key-missing" || state === "save-failed") {
      selectProvider("mock-openai-2"); // no key → shows save form
    } else {
      const primary = sel.primaryUuid;
      const pp = provs.find((p) => p.uuid === primary);
      selectProvider(primary && pp?.enabled && pp?.status === "active" ? primary : (provs.find((p) => p.enabled && p.status === "active")?.uuid ?? null));
    }

    if (state === "connection-ok") {
      setConnStatus({ "mock-openai-1": "ok" });
      setConnLatency({ "mock-openai-1": 42 });
    } else if (state === "connection-failed") {
      setConnStatus({ "mock-openai-1": "failed" });
    }

    // Balance: demo states set the OUTCOME fixture (what fetch will resolve
    // to), NOT the current status. Current status starts at "idle" so the
    // Fetch button is visible. The user clicks Fetch → loading → outcome.
    // Exception: balance-loading starts mid-loading (shows spinner).
    if (state === "balance-loading") {
      setBalanceByUuid({ "mock-openai-1": "loading" });
    }
    // For unsupported/rate-limited/error: DON'T pre-set status. The outcome
    // is determined by handleFetchBalance reading props.state. Status stays
    // idle so the Fetch button renders.

    setConsentKey(consentScopeKey(buildConsentScope(sel, provs)));
    }),
    ),
  );

  onCleanup(() => { opRegistry.cancelAll(); clearTrackedTimers(); });

  // --- card labels from i18n ---
  const cardLabels = (): ProviderCardLabels => ({
    primary: props.t.primary,
    parallel: props.t.parallel,
    fallback: props.t.fallback,
    keySaved: props.t.cardKeySaved,
    keyMissing: props.t.cardKeyMissing,
    enabled: props.t.enabled,
    disabled: props.t.disabled,
    edit: props.t.cardEdit,
    delete: props.t.cardDelete,
  });

  // --- derived role for each provider ---
  const roleFor = (uuid: string): ProviderRole => {
    const sel = selection();
    if (sel.primaryUuid === uuid) return { kind: "primary" };
    const idx = sel.parallelUuids.indexOf(uuid);
    if (idx >= 0) return { kind: "parallel", index: idx + 1 };
    if (sel.fallbackUuid === uuid) return { kind: "fallback" };
    return { kind: "none" };
  };

  // --- handleToggle: uses commitProviderState (atomic, validates nextProviders) ---
  const handleToggle = (uuid: string, enabled: boolean) => {
    const nextProviders = providers().map((p) => (p.uuid === uuid ? { ...p, enabled } : p));
    let nextSel = selection();
    if (!enabled) {
      nextSel = {
        primaryUuid: nextSel.primaryUuid === uuid ? null : nextSel.primaryUuid,
        parallelUuids: nextSel.parallelUuids.filter((u) => u !== uuid),
        fallbackUuid: nextSel.fallbackUuid === uuid ? null : nextSel.fallbackUuid,
      };
    }
    commitProviderState(nextProviders, nextSel);
  };

  const handleSetPrimary = (uuid: string) => {
    const prev = selection();
    const candidate: ActiveSelection = {
      primaryUuid: uuid,
      parallelUuids: prev.parallelUuids.filter((u) => u !== uuid),
      fallbackUuid: prev.fallbackUuid === uuid ? null : prev.fallbackUuid,
    };
    commitProviderState(providers(), candidate);
  };

  const handleAddParallel = (uuid: string, triggerEl?: HTMLElement) => {
    if (triggerEl) consentTriggerRef.current = triggerEl;
    setPendingParallel(uuid);
    setConsentOpen(true);
  };

  const confirmConsent = () => {
    const uuid = pendingParallel();
    if (uuid) {
      const prev = selection();
      if (uuid !== prev.primaryUuid && !prev.parallelUuids.includes(uuid)) {
        const candidate: ActiveSelection = {
          ...prev,
          parallelUuids: [...prev.parallelUuids, uuid],
          fallbackUuid: prev.fallbackUuid === uuid ? null : prev.fallbackUuid,
        };
        const approvedKey = consentScopeKey(buildConsentScope(candidate, providers()));
        // Pass approvedConsentKey so it's committed in the same batch
        commitProviderState(providers(), candidate, approvedKey);
      }
    }
    setConsentOpen(false);
    setPendingParallel(null);
  };

  const cancelConsent = () => {
    setConsentOpen(false);
    setPendingParallel(null);
  };

  const handleRemoveParallel = (uuid: string) => {
    const candidate: ActiveSelection = {
      ...selection(),
      parallelUuids: selection().parallelUuids.filter((u) => u !== uuid),
    };
    // commitProviderState already handles consent correctly:
    // scope change → null (invalidated), scope unchanged → preserved.
    // Do NOT write consentKey here — that would mint approval without Confirm.
    commitProviderState(providers(), candidate);
  };

  const handleSetFallback = (uuid: string) => {
    const prev = selection();
    // Fallback must not overlap primary or parallel
    const candidate: ActiveSelection = {
      primaryUuid: prev.primaryUuid === uuid ? null : prev.primaryUuid,
      parallelUuids: prev.parallelUuids.filter((u) => u !== uuid),
      fallbackUuid: uuid,
    };
    commitProviderState(providers(), candidate);
  };

  // --- profile save: validate endpoint, commit draft to provider, recalc consent ---
  // Only committed providers participate in consent scope. Unsaved drafts
  // never enter the recipient set.
  const [profileSaveByUuid, setProfileSaveByUuid] = createSignal<Record<string, "idle" | "saving" | "saved" | "failed">>({});
  const [endpointErrorByUuid, setEndpointErrorByUuid] = createSignal<Record<string, string>>({});

  const handleSaveProfile = (uuid: string) => {
    const draftEndpoint = endpointDraft()[uuid];
    const draftModel = modelDraftByUuid()[uuid];
    const provider = providers().find((p) => p.uuid === uuid);
    if (!provider) return;

    // Validate endpoint if changed
    const effectiveEndpoint = draftEndpoint ?? provider.endpoint;
    const epCheck = validateEndpoint(effectiveEndpoint);
    if (!epCheck.ok) {
      setEndpointErrorByUuid((prev) => ({ ...prev, [uuid]: epCheck.error! }));
      return;
    }
    setEndpointErrorByUuid((prev) => {
      const next = { ...prev };
      delete next[uuid];
      return next;
    });

    setProfileSaveByUuid((prev) => ({ ...prev, [uuid]: "saving" }));
    const token = opRegistry.startOp(
      "profile-save" as OpKind,
      uuid,
      () => setProfileSaveByUuid((prev) => ({ ...prev, [uuid]: "idle" })),
      () => {
        // Commit: write drafts into the provider fixture
        const oldProviders = providers();
        const oldScopeKey = consentScopeKey(buildConsentScope(selection(), oldProviders));
        const nextProviders = oldProviders.map((p) =>
          p.uuid === uuid
            ? { ...p, endpoint: effectiveEndpoint, model: draftModel ?? p.model }
            : p,
        );
        const sel = selection();
        const newScope = buildConsentScope(sel, nextProviders);
        const newKey = consentScopeKey(newScope);
        const previousConsent = consentKey();
        // Consent preservation: only retain previousConsent if it matched
        // the OLD scope AND the scope hasn't changed. Never auto-approve.
        const nextConsent =
          previousConsent !== null &&
          previousConsent === oldScopeKey &&
          newKey === oldScopeKey
            ? previousConsent
            : null;
        // Atomic batch: providers + selection + consent together
        batch(() => {
          setProviders(nextProviders);
          setConsentKey(nextConsent);
        });
        // Clear the committed drafts
        setEndpointDraft((prev) => { const n = { ...prev }; delete n[uuid]; return n; });
        setModelDraftByUuid((prev) => { const n = { ...prev }; delete n[uuid]; return n; });
        setProfileSaveByUuid((prev) => ({ ...prev, [uuid]: "saved" }));
        pushToast("success", props.t.profileSaved);
      },
      1000,
    );
    void token;
  };

  // --- save key: CAS registry (per-UUID) ---
  // Key input cleared at SUBMISSION START (before async), regardless of outcome.
  const handleSaveKey = () => {
    const targetUuid = selectedUuid();
    if (!targetUuid) return;
    // Clear key input IMMEDIATELY — never readable back, never in DOM after submit
    setKeyInputByUuid((prev) => {
      const next = { ...prev };
      delete next[targetUuid];
      return next;
    });
    const token = opRegistry.startOp(
      "save" as OpKind,
      targetUuid,
      () => setSaveByUuid((prev) => ({ ...prev, [targetUuid]: "idle" })),
      () => {
        if (props.state === "save-failed") {
          setSaveByUuid((prev) => ({ ...prev, [targetUuid]: "failed" }));
          pushToast("destructive", props.t.saveFailed);
        } else {
          setProviders((prev) => prev.map((p) => (p.uuid === targetUuid ? { ...p, hasKey: true } : p)));
          setSaveByUuid((prev) => ({ ...prev, [targetUuid]: "saved" }));
          pushToast("success", props.t.keySaved);
        }
      },
      1000,
    );
    setSaveByUuid((prev) => ({ ...prev, [targetUuid]: "saving" }));
    void token;
  };

  // --- connection test: CAS registry (per-UUID) ---
  const handleTestConnection = () => {
    const targetUuid = selectedUuid();
    if (!targetUuid) return;
    const token = opRegistry.startOp(
      "test" as OpKind,
      targetUuid,
      () => setConnStatus((prev) => ({ ...prev, [targetUuid]: "idle" })),
      () => {
        if (props.state === "connection-failed") {
          setConnStatus((prev) => ({ ...prev, [targetUuid]: "failed" }));
        } else {
          setConnLatency((prev) => ({ ...prev, [targetUuid]: 42 }));
          setConnStatus((prev) => ({ ...prev, [targetUuid]: "ok" }));
        }
      },
      1200,
    );
    setConnStatus((prev) => ({ ...prev, [targetUuid]: "testing" }));
    void token;
  };

  // --- fetch models: CAS registry (per-UUID) ---
  const handleFetchModels = () => {
    const targetUuid = selectedUuid();
    if (!targetUuid) return;
    const token = opRegistry.startOp(
      "fetch" as OpKind,
      targetUuid,
      () => setModelFetchByUuid((prev) => ({ ...prev, [targetUuid]: "idle" })),
      () => {
        if (props.state === "model-fetch-error") {
          setModelFetchByUuid((prev) => ({ ...prev, [targetUuid]: "error" }));
        } else {
          setModelFetchByUuid((prev) => ({ ...prev, [targetUuid]: "idle" }));
        }
      },
      1000,
    );
    setModelFetchByUuid((prev) => ({ ...prev, [targetUuid]: "loading" }));
    void token;
  };

  // --- duplicate (key NOT copied) ---
  const handleDuplicate = (uuid: string) => {
    const orig = providers().find((p) => p.uuid === uuid);
    if (!orig) return;
    const copy: MockProvider = {
      ...orig,
      uuid: mockUuid(),
      name: `${orig.name} (copy)`,
      hasKey: false,
      sortOrder: providers().length,
    };
    setProviders((prev) => [...prev, copy]);
  };

  // --- delete flow ---
  const handleDelete = (uuid: string, triggerEl?: HTMLElement) => {
    if (triggerEl) deleteTriggerRef.current = triggerEl;
    setDeleteConfirmUuid(uuid);
  };

  const handleEdit = (uuid: string) => {
    selectProvider(uuid);
  };

  const confirmDelete = () => {
    const uuid = deleteConfirmUuid();
    if (!uuid) return;
    // Deleting: immediately disable + clear roles via commitProviderState
    const nextProviders = providers()
      .map((p) => (p.uuid === uuid ? { ...p, status: "deleting" as const, enabled: false } : p));
    const nextSel: ActiveSelection = {
      primaryUuid: selection().primaryUuid === uuid ? null : selection().primaryUuid,
      parallelUuids: selection().parallelUuids.filter((u) => u !== uuid),
      fallbackUuid: selection().fallbackUuid === uuid ? null : selection().fallbackUuid,
    };
    commitProviderState(nextProviders, nextSel);
    setDeleteConfirmUuid(null);
    // Schedule removal via tracked timer (cancelled on state change/unmount)
    scheduleTracked(() => {
      setProviders((prev) => prev.filter((p) => p.uuid !== uuid));
    }, 1500);
  };

  const cancelDelete = () => {
    setDeleteConfirmUuid(null);
  };

  // --- save conflict: reload discards drafts + clears key input; cancel keeps drafts ---
  const handleConflictReload = () => {
    setConflictResolved(true);
    // Reload: overwrite endpoint/model drafts from fixture, CLEAR key input
    // (key cannot be read back from backend — only clear, never backfill)
    const provs = initialProviders();
    setEndpointDraft({});
    setModelDraftByUuid({});
    setKeyInputByUuid({});
    commitProviderState(provs, { ...DEFAULT_SELECTION });
    selectProvider(DEFAULT_SELECTION.primaryUuid);
    pushToast("info", props.t.reload);
  };

  const handleConflictCancel = () => {
    setConflictResolved(true);
    // Cancel: keep endpoint/model drafts + dirty. Clear key input (not restored).
    // Provider fixture unchanged.
    setKeyInputByUuid({});
  };

  // --- delete retry: re-attempt the delete for the stuck provider ---
  const handleDeleteRetry = () => {
    const uuid = retryTargetUuid();
    if (!uuid || deleteRetryPending()) return;
    setDeleteRetryPending(true);
    scheduleTracked(() => {
      setProviders((prev) => prev.filter((p) => p.uuid !== uuid));
      setRetryTargetUuid(null);
      setDeleteRetryPending(false);
      pushToast("success", props.t.delete);
    }, 1500);
  };

  // --- add provider from preset ---
  const handleAddPreset = (preset: { template: string; name: string | null }) => {
    const newProvider: MockProvider = {
      uuid: mockUuid(),
      template: preset.template as MockProvider["template"],
      name: preset.name ?? props.t.presetOllama,
      endpoint: preset.template === "ollama" ? "http://localhost:11434/v1/chat/completions" : `https://api.${preset.template}.example.com/v1`,
      model: null,
      enabled: true,
      isLocal: preset.template === "ollama",
      hasKey: false,
      status: "active",
      sortOrder: providers().length,
    };
    setProviders((prev) => [...prev, newProvider]);
    setShowPresetGrid(false);
  };

  // --- reorder (keyboard-first) with persist/rollback ---
  const reorderProviders = (fromUuid: string, toUuid: string, pos: "before" | "after") => {
    const snapshot = providers(); // capture before reorder
    setProviders((prev) => {
      const sorted = [...prev].sort((a, b) => a.sortOrder - b.sortOrder);
      const fromIdx = sorted.findIndex((p) => p.uuid === fromUuid);
      const toIdx = sorted.findIndex((p) => p.uuid === toUuid);
      if (fromIdx < 0 || toIdx < 0 || fromIdx === toIdx) return prev;
      const [moved] = sorted.splice(fromIdx, 1);
      const insertAt = pos === "before" ? toIdx : toIdx + 1;
      const adjusted = fromIdx < toIdx ? insertAt - 1 : insertAt;
      sorted.splice(adjusted, 0, moved!);
      return sorted.map((p, i) => ({ ...p, sortOrder: i }));
    });
    const name = providers().find((p) => p.uuid === fromUuid)?.name ?? "";
    setReorderAnnouncement(`${name} ${props.t.movedDown}`);
    maybeRevertReorder(snapshot);
  };

  const moveProvider = (uuid: string, dir: "up" | "down") => {
    const snapshot = providers(); // capture before reorder
    setProviders((prev) => {
      const sorted = [...prev].sort((a, b) => a.sortOrder - b.sortOrder);
      const idx = sorted.findIndex((p) => p.uuid === uuid);
      if (idx < 0) return prev;
      const target = dir === "up" ? idx - 1 : idx + 1;
      if (target < 0 || target >= sorted.length) return prev;
      [sorted[idx], sorted[target]] = [sorted[target], sorted[idx]];
      return sorted.map((p, i) => ({ ...p, sortOrder: i }));
    });
    const name = providers().find((p) => p.uuid === uuid)?.name ?? "";
    setReorderAnnouncement(`${name} ${dir === "up" ? props.t.movedUp : props.t.movedDown}`);
    maybeRevertReorder(snapshot);
  };

  // Persist failure → revert ONLY sortOrder (snapshot before reorder),
  // preserving all other provider modifications (endpoint, key, roles, etc.)
  const maybeRevertReorder = (preReorderProviders: MockProvider[]) => {
    if (props.state === "reorder-failed") {
      setReorderPending(true);
      scheduleTracked(() => {
        const sortOrderSnapshot = new Map(preReorderProviders.map((p) => [p.uuid, p.sortOrder]));
        setProviders((prev) => prev.map((p) => ({
          ...p,
          sortOrder: sortOrderSnapshot.get(p.uuid) ?? p.sortOrder,
        })));
        setReorderAnnouncement(props.t.reorderReverted);
        pushToast("destructive", props.t.reorderReverted);
        setReorderPending(false);
      }, 800);
    }
  };

  // --- drag-to-reorder (HTML5 DnD) ---
  const handleDragStart = (e: DragEvent, uuid: string) => {
    // Must set dataTransfer or some browsers won't initiate the drag
    e.dataTransfer?.setData("text/plain", uuid);
    e.dataTransfer!.effectAllowed = "move";
    setDraggedUuid(uuid);
  };
  const handleDragOver = (e: DragEvent, uuid: string) => {
    e.preventDefault();
    if (!draggedUuid() || draggedUuid() === uuid) return;
    const row = (e.currentTarget as HTMLElement);
    const rect = row.getBoundingClientRect();
    const midpoint = rect.top + rect.height / 2;
    setDragOverUuid(uuid);
    setDragOverPos(e.clientY < midpoint ? "before" : "after");
  };
  const handleDrop = (uuid: string) => {
    const dragged = draggedUuid();
    if (dragged && dragged !== uuid) {
      reorderProviders(dragged, uuid, dragOverPos());
    }
    setDraggedUuid(null);
    setDragOverUuid(null);
  };
  const handleDragEnd = () => {
    setDraggedUuid(null);
    setDragOverUuid(null);
  };

  // --- state-driven rendering ---
  const showEmpty = createMemo(() => props.state === "empty" && providers().length === 0);
  const sortedProviders = createMemo(() =>
    [...providers()].sort((a, b) => a.sortOrder - b.sortOrder),
  );
  const selectedProvider = createMemo(() =>
    providers().find((p) => p.uuid === selectedUuid()),
  );
  // Effective model = override if set, else provider's stored model
  const selectedModel = createMemo(() => {
    const uuid = selectedUuid();
    if (!uuid) return null;
    return modelDraftByUuid()[uuid] ?? selectedProvider()?.model ?? null;
  });

  // --- fetch balance: CAS registry (per-UUID) ---
  const handleFetchBalance = () => {
    const targetUuid = selectedUuid();
    if (!targetUuid) return;
    const token = opRegistry.startOp(
      "balance" as OpKind,
      targetUuid,
      () => setBalanceByUuid((prev) => ({ ...prev, [targetUuid]: "idle" })),
      () => {
        // Result depends on the state fixture
        const st = props.state;
        if (st === "balance-unsupported") setBalanceByUuid((prev) => ({ ...prev, [targetUuid]: "unsupported" }));
        else if (st === "balance-rate-limited") setBalanceByUuid((prev) => ({ ...prev, [targetUuid]: "rate-limited" }));
        else if (st === "balance-error") setBalanceByUuid((prev) => ({ ...prev, [targetUuid]: "error" }));
        else setBalanceByUuid((prev) => ({ ...prev, [targetUuid]: "done" }));
      },
      1000,
    );
    setBalanceByUuid((prev) => ({ ...prev, [targetUuid]: "loading" }));
    void token;
  };

  // Per-UUID derived states
  const isSaving = createMemo(() => {
    const uuid = selectedUuid();
    return uuid ? (saveByUuid()[uuid] === "saving") : false;
  });
  const isProfileSaving = createMemo(() => {
    const uuid = selectedUuid();
    return uuid ? (profileSaveByUuid()[uuid] === "saving") : false;
  });
  // profileDirty: draft exists AND differs from committed provider value
  const profileDirty = (uuid: string): boolean => {
    const p = providers().find((x) => x.uuid === uuid);
    if (!p) return false;
    const epDraft = endpointDraft()[uuid];
    const modelDraft = modelDraftByUuid()[uuid];
    const epDirty = epDraft !== undefined && epDraft !== p.endpoint;
    const modelDirty = modelDraft !== undefined && modelDraft !== (p.model ?? "");
    return epDirty || modelDirty;
  };
  const connForSelected = createMemo(() => {
    const uuid = selectedUuid();
    return uuid ? (connStatus()[uuid] ?? "idle") : "idle";
  });
  const modelFetchForSelected = createMemo(() => {
    const uuid = selectedUuid();
    return uuid ? (modelFetchByUuid()[uuid] ?? "idle") : "idle";
  });
  const balanceForSelected = createMemo(() => {
    const uuid = selectedUuid();
    return uuid ? (balanceByUuid()[uuid] ?? "idle") : "idle";
  });

  // Consent recipients for the dialog — show names + origins + local/remote
  const consentRecipients = createMemo(() => {
    const sel = pendingParallel()
      ? { ...selection(), parallelUuids: [...selection().parallelUuids, pendingParallel()!] }
      : selection();
    const scope = buildConsentScope(sel, providers());
    return scope.recipients.map((r) => {
      const p = providers().find((x) => x.uuid === r.providerUuid);
      return {
        name: p?.name ?? r.providerUuid,
        origin: r.endpointOrigin,
        localLabel: r.isLocal ? props.t.consentLocal : props.t.consentRemote,
      };
    });
  });

  return (
    <div class="pc__body" role="region" aria-label={props.t.states[props.state]}>
      {/* Settings shell: nav rail (icon-only at 600-699px) + content */}
      <div class="pc__settings-shell">
        <nav class="pc__settings-rail" aria-label={props.t.navSettings}>
          {/* Active nav item — real button, keyboard-focusable */}
          <button
            type="button"
            class="pc__rail-item lr-focusable pc__rail-item--active"
            aria-current="page"
            aria-label={props.t.navProviderCenter}
            title={props.t.navProviderCenter}
          >
            <Server size={20} aria-hidden="true" />
            <span class="pc__rail-item__label">{props.t.navProviderCenter}</span>
          </button>
          {/* Disabled nav items — focusable (not native disabled) with aria-disabled */}
          <button
            type="button"
            class="pc__rail-item lr-focusable"
            aria-disabled="true"
            aria-label={props.t.navShortcuts}
            title={props.t.navShortcuts}
            tabindex="0"
            onClick={(e) => e.preventDefault()}
            onKeyDown={(e) => { if (e.key === "Enter" || e.key === " ") e.preventDefault(); }}
          >
            <Keyboard size={20} aria-hidden="true" />
            <span class="pc__rail-item__label">{props.t.navShortcuts}</span>
          </button>
          <button
            type="button"
            class="pc__rail-item lr-focusable"
            aria-disabled="true"
            aria-label={props.t.navPrivacy}
            title={props.t.navPrivacy}
            tabindex="0"
            onClick={(e) => e.preventDefault()}
            onKeyDown={(e) => { if (e.key === "Enter" || e.key === " ") e.preventDefault(); }}
          >
            <Shield size={20} aria-hidden="true" />
            <span class="pc__rail-item__label">{props.t.navPrivacy}</span>
          </button>
        </nav>
        <div class="pc__content">
      <div class="pc__layout">
        {/* Sidebar: provider list */}
        <aside class="pc__sidebar" aria-label={props.t.providerListLabel} tabindex="-1" ref={(el) => { sidebarFallbackRef.current = el; }}>
          <div class="pc__sidebar-header">
            <h2 class="pc__sidebar-title">{props.t.addProvider}</h2>
            <Button
              variant="primary"
              size="sm"
              leftIcon={<Plus size={14} />}
              onClick={() => setShowPresetGrid((v) => !v)}
            >
              {props.t.addProvider}
            </Button>
          </div>

          {/* Preset grid (add provider) */}
          <Show when={showPresetGrid()}>
            <div class="pc__preset-grid">
              <For each={PRESETS}>
                {(preset) => (
                  <button
                    type="button"
                    class="lr-icon-btn lr-focusable lr-icon-btn--ghost lr-icon-btn--md pc__preset"
                    onClick={() => handleAddPreset(preset)}
                  >
                    <span>{preset.name ?? props.t.presetOllama}</span>
                  </button>
                )}
              </For>
            </div>
          </Show>

          <Show
            when={!showEmpty()}
            fallback={
              <EmptyState
                icon={<Server size={32} />}
                title={props.t.addFirst}
                description={props.t.addFirstDesc}
              />
            }
          >
            {/* Reorder announcement (aria-live) */}
            <div class="lr-visually-hidden" role="status" aria-live="polite">
              {reorderAnnouncement()}
            </div>

            <div class="pc__provider-list">
              <For each={sortedProviders()}>
                {(p, index) => (
                  <div
                    class="pc__provider-row"
                    classList={{
                      "pc__provider-row--dragging": draggedUuid() === p.uuid,
                      "pc__provider-row--drag-over-before": dragOverUuid() === p.uuid && dragOverPos() === "before",
                      "pc__provider-row--drag-over-after": dragOverUuid() === p.uuid && dragOverPos() === "after",
                    }}
                    data-status={p.status}
                    onDragOver={(e: DragEvent) => handleDragOver(e, p.uuid)}
                    onDrop={() => handleDrop(p.uuid)}
                  >
                    <Show when={p.status === "deleting"}>
                      <div class="pc__deleting-overlay">
                        <Spinner size={16} label={props.t.deleting} />
                      </div>
                    </Show>
                    {/* Drag handle — draggable is HERE, not on the row */}
                    <button
                      type="button"
                      class="pc__drag-handle lr-focusable"
                      aria-label={props.t.dragHandle}
                      draggable={p.status === "active" && !reorderPending()}
                      disabled={p.status !== "active" || reorderPending()}
                      onDragStart={(e: DragEvent) => handleDragStart(e, p.uuid)}
                      onDragEnd={() => handleDragEnd()}
                    >
                      <GripVertical size={16} aria-hidden="true" />
                    </button>
                    <ProviderCard
                      profile={{ name: p.name, template: p.template, status: p.status }}
                      hasKey={p.hasKey}
                      role={roleFor(p.uuid)}
                      enabled={p.enabled}
                      onToggle={(en) => handleToggle(p.uuid, en)}
                      onEdit={() => handleEdit(p.uuid)}
                      onDelete={(triggerEl) => handleDelete(p.uuid, triggerEl)}
                      labels={cardLabels()}
                    />
                    {/* Reorder controls — buttons with aria-label (accessible
                        name). Tooltip omitted to avoid nested-interactive
                        (Kobante Trigger renders its own focusable wrapper). */}
                    <div class="pc__reorder-controls">
                      <button
                        type="button"
                        class="lr-icon-btn lr-focusable lr-icon-btn--ghost lr-icon-btn--sm"
                        aria-label={props.t.moveUp}
                        title={props.t.moveUp}
                        disabled={index() === 0 || p.status === "deleting" || reorderPending()}
                        onClick={() => moveProvider(p.uuid, "up")}
                      >
                        <ArrowUp size={14} aria-hidden="true" />
                      </button>
                      <button
                        type="button"
                        class="lr-icon-btn lr-focusable lr-icon-btn--ghost lr-icon-btn--sm"
                        aria-label={props.t.moveDown}
                        title={props.t.moveDown}
                        disabled={index() === sortedProviders().length - 1 || p.status === "deleting" || reorderPending()}
                        onClick={() => moveProvider(p.uuid, "down")}
                      >
                        <ArrowDown size={14} aria-hidden="true" />
                      </button>
                    </div>
                    {/* Role / action menu */}
                    <div class="pc__role-actions">
                      <Show when={roleFor(p.uuid).kind !== "primary" && p.enabled}>
                        <Button variant="ghost" size="sm" onClick={() => handleSetPrimary(p.uuid)}>
                          {props.t.setPrimary}
                        </Button>
                      </Show>
                      <Show when={roleFor(p.uuid).kind !== "parallel" && p.enabled}>
                        <Button variant="ghost" size="sm" onClick={(e) => handleAddParallel(p.uuid, e.currentTarget)}>
                          {props.t.addParallel}
                        </Button>
                      </Show>
                      <Show when={roleFor(p.uuid).kind === "parallel"}>
                        <Button variant="ghost" size="sm" onClick={() => handleRemoveParallel(p.uuid)}>
                          {props.t.removeParallel}
                        </Button>
                      </Show>
                      <Show when={TRADITIONAL_TEMPLATES.has(p.template) && roleFor(p.uuid).kind !== "fallback" && p.enabled}>
                        <Button variant="ghost" size="sm" onClick={() => handleSetFallback(p.uuid)}>
                          {props.t.setFallback}
                        </Button>
                      </Show>
                      <Button variant="ghost" size="sm" leftIcon={<Copy size={14} />} onClick={() => handleDuplicate(p.uuid)}>
                        {props.t.duplicate}
                      </Button>
                    </div>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </aside>

        {/* Detail panel */}
        <section class="pc__detail" aria-label={props.t.detailLabel}>
          <Show when={selectedProvider()} fallback={<p class="pc__select-primary">{props.t.selectPrimary}</p>}>
            {(p) => (
              <div class="pc__detail-content">
                <h3 class="pc__detail-title">{p().name}</h3>

                {/* Endpoint — editable draft (not read-only) */}
                <TextField
                  label={props.t.endpoint}
                  value={endpointDraft()[p().uuid] ?? p().endpoint}
                  disabled={isSaving() || (profileSaveByUuid()[p().uuid] === "saving")}
                  onInput={(e) => setEndpointDraft((prev) => ({ ...prev, [p().uuid]: e.currentTarget.value }))}
                  errorText={endpointErrorByUuid()[p().uuid] ?? (props.state === "endpoint-invalid" ? props.t.endpointInvalid : undefined)}
                />
                {/* Profile Save button — commits endpoint+model draft */}
                <Show when={profileDirty(p().uuid)}>
                  <Button
                    variant="primary"
                    size="sm"
                    loading={isProfileSaving()}
                    loadingLabel={props.t.saving}
                    onClick={() => handleSaveProfile(p().uuid)}
                  >
                    {props.t.saveProfile}
                  </Button>
                </Show>

                {/* Model select / manual entry */}
                <FlowSwitch>
                  <Match when={modelFetchForSelected() === "error" || props.state === "model-manual-entry"}>
                    <TextField
                      label={props.t.manualModelEntry}
                      value={modelDraftByUuid()[p().uuid] ?? ""}
                      placeholder={props.t.manualModelPlaceholder}
                      disabled={isSaving() || isProfileSaving()}
                      onInput={(e) => setModelDraftByUuid((prev) => ({ ...prev, [p().uuid]: e.currentTarget.value }))}
                      helperText={modelFetchForSelected() === "error" || props.state === "model-fetch-error" ? props.t.modelFetchError : undefined}
                    />
                  </Match>
                  <Match when={true}>
                    <Select
                      label={props.t.models}
                      value={selectedModel()}
                      options={MODEL_OPTIONS}
                      onChange={(v) => {
                        const uuid = selectedUuid();
                        if (uuid) setModelDraftByUuid((prev) => ({ ...prev, [uuid]: v }));
                      }}
                      disabled={isSaving() || isProfileSaving()}
                      loading={modelFetchForSelected() === "loading" || props.state === "loading-models"}
                      loadingLabel={props.t.loadingModels}
                      errorText={modelFetchForSelected() === "error" || props.state === "model-fetch-error" ? props.t.modelFetchError : undefined}
                    />
                    <Button variant="ghost" size="sm" onClick={handleFetchModels} disabled={isSaving() || isProfileSaving()}>
                      {props.t.fetchModels}
                    </Button>
                  </Match>
                </FlowSwitch>

                {/* API key */}
                <Show
                  when={p().hasKey}
                  fallback={
                    <div class="pc__key-section">
                      <TextField
                        label={props.t.apiKey}
                        type="password"
                        value={keyInputByUuid()[p().uuid] ?? ""}
                        placeholder={props.t.apiKeyPlaceholder}
                        disabled={isSaving() || isProfileSaving()}
                        onInput={(e) => setKeyInputByUuid((prev) => ({ ...prev, [p().uuid]: e.currentTarget.value }))}
                      />
                      <Button
                        variant="primary"
                        size="md"
                        loading={isSaving()}
                        loadingLabel={props.t.saving}
                        onClick={handleSaveKey}
                      >
                        {props.t.saveKey}
                      </Button>
                    </div>
                  }
                >
                  <div class="pc__key-saved">
                    <Check size={16} aria-hidden="true" />
                    <span>{props.t.keySaved}</span>
                  </div>
                </Show>

                {/* Connection test */}
                <div class="pc__conn-section">
                  <Button
                    variant="secondary"
                    size="md"
                    loading={connForSelected() === "testing" || props.state === "connection-testing"}
                    loadingLabel={props.t.testing}
                    onClick={handleTestConnection}
                    disabled={isSaving() || isProfileSaving()}
                  >
                    {props.t.testConnection}
                  </Button>
                  <Show when={connForSelected() === "ok" || props.state === "connection-ok"}>
                    <span class="pc__conn-ok">
                      <Check size={14} aria-hidden="true" />
                      {props.t.connectionOk} · {connLatency()[selectedUuid()!] ?? 42}ms
                    </span>
                  </Show>
                  <Show when={connForSelected() === "failed" || props.state === "connection-failed"}>
                    <span class="pc__conn-fail">
                      <X size={14} aria-hidden="true" />
                      {props.t.connectionFailed}
                    </span>
                  </Show>
                </div>

                {/* Balance — fetch button + loading→result transition.
                    Reads per-UUID balanceForSelected only (no props.state
                    static branches). The state fixture pre-sets the result
                    for demo states; user Fetch triggers loading→result. */}
                <div class="pc__balance-section">
                  <FlowSwitch>
                    <Match when={balanceForSelected() === "loading"}>
                      <span class="pc__balance">{props.t.balanceLoading}</span>
                    </Match>
                    <Match when={balanceForSelected() === "unsupported"}>
                      <span class="pc__balance">{props.t.balanceUnsupported}</span>
                    </Match>
                    <Match when={balanceForSelected() === "rate-limited"}>
                      <span class="pc__balance">{props.t.balanceRateLimited}</span>
                    </Match>
                    <Match when={balanceForSelected() === "error"}>
                      <span class="pc__balance">{props.t.balanceError}</span>
                    </Match>
                    <Match when={balanceForSelected() === "done"}>
                      <span class="pc__balance">$12.50</span>
                    </Match>
                    <Match when={true}>
                      <Button
                        variant="ghost"
                        size="sm"
                        loadingLabel={props.t.balanceLoading}
                        onClick={handleFetchBalance}
                        disabled={isSaving() || isProfileSaving()}
                      >
                        {props.t.balanceLoading.replace("…", "")}
                      </Button>
                    </Match>
                  </FlowSwitch>
                </div>
              </div>
            )}
          </Show>
        </section>
      </div>
        </div>{/* close pc__content */}
      </div>{/* close pc__settings-shell */}

      {/* Save conflict banner (uses Banner component) */}
      <Show when={props.state === "save-conflict" && !conflictResolved()}>
        <Banner
          variant="warning"
          title={props.t.saveConflict}
          action={
            <>
              <Button variant="secondary" size="sm" onClick={handleConflictReload}>
                {props.t.reload}
              </Button>
              <Button variant="ghost" size="sm" onClick={handleConflictCancel}>
                {props.t.cancel}
              </Button>
            </>
          }
        />
      </Show>

      {/* Delete retry */}
      <Show when={props.state === "delete-retry" && retryTargetUuid()}>
        <div class="pc__delete-retry">
          <span>{props.t.deleteRetry}</span>
          <Button variant="destructive" size="sm" loading={deleteRetryPending()} loadingLabel={props.t.deleting} onClick={handleDeleteRetry}>
            {props.t.delete}
          </Button>
        </div>
      </Show>

      {/* Toasts */}
      <div class="pc__toasts">
        <For each={toasts()}>
          {(toast) => (
            <Toast
              variant={toast.variant}
              message={toast.message}
              dismissLabel={props.t.toastDismiss}
              onDismiss={() => dismissToast(toast.id)}
            />
          )}
        </For>
      </div>

      {/* Delete confirm dialog */}
      <Confirm
        open={deleteConfirmUuid() !== null}
        onOpenChange={(open) => { if (!open) cancelDelete(); }}
        title={props.t.deleteConfirmTitle}
        message={props.t.deleteConfirmMsg}
        confirmLabel={props.t.delete}
        cancelLabel={props.t.cancel}
        variant="destructive"
        onConfirm={confirmDelete}
        onCancel={cancelDelete}
        triggerRef={deleteTriggerRef}
        fallbackFocusRef={sidebarFallbackRef}
      />

      {/* Consent dialog */}
      <Confirm
        open={consentOpen()}
        onOpenChange={(open) => { if (!open) cancelConsent(); }}
        title={props.t.consentTitle}
        message={props.t.consentMsg}
        confirmLabel={props.t.consentConfirm}
        cancelLabel={props.t.consentCancel}
        variant="primary"
        onConfirm={confirmConsent}
        onCancel={cancelConsent}
        triggerRef={consentTriggerRef}
      >
        <ul class="pc__consent-list">
          <For each={consentRecipients()}>
            {(r) => (
              <li>
                <Globe size={12} aria-hidden="true" />
                <span>{r.name}</span>
                <span class="pc__consent-origin">{r.origin}</span>
                <span class="pc__consent-local">{r.localLabel}</span>
              </li>
            )}
          </For>
        </ul>
      </Confirm>
    </div>
  );
};

export default ProviderCenter;
