/**
 * Provider Center (Surface 05) — production component on real IPC.
 *
 * Ported from `apps/ui-lab/src/pages/ProviderCenter.tsx` and rewired to the
 * Task 2 IPC wrappers. Mock-fixture/state-driver props and the OpRegistry
 * mock timers are DELETED — real IPC awaits replace them.
 *
 * Layout divergence from the lab: the lab used `ProviderCard` (NOT exported
 * from `@linguaray/ui`). This production build uses `ProviderRow` for the
 * list rows, with role-assignment (set-primary / add-parallel / set-fallback)
 * and reorder actions rendered as sibling icon buttons in a row wrapper
 * (ProviderRow has no `extraActions` slot).
 *
 * Known R3a limitations (degrade gracefully + TODO, do NOT touch backend):
 *  1. Active selection cold-loads via `provider_get_active_selection` (fail-
 *     closed: a read failure blocks `providerSetActive` until a retry succeeds).
 *  2. Balance / quota introspection not implemented — the balance section
 *     renders a muted TODO note (no fetch button).
 */
import {
  For,
  Show,
  createSignal,
  createMemo,
  onMount,
  type Component,
} from "solid-js";
import {
  Server,
  Plus,
  ArrowUp,
  ArrowDown,
  Star,
  Layers,
  CornerDownLeft,
  Copy,
} from "lucide-solid";
import {
  ProviderRow,
  Button,
  TextField,
  Select,
  Confirm,
  Toast,
  EmptyState,
  StatusBadge,
  InlineError,
  type ProviderRowLabels,
  type SelectOption,
} from "@linguaray/ui";
import { SETTINGS_COPY } from "./copy";
import { detectLocale } from "../../i18n";
import type {
  ProviderProfileFE,
  ActiveSelection,
  ConnectionResult,
  ModelInfo,
} from "./provider-types";
import {
  loadProviders,
  providerCreate,
  providerUpdate,
  providerDuplicate,
  providerDelete,
  providerReorder,
  providerToggle,
  providerSetKey,
  providerSetActive,
  providerConfirmAndSetActive,
  providerTestConnection,
  providerGetModels,
  providerGetActiveSelection,
} from "./provider-ipc";
import type { ProviderRole } from "@linguaray/ui";
import { validateEndpoint } from "./provider-domain";
import "./ProviderCenter.css";

// --- Presets (template ids from `src-tauri/src/providers.rs`) -------------
// `name` may be null to signal "use the localized Ollama label" at render.
// R2/C2: only the 4 supported AI presets are exposed. Traditional MT engines
// (google / deepl) are no longer offered as presets.
type Preset = { templateId: string; name: string | null; endpoint: string; model: string | null };

/** Escape a literal string for use inside a CSS attribute selector. Provider
 *  names may contain characters that break `button[aria-label="..."]` otherwise. */
const cssEscape = (s: string): string =>
  s.replace(/["\\]/g, (c) => `\\${c}`);
const PRESETS: Preset[] = [
  { templateId: "openai", name: "OpenAI", endpoint: "https://api.openai.com/v1/chat/completions", model: "gpt-4o-mini" },
  { templateId: "anthropic", name: "Anthropic", endpoint: "https://api.anthropic.com/v1/messages", model: "claude-sonnet-4-5" },
  { templateId: "gemini", name: "Gemini", endpoint: "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions", model: "gemini-3.6-flash" },
  { templateId: "ollama", name: null, endpoint: "http://localhost:11434/v1/chat/completions", model: "qwen2.5:7b" },
];

const ProviderCenter: Component = () => {
  const locale = detectLocale();
  const t = SETTINGS_COPY[locale].provider;

  // --- Core state ---
  const [providers, setProviders] = createSignal<ProviderProfileFE[]>([]);
  // Active selection is cold-loaded via `provider_get_active_selection` and then
  // kept in sync as a session mirror. Fail-closed: while loading or after a
  // read failure, role-assignment handlers are disabled (no `providerSetActive`).
  const [selection, setSelection] = createSignal<ActiveSelection>({
    primaryUuid: null,
    parallelUuids: [],
    fallbackUuid: null,
  });
  const [loadError, setLoadError] = createSignal(false);
  // Cold-load in-flight (initial + retries) — gates role mutations.
  const [selectionLoading, setSelectionLoading] = createSignal(true);
  // Cold-load read failed — gates role mutations until a successful retry.
  const [selectionError, setSelectionError] = createSignal(false);

  // --- Detail panel state (per-UUID to prevent cross-provider leakage) ---
  const [selectedUuid, setSelectedUuid] = createSignal<string | null>(null);
  const [keyInputByUuid, setKeyInputByUuid] = createSignal<Record<string, string>>({});
  const [endpointDraft, setEndpointDraft] = createSignal<Record<string, string>>({});
  const [modelDraftByUuid, setModelDraftByUuid] = createSignal<Record<string, string>>({});
  // Per-UUID name draft for the editable name field (mirrors endpoint/model
  // drafts). Undefined → fall back to the stored provider name.
  const [nameDraftByUuid, setNameDraftByUuid] = createSignal<Record<string, string>>({});
  const [saveByUuid, setSaveByUuid] = createSignal<Record<string, "idle" | "saving" | "saved" | "failed">>({});
  // Per-UUID key-save error message (localized). Cleared on the next key edit.
  const [keyErrorByUuid, setKeyErrorByUuid] = createSignal<Record<string, string>>({});
  // Per-UUID name conflict error (structured duplicate-name check). Cleared on
  // the next name edit.
  const [nameErrorByUuid, setNameErrorByUuid] = createSignal<Record<string, string>>({});
  const [connByUuid, setConnByUuid] = createSignal<Record<string, ConnectionResult | "testing">>({});
  const [modelOptionsByUuid, setModelOptionsByUuid] = createSignal<Record<string, ModelInfo[]>>({});
  const [modelFetchByUuid, setModelFetchByUuid] = createSignal<Record<string, "idle" | "loading" | "error">>({});

  // --- Dialogs + toasts ---
  const [deleteConfirmUuid, setDeleteConfirmUuid] = createSignal<string | null>(null);
  const deleteTriggerRef: { current?: HTMLElement } = {};
  // Delete error/retry state machine. On a failed delete the Confirm closes and
  // a Retry banner surfaces in the main area (Kobalte's Dialog sets body
  // pointer-events:none during its close transition, which would swallow clicks
  // on an in-dialog Retry button). `deleteFailedUuid` holds the provider that
  // failed so Retry can re-attempt without re-opening the dialog.
  const [deleteError, setDeleteError] = createSignal(false);
  const [deleteFailedUuid, setDeleteFailedUuid] = createSignal<string | null>(null);
  const [consentOpen, setConsentOpen] = createSignal(false);
  const [pendingParallelUuid, setPendingParallelUuid] = createSignal<string | null>(null);
  const [consentActualScope, setConsentActualScope] = createSignal<string | null>(null);
  const consentTriggerRef: { current?: HTMLElement } = {};
  const [toasts, setToasts] = createSignal<
    { id: number; variant: "info" | "success" | "warning" | "destructive"; message: string }[]
  >([]);

  let toastId = 0;
  const pushToast = (
    variant: "info" | "success" | "warning" | "destructive",
    message: string,
  ) => {
    const id = ++toastId;
    setToasts((prev) => [...prev, { id, variant, message }]);
  };
  const dismissToast = (id: number) =>
    setToasts((prev) => prev.filter((x) => x.id !== id));

  // --- Initial load (fail-closed) ---
  // BOTH the provider list AND the stored active selection must resolve before
  // roles are applied. If either rejects, no `providerSetActive` is allowed
  // (handlers short-circuit on `selectionError()`/`selectionLoading()`).
  const refresh = async () => {
    setSelectionLoading(true);
    setSelectionError(false);
    try {
      const [list, active] = await Promise.all([
        loadProviders(),
        providerGetActiveSelection(),
      ]);
      setProviders(list);
      setSelection({
        primaryUuid: active.primary,
        parallelUuids: active.parallel,
        fallbackUuid: active.fallback,
      });
      setLoadError(false);
    } catch (e) {
      setLoadError(true);
      setSelectionError(true);
      pushToast("destructive", t.saveFailed);
    } finally {
      setSelectionLoading(false);
    }
  };

  onMount(() => {
    void refresh();
  });

  // --- Derived role for a provider (session mirror) ---
  const roleFor = (uuid: string): ProviderRole => {
    const sel = selection();
    if (sel.primaryUuid === uuid) return { kind: "primary" };
    const idx = sel.parallelUuids.indexOf(uuid);
    if (idx >= 0) return { kind: "parallel", index: idx + 1 };
    if (sel.fallbackUuid === uuid) return { kind: "fallback" };
    return { kind: "none" };
  };

  const sortedProviders = createMemo(() =>
    [...providers()].sort((a, b) => a.sort_order - b.sort_order),
  );
  const selectedProvider = createMemo(() =>
    providers().find((p) => p.uuid === selectedUuid()),
  );

  // --- Row labels for ProviderRow (per-provider: {name} interpolated) ---
  const rowLabelsFor = (name: string): ProviderRowLabels => ({
    edit: t.cardEdit.replace("{name}", name),
    delete: t.cardDelete.replace("{name}", name),
    enabled: t.enabled,
    statusText: {
      active: t.role.primary,
      available: t.role.none,
      "key-missing": t.keyMissing,
      disabled: t.disabled,
    },
  });

  // --- Mutations ---

  /** Toggle: optimistic flip → IPC → revert + toast on error. */
  const handleToggle = async (uuid: string, enabled: boolean) => {
    const prev = providers();
    const next = prev.map((p) => (p.uuid === uuid ? { ...p, enabled } : p));
    setProviders(next);
    try {
      await providerToggle(uuid, enabled);
      // Backend evicts slots, but our session mirror must too.
      if (!enabled) {
        setSelection((sel) => ({
          primaryUuid: sel.primaryUuid === uuid ? null : sel.primaryUuid,
          parallelUuids: sel.parallelUuids.filter((u) => u !== uuid),
          fallbackUuid: sel.fallbackUuid === uuid ? null : sel.fallbackUuid,
        }));
      }
    } catch (e) {
      setProviders(prev); // rollback
      pushToast("destructive", t.saveFailed);
    }
  };

  const buildCandidatePrimary = (uuid: string): ActiveSelection => {
    const prev = selection();
    return {
      primaryUuid: uuid,
      parallelUuids: prev.parallelUuids.filter((u) => u !== uuid),
      fallbackUuid: prev.fallbackUuid === uuid ? null : prev.fallbackUuid,
    };
  };

  const handleSetPrimary = async (uuid: string) => {
    // Fail-closed: never call providerSetActive while the cold-load read is
    // in-flight or has failed (prevents overwriting a stored selection we
    // failed to read).
    if (selectionLoading() || selectionError()) return;
    const candidate = buildCandidatePrimary(uuid);
    try {
      const result = await providerSetActive(
        candidate.primaryUuid!,
        candidate.parallelUuids,
        candidate.fallbackUuid,
      );
      if (result.outcome === "written") {
        setSelection(candidate);
      }
      // set-primary alone has empty parallel → always "written".
    } catch (e) {
      pushToast("destructive", t.saveFailed);
    }
  };

  const handleAddParallel = (uuid: string, triggerEl?: HTMLElement) => {
    if (selectionLoading() || selectionError()) return;
    if (triggerEl) consentTriggerRef.current = triggerEl;
    const candidate: ActiveSelection = {
      ...selection(),
      parallelUuids: [...selection().parallelUuids, uuid],
      fallbackUuid: selection().fallbackUuid === uuid ? null : selection().fallbackUuid,
    };
    void (async () => {
      try {
        const result = await providerSetActive(
          candidate.primaryUuid ?? "",
          candidate.parallelUuids,
          candidate.fallbackUuid,
        );
        if (result.outcome === "written") {
          setSelection(candidate);
        } else if (result.outcome === "needs_consent") {
          setPendingParallelUuid(uuid);
          setConsentActualScope(result.actual_scope);
          setConsentOpen(true);
        }
      } catch (e) {
        pushToast("destructive", t.saveFailed);
      }
    })();
  };

  const confirmConsent = async () => {
    const uuid = pendingParallelUuid();
    if (!uuid) return;
    if (selectionLoading() || selectionError()) return;
    const candidate: ActiveSelection = {
      ...selection(),
      parallelUuids: [...selection().parallelUuids, uuid],
      fallbackUuid: selection().fallbackUuid === uuid ? null : selection().fallbackUuid,
    };
    const scope = consentActualScope();
    try {
      await providerConfirmAndSetActive(
        candidate.primaryUuid ?? "",
        candidate.parallelUuids,
        candidate.fallbackUuid,
        scope ?? "",
      );
      setSelection(candidate);
      setConsentOpen(false);
      setPendingParallelUuid(null);
      setConsentActualScope(null);
    } catch (e) {
      const err = e as { error?: string };
      if (err?.error === "stale_scope") {
        pushToast("destructive", t.saveFailed);
      } else {
        pushToast("destructive", t.saveFailed);
      }
      setConsentOpen(false);
      setPendingParallelUuid(null);
      setConsentActualScope(null);
    }
  };

  const cancelConsent = () => {
    setConsentOpen(false);
    setPendingParallelUuid(null);
    setConsentActualScope(null);
  };

  const handleSetFallback = async (uuid: string) => {
    if (selectionLoading() || selectionError()) return;
    const prev = selection();
    const candidate: ActiveSelection = {
      primaryUuid: prev.primaryUuid === uuid ? null : prev.primaryUuid,
      parallelUuids: prev.parallelUuids.filter((u) => u !== uuid),
      fallbackUuid: uuid,
    };
    try {
      const result = await providerSetActive(
        candidate.primaryUuid ?? "",
        candidate.parallelUuids,
        candidate.fallbackUuid,
      );
      if (result.outcome === "written") setSelection(candidate);
    } catch (e) {
      pushToast("destructive", t.saveFailed);
    }
  };

  const handleRemoveParallel = async (uuid: string) => {
    if (selectionLoading() || selectionError()) return;
    const candidate: ActiveSelection = {
      ...selection(),
      parallelUuids: selection().parallelUuids.filter((u) => u !== uuid),
    };
    try {
      const result = await providerSetActive(
        candidate.primaryUuid ?? "",
        candidate.parallelUuids,
        candidate.fallbackUuid,
      );
      if (result.outcome === "written") setSelection(candidate);
    } catch (e) {
      pushToast("destructive", t.saveFailed);
    }
  };

  const handleAddPreset = async (preset: Preset) => {
    const name = preset.name ?? "Ollama";
    try {
      await providerCreate(preset.templateId, name, preset.endpoint, preset.model ?? undefined);
      await refresh();
      pushToast("success", t.profileSaved);
    } catch (e) {
      pushToast("destructive", t.saveFailed);
    }
  };

  /** Duplicate a provider: new UUID, new secret_ref, keyless. Re-fetches the
   *  list so the clone appears. */
  const handleDuplicate = async (uuid: string) => {
    try {
      await providerDuplicate(uuid);
      await refresh();
      pushToast("success", t.profileSaved);
    } catch (e) {
      pushToast("destructive", t.saveFailed);
    }
  };

  /** Save profile: validate endpoint locally (reactive epError already shown),
   *  then IPC. Aborts on invalid endpoint or a duplicate-name conflict. */
  const handleSaveProfile = async (uuid: string) => {
    const draft = endpointDraft()[uuid];
    const modelDraft = modelDraftByUuid()[uuid];
    const nameDraft = nameDraftByUuid()[uuid];
    const provider = providers().find((p) => p.uuid === uuid);
    if (!provider) return;
    const effectiveEndpoint = draft ?? provider.endpoint;
    const epCheck = validateEndpoint(effectiveEndpoint);
    if (!epCheck.ok) {
      // The reactive epError memo will surface the message; abort the save.
      return;
    }
    const effectiveName = (nameDraft ?? provider.name).trim();
    // Structured duplicate-name conflict: the DB has no UNIQUE constraint on
    // name, so enforce uniqueness client-side before the IPC round-trip.
    if (effectiveName !== provider.name) {
      const conflict = providers().some(
        (other) => other.uuid !== uuid && other.name === effectiveName,
      );
      if (conflict) {
        setNameErrorByUuid((prev) => ({ ...prev, [uuid]: t.nameExists }));
        return;
      }
    }
    // Clear any stale name conflict error now that the name is valid.
    setNameErrorByUuid((prev) => {
      const n = { ...prev };
      delete n[uuid];
      return n;
    });
    setSaveByUuid((prev) => ({ ...prev, [uuid]: "saving" }));
    try {
      const updated = await providerUpdate(uuid, {
        name: effectiveName,
        endpoint: effectiveEndpoint,
        model: modelDraft ?? provider.model,
      });
      setProviders((prev) => prev.map((p) => (p.uuid === uuid ? { ...p, ...updated, hasKey: p.hasKey } : p)));
      setSaveByUuid((prev) => ({ ...prev, [uuid]: "saved" }));
      pushToast("success", t.profileSaved);
    } catch (e) {
      setSaveByUuid((prev) => ({ ...prev, [uuid]: "failed" }));
      pushToast("destructive", t.saveFailed);
    }
  };

  /**
   * Save key: clear the input IMMEDIATELY (never re-readable), then IPC.
   * On success re-fetch key_status to update `hasKey`. The key is NEVER
   * re-read from the input after submit — the input stays cleared.
   */
  const handleSaveKey = async (uuid: string) => {
    const key = keyInputByUuid()[uuid];
    // Clear IMMEDIATELY — never readable back, never in DOM after submit.
    setKeyInputByUuid((prev) => {
      const n = { ...prev };
      delete n[uuid];
      return n;
    });
    setSaveByUuid((prev) => ({ ...prev, [uuid]: "saving" }));
    setKeyErrorByUuid((prev) => {
      const n = { ...prev };
      delete n[uuid];
      return n;
    });
    try {
      await providerSetKey(uuid, key);
      setSaveByUuid((prev) => ({ ...prev, [uuid]: "saved" }));
      // Re-fetch to update hasKey.
      const list = await loadProviders();
      setProviders(list);
      pushToast("success", t.keySaved);
    } catch (e) {
      setSaveByUuid((prev) => ({ ...prev, [uuid]: "failed" }));
      // Detect UNIQUE constraint violations and surface a localized "already
      // exists" message; everything else is a generic save-failed.
      const msg = (e as { message?: string })?.message ?? "";
      if (/UNIQUE constraint/i.test(String(msg))) {
        setKeyErrorByUuid((prev) => ({ ...prev, [uuid]: t.keyAlreadyExists }));
        pushToast("destructive", t.keyAlreadyExists);
      } else {
        pushToast("destructive", t.saveFailed);
      }
    }
  };

  const handleFetchModels = async (uuid: string) => {
    setModelFetchByUuid((prev) => ({ ...prev, [uuid]: "loading" }));
    try {
      const models = await providerGetModels(uuid);
      setModelOptionsByUuid((prev) => ({ ...prev, [uuid]: models }));
      setModelFetchByUuid((prev) => ({ ...prev, [uuid]: "idle" }));
    } catch (e) {
      setModelFetchByUuid((prev) => ({ ...prev, [uuid]: "error" }));
    }
  };

  const handleTestConnection = async (uuid: string) => {
    setConnByUuid((prev) => ({ ...prev, [uuid]: "testing" }));
    try {
      const result = await providerTestConnection(uuid);
      setConnByUuid((prev) => ({ ...prev, [uuid]: result }));
    } catch (e) {
      setConnByUuid((prev) => ({
        ...prev,
        [uuid]: { ok: false, message: t.connectionFailed },
      }));
    }
  };

  const confirmDelete = async () => {
    const uuid = deleteConfirmUuid() ?? deleteFailedUuid();
    if (!uuid) return;
    try {
      await providerDelete(uuid);
      setDeleteError(false);
      setDeleteFailedUuid(null);
      setDeleteConfirmUuid(null);
      await refresh();
    } catch (e) {
      // Close the dialog and surface a Retry banner in the main area. Kobalte's
      // Dialog sets body pointer-events:none during its close transition, so an
      // in-dialog Retry button would have its clicks swallowed.
      setDeleteError(true);
      setDeleteFailedUuid(uuid);
      setDeleteConfirmUuid(null);
      pushToast("destructive", t.saveFailed);
    }
  };

  /** Retry a failed delete (re-attempts providerDelete for the failed uuid). */
  const retryDelete = () => {
    void confirmDelete();
  };

  /** Dismiss the delete-error banner (gives up on retry). */
  const dismissDeleteError = () => {
    setDeleteError(false);
    setDeleteFailedUuid(null);
  };

  /** Cancel the delete dialog: clear error/attempts and restore focus to the
   *  delete trigger button. We focus the trigger explicitly here (in addition
   *  to the Confirm's onCloseAutoFocus) because Kobalte's auto-focus restore is
   *  unreliable under jsdom's synthesized events. */
  const cancelDelete = () => {
    setDeleteConfirmUuid(null);
    setDeleteError(false);
    setDeleteFailedUuid(null);
    // Restore focus on the next tick (after the dialog unmounts) so the trigger
    // is the active element when the row re-renders.
    queueMicrotask(() => {
      deleteTriggerRef.current?.focus();
    });
  };

  /** Reorder: optimistic local swap → IPC → revert + toast on error. */
  const moveProvider = async (uuid: string, dir: "up" | "down") => {
    const ordered = sortedProviders();
    const idx = ordered.findIndex((p) => p.uuid === uuid);
    if (idx < 0) return;
    const swap = dir === "up" ? idx - 1 : idx + 1;
    if (swap < 0 || swap >= ordered.length) return;
    const snapshot = [...ordered];
    const newOrder = [...ordered];
    [newOrder[idx], newOrder[swap]] = [newOrder[swap], newOrder[idx]];
    // Optimistic: re-number sort_order.
    const renumbered = newOrder.map((p, i) => ({ ...p, sort_order: i }));
    setProviders(renumbered);
    try {
      await providerReorder(renumbered.map((p) => p.uuid));
    } catch (e) {
      // Revert to snapshot order.
      setProviders(
        snapshot.map((p, i) => ({ ...p, sort_order: i })),
      );
      pushToast("destructive", t.reorderReverted);
    }
  };

  // --- Render helpers ---
  const consentRecipients = createMemo(() => {
    const sel = pendingParallelUuid()
      ? {
          ...selection(),
          parallelUuids: [...selection().parallelUuids, pendingParallelUuid()!],
        }
      : selection();
    const recipientUuids = [sel.primaryUuid, ...sel.parallelUuids].filter(
      (u): u is string => u !== null,
    );
    return recipientUuids.map((uuid) => {
      const p = providers().find((x) => x.uuid === uuid);
      return {
        name: p?.name ?? uuid,
        localLabel: p?.is_local ? t.consent.local : t.consent.remote,
      };
    });
  });

  return (
    <div class="pc__body" role="region" aria-label={t.providerListLabel}>
      <Show when={loadError()}>
        <InlineError>{t.saveFailed}</InlineError>
        <div class="pc__retry">
          <Button variant="primary" size="sm" onClick={() => void refresh()}>
            {t.reload}
          </Button>
        </div>
      </Show>

      {/* Cold-load failure: selection read failed → fail-closed. Role mutations
          are disabled (see handler guards) until a successful retry. */}
      <Show when={selectionError()}>
        <div class="pc__retry" role="alert">
          <span class="pc__load-failed">{t.loadFailed}</span>
          <Button variant="secondary" size="sm" onClick={() => void refresh()}>
            {t.retry}
          </Button>
        </div>
      </Show>

      <div class="pc__layout">
        {/* Sidebar: provider list */}
        <aside class="pc__sidebar" aria-label={t.providerListLabel}>
          <div class="pc__sidebar-header">
            <h2 class="pc__sidebar-title">{t.addProvider}</h2>
          </div>

          <Show
            when={providers().length > 0}
            fallback={
              <EmptyState
                icon={<Server size={32} />}
                title={t.empty.title}
                description={t.empty.description}
              />
            }
          >
            <ul class="pc__provider-list" role="list">
              <For each={sortedProviders()}>
                {(p) => {
                  // role is a reactive accessor: re-evaluated when selection()
                  // changes, so role-action visibility + badges update in place.
                  const role = () => roleFor(p.uuid);
                  return (
                    <li class="pc__provider-row-wrapper" data-status={p.status}>
                      <div class="pc__provider-row-main">
                        <ProviderRow
                          name={p.name}
                          template={p.template_id}
                          hasKey={p.hasKey}
                          role={role()}
                          enabled={p.enabled}
                          active={selectedUuid() === p.uuid}
                          labels={rowLabelsFor(p.name)}
                          onToggle={(enabled) => void handleToggle(p.uuid, enabled)}
                          onEdit={() => setSelectedUuid(p.uuid)}
                          onDelete={() => {
                            // Capture the delete trigger button so the Confirm's
                            // onCloseAutoFocus can restore focus to it on cancel.
                            // ProviderRow.onDelete carries no event/element, so
                            // resolve the button by its deterministic aria-label
                            // ("Delete {name}"). document.activeElement is
                            // unreliable here — fireEvent.click in jsdom does not
                            // move focus to the button, and a real click may have
                            // already blurred it by the time the handler runs.
                            const label = t.cardDelete.replace("{name}", p.name);
                            const btn = document.querySelector<HTMLButtonElement>(
                              `button[aria-label="${cssEscape(label)}"]`,
                            );
                            deleteTriggerRef.current = btn ?? undefined;
                            setDeleteError(false);
                            setDeleteFailedUuid(null);
                            setDeleteConfirmUuid(p.uuid);
                          }}
                        />
                        {/* Role-action icon buttons (ProviderRow has no slot).
                            Role-assign buttons (set-primary / add-parallel /
                            set-fallback / remove-parallel) are hidden for
                            disabled providers — a disabled provider cannot hold
                            a role. Reorder + duplicate remain available. */}
                        <div class="pc__role-actions">
                          <Show when={p.enabled}>
                            <Show when={role().kind !== "primary"}>
                              <button
                                type="button"
                                class="pc__icon-btn"
                                aria-label={t.setPrimary}
                                title={t.setPrimary}
                                onClick={() => void handleSetPrimary(p.uuid)}
                              >
                                <Star size={14} />
                              </button>
                            </Show>
                            <Show when={role().kind === "parallel"}>
                              <button
                                type="button"
                                class="pc__icon-btn"
                                aria-label={t.removeParallel}
                                title={t.removeParallel}
                                onClick={() => void handleRemoveParallel(p.uuid)}
                              >
                                <Layers size={14} />
                              </button>
                            </Show>
                            <Show when={role().kind !== "parallel" && role().kind !== "primary"}>
                              <button
                                type="button"
                                class="pc__icon-btn"
                                aria-label={t.addParallel}
                                title={t.addParallel}
                                onClick={(e) =>
                                  handleAddParallel(
                                    p.uuid,
                                    e.currentTarget as unknown as HTMLElement,
                                  )
                                }
                              >
                                <Layers size={14} />
                              </button>
                            </Show>
                            <Show when={role().kind !== "fallback" && role().kind !== "primary"}>
                              <button
                                type="button"
                                class="pc__icon-btn"
                                aria-label={t.setFallback}
                                title={t.setFallback}
                                onClick={() => void handleSetFallback(p.uuid)}
                              >
                                <CornerDownLeft size={14} />
                              </button>
                            </Show>
                          </Show>
                          {/* Duplicate */}
                          <button
                            type="button"
                            class="pc__icon-btn"
                            aria-label={t.duplicate}
                            title={t.duplicate}
                            onClick={() => void handleDuplicate(p.uuid)}
                          >
                            <Copy size={14} />
                          </button>
                          {/* Reorder */}
                          <button
                            type="button"
                            class="pc__icon-btn"
                            aria-label={t.moveUp}
                            title={t.moveUp}
                            onClick={() => void moveProvider(p.uuid, "up")}
                          >
                            <ArrowUp size={14} />
                          </button>
                          <button
                            type="button"
                            class="pc__icon-btn"
                            aria-label={t.moveDown}
                            title={t.moveDown}
                            onClick={() => void moveProvider(p.uuid, "down")}
                          >
                            <ArrowDown size={14} />
                          </button>
                        </div>
                      </div>
                      {/* Role badge row (cold-loaded + session mirror). On read
                          failure all roles render as "none" (fail-closed). */}
                      <Show when={role().kind !== "none"}>
                        <div class="pc__role-badge-row">
                          <Show when={role().kind === "primary"}>
                            <StatusBadge variant="success">{t.role.primary}</StatusBadge>
                          </Show>
                          <Show when={role().kind === "parallel"}>
                            <StatusBadge variant="info">
                              {t.role.parallel} {(role() as { kind: "parallel"; index: number }).index}
                            </StatusBadge>
                          </Show>
                          <Show when={role().kind === "fallback"}>
                            <StatusBadge variant="neutral">{t.role.fallback}</StatusBadge>
                          </Show>
                        </div>
                      </Show>
                    </li>
                  );
                }}
              </For>
            </ul>
          </Show>

          {/* Preset grid (always visible — add provider). */}
          <div class="pc__preset-grid">
            <For each={PRESETS}>
              {(preset) => (
                <button
                  type="button"
                  class="pc__preset"
                  onClick={() => void handleAddPreset(preset)}
                >
                  <Plus size={12} />
                  <span>{preset.name ?? "Ollama"}</span>
                </button>
              )}
            </For>
          </div>
        </aside>

        {/* Detail panel */}
        <section class="pc__detail" aria-label={t.detailLabel}>
          <Show
            when={selectedProvider()}
            fallback={
              <EmptyState icon={<Server size={32} />} title={t.selectPrimary} />
            }
          >
            {(p) => {
              const uuid = p().uuid;
              const draftEndpoint = createMemo(() => endpointDraft()[uuid] ?? p().endpoint);
              const draftModel = createMemo(() => modelDraftByUuid()[uuid] ?? p().model ?? "");
              const draftName = createMemo(() => nameDraftByUuid()[uuid] ?? p().name);
              const nameError = createMemo(() => nameErrorByUuid()[uuid]);
              const epError = createMemo(() => {
                // Reactive: validate the draft as it changes (not just on save).
                const draft = draftEndpoint();
                // Don't show an error before the user has touched the field
                // (the stored endpoint is always valid). Only validate drafts
                // that differ from the stored value.
                if (draft === p().endpoint) return undefined;
                const check = validateEndpoint(draft);
                if (!check.ok) return t.endpoint.errors[check.code];
                return undefined;
              });
              const conn = createMemo(() => connByUuid()[uuid]);
              const saveState = createMemo(() => saveByUuid()[uuid] ?? "idle");
              const keyText = createMemo(() => keyInputByUuid()[uuid] ?? "");
              const keyError = createMemo(() => keyErrorByUuid()[uuid]);
              const options = createMemo(() => modelOptionsByUuid()[uuid]);
              const modelFetch = createMemo(() => modelFetchByUuid()[uuid] ?? "idle");
              // Only providers that advertise model_list can fetch a dropdown;
              // others get the manual-entry input directly.
              const canListModels = createMemo(() => p().capabilities.model_list);
              const selectOptions = createMemo<SelectOption[]>(() => {
                const opts = options();
                if (opts && opts.length > 0) {
                  return opts.map((m) => ({ value: m.id, label: m.label, disabled: false }));
                }
                // Manual entry fallback: current model as the only option.
                return [{ value: draftModel() || "—", label: draftModel() || "—", disabled: false }];
              });
              return (
                <div class="pc__detail-content">
                  {/* Name (editable — backend supports `name` patches). */}
                  <TextField
                    label={t.name}
                    value={draftName()}
                    errorText={nameError() ?? undefined}
                    onInput={(e) => {
                      setNameDraftByUuid((prev) => ({
                        ...prev,
                        [uuid]: e.currentTarget.value,
                      }));
                      // Clear any stale conflict error as soon as the user
                      // edits the name again.
                      if (nameError()) {
                        setNameErrorByUuid((prev) => {
                          const n = { ...prev };
                          delete n[uuid];
                          return n;
                        });
                      }
                    }}
                  />

                  {/* Endpoint */}
                  <TextField
                    label={t.endpoint.label}
                    value={draftEndpoint()}
                    placeholder={t.endpoint.placeholder}
                    errorText={epError() ?? undefined}
                    onInput={(e) =>
                      setEndpointDraft((prev) => ({
                        ...prev,
                        [uuid]: e.currentTarget.value,
                      }))
                    }
                  />

                  {/* Model: dropdown + Fetch models only when the provider
                      advertises model_list; otherwise a manual-entry input.
                      A fetch error also falls back to the manual input. */}
                  <Show
                    when={modelFetch() !== "error" && canListModels()}
                    fallback={
                      <TextField
                        label={t.models}
                        value={draftModel()}
                        placeholder={t.manualModelPlaceholder}
                        onInput={(e) =>
                          setModelDraftByUuid((prev) => ({
                            ...prev,
                            [uuid]: e.currentTarget.value,
                          }))
                        }
                      />
                    }
                  >
                    <div class="pc__model-row">
                      <Select
                        label={t.models}
                        value={draftModel() || null}
                        options={selectOptions()}
                        loading={modelFetch() === "loading"}
                        loadingLabel={t.loadingModels}
                        onChange={(v) =>
                          setModelDraftByUuid((prev) => ({ ...prev, [uuid]: v }))
                        }
                      />
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => void handleFetchModels(uuid)}
                      >
                        {t.fetchModels}
                      </Button>
                    </div>
                  </Show>

                  <div class="pc__save-row">
                    <Button
                      variant="primary"
                      size="sm"
                      loading={saveState() === "saving"}
                      onClick={() => void handleSaveProfile(uuid)}
                    >
                      {t.saveProfile}
                    </Button>
                    <Show when={saveState() === "saved"}>
                      <span class="pc__saved-note">{t.profileSaved}</span>
                    </Show>
                  </div>

                  {/* Key section */}
                  <div class="pc__key-section">
                    <Show
                      when={p().hasKey}
                      fallback={
                        <>
                          <TextField
                            label={t.apiKey}
                            type="password"
                            value={keyText()}
                            placeholder={t.apiKeyPlaceholder}
                            errorText={keyError() ?? undefined}
                            onInput={(e) => {
                              setKeyInputByUuid((prev) => ({
                                ...prev,
                                [uuid]: e.currentTarget.value,
                              }));
                              // Clear any stale inline error as soon as the
                              // user edits the key again.
                              if (keyError()) {
                                setKeyErrorByUuid((prev) => {
                                  const n = { ...prev };
                                  delete n[uuid];
                                  return n;
                                });
                              }
                            }}
                          />
                          <Button
                            variant="primary"
                            size="sm"
                            disabled={p().needs_key && keyText().length === 0}
                            loading={saveState() === "saving"}
                            onClick={() => void handleSaveKey(uuid)}
                          >
                            {t.saveKey}
                          </Button>
                        </>
                      }
                    >
                      <span class="pc__key-saved-badge">
                        {t.keySaved}
                      </span>
                    </Show>
                  </div>

                  {/* Connection test */}
                  <div class="pc__conn-section">
                    <Button
                      variant="secondary"
                      size="sm"
                      loading={conn() === "testing"}
                      onClick={() => void handleTestConnection(uuid)}
                    >
                      {t.testConnection}
                    </Button>
                    <Show when={conn() && conn() !== "testing"}>
                      <StatusBadge
                        variant={
                          conn() && conn() !== "testing" && (conn() as ConnectionResult).ok
                            ? "success"
                            : "danger"
                        }
                      >
                        {conn() && conn() !== "testing" && (conn() as ConnectionResult).ok
                          ? t.connectionOk
                          : t.connectionFailed}
                      </StatusBadge>
                      <span class="pc__conn-message">
                        {(conn() as ConnectionResult).message}
                        <Show when={typeof (conn() as ConnectionResult).latency_ms === "number"}>
                          <span class="pc__conn-latency">
                            {" · "}{(conn() as ConnectionResult).latency_ms}ms
                          </span>
                        </Show>
                      </span>
                    </Show>
                  </div>

                  {/* Balance — R3a limitation: muted TODO note, no fetch. */}
                  <div class="pc__balance-section">
                    <span class="pc__balance-title">{t.balance.title}</span>
                    <span class="pc__balance-note">
                      {/* TODO(r3b): balance/quota IPC not yet implemented. */}
                      {t.balance.unsupportedNote}
                    </span>
                  </div>
                </div>
              );
            }}
          </Show>
        </section>
      </div>

      {/* Delete Confirm */}
      <Confirm
        open={!!deleteConfirmUuid()}
        onOpenChange={(o) => {
          if (!o) cancelDelete();
        }}
        title={t.deleteConfirmTitle}
        message={t.deleteConfirmMsg}
        confirmLabel={t.delete}
        cancelLabel={t.cancel}
        variant="destructive"
        onConfirm={() => void confirmDelete()}
        onCancel={() => cancelDelete()}
        triggerRef={deleteTriggerRef}
      />

      {/* Delete-error Retry banner: surfaced after a failed delete. Lives in the
          main layout (NOT inside the Kobalte Dialog) because the Dialog sets
          body pointer-events:none during its close transition, which would
          swallow clicks on an in-dialog Retry button. */}
      <Show when={deleteError()}>
        <div class="pc__delete-error-banner" role="alert">
          <span class="pc__delete-error-msg">{t.saveFailed}</span>
          <Button variant="secondary" size="sm" onClick={retryDelete}>
            {t.retry}
          </Button>
          <Button variant="ghost" size="sm" onClick={dismissDeleteError}>
            {t.cancel}
          </Button>
        </div>
      </Show>

      {/* Consent Confirm */}
      <Confirm
        open={consentOpen()}
        onOpenChange={(o) => !o && cancelConsent()}
        title={t.consent.title}
        message={t.consent.message}
        confirmLabel={t.consent.confirm}
        cancelLabel={t.consent.cancel}
        onConfirm={() => void confirmConsent()}
        onCancel={() => cancelConsent()}
        triggerRef={consentTriggerRef}
      >
        <ul class="pc__consent-recipients">
          <For each={consentRecipients()}>
            {(r) => (
              <li>
                <strong>{r.name}</strong> <span class="pc__consent-kind">{r.localLabel}</span>
              </li>
            )}
          </For>
        </ul>
      </Confirm>

      {/* Toasts */}
      <div class="pc__toasts" role="region" aria-label="notifications">
        <For each={toasts()}>
          {(toast) => (
            <Toast
              variant={toast.variant}
              message={toast.message}
              onDismiss={() => dismissToast(toast.id)}
              dismissLabel={t.toastDismiss}
            />
          )}
        </For>
      </div>
    </div>
  );
};

export default ProviderCenter;
