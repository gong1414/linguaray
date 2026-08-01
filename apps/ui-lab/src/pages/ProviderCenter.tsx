import {
  For,
  Show,
  Switch as FlowSwitch,
  Match,
  createSignal,
  createMemo,
  createEffect,
  onCleanup,
  type Component,
} from "solid-js";
import { Server, Plus, Copy, ArrowUp, ArrowDown, Check, X, AlertTriangle } from "lucide-solid";
import {
  ProviderCard,
  Button,
  TextField,
  Select,
  Confirm,
  Toast,
  EmptyState,
  type ProviderRole,
  type SelectOption,
} from "@linguaray/ui";
import type { Locale, LabStrings, ProviderState } from "../i18n";
import {
  type MockProvider,
  type ActiveSelection,
  validateActiveSelection,
  buildConsentScope,
  consentScopeKey,
  isConsentValid,
  TRADITIONAL_TEMPLATES,
} from "./provider-domain";
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
    { uuid: "mock-openai-2", template: "openai", name: "OpenAI #2 (copy)", endpoint: "https://api.openai.com/v1/chat/completions", model: "gpt-4o-mini", enabled: true, isLocal: false, hasKey: false, status: "active", sortOrder: 1 },
    { uuid: "mock-deepseek", template: "deepseek", name: "DeepSeek", endpoint: "https://api.deepseek.com/v1/chat/completions", model: "deepseek-chat", enabled: true, isLocal: false, hasKey: true, status: "active", sortOrder: 2 },
    { uuid: "mock-google", template: "google", name: "Google Translate", endpoint: "https://translation.googleapis.com/", model: null, enabled: true, isLocal: false, hasKey: true, status: "active", sortOrder: 3 },
    { uuid: "mock-ollama", template: "ollama", name: "Ollama (local)", endpoint: "http://localhost:11434/v1/chat/completions", model: "llama3", enabled: false, isLocal: true, hasKey: false, status: "active", sortOrder: 4 },
  ];
}

const MODEL_OPTIONS: SelectOption[] = [
  { value: "gpt-4o", label: "GPT-4o", disabled: false },
  { value: "gpt-4o-mini", label: "GPT-4o mini", disabled: false },
  { value: "gpt-4-turbo", label: "GPT-4 Turbo", disabled: false },
  { value: "manual", label: "Manual entry…", disabled: false },
];

// --- Component ------------------------------------------------------------

const ProviderCenter: Component<ProviderCenterProps> = (props) => {
  const [providers, setProviders] = createSignal<MockProvider[]>(initialProviders());
  const [selection, setSelection] = createSignal<ActiveSelection>({
    primaryUuid: "mock-openai-1",
    parallelUuids: ["mock-deepseek"],
    fallbackUuid: "mock-google",
  });
  const [consentKey, setConsentKey] = createSignal<string | null>(
    consentScopeKey(buildConsentScope(
      { primaryUuid: "mock-openai-1", parallelUuids: ["mock-deepseek"], fallbackUuid: "mock-google" },
      initialProviders(),
    )),
  );
  const [consentOpen, setConsentOpen] = createSignal(false);
  const [pendingParallel, setPendingParallel] = createSignal<string | null>(null);

  // Detail panel state
  const [selectedUuid, setSelectedUuid] = createSignal<string | null>("mock-openai-1");
  const [keyInput, setKeyInput] = createSignal("");
  const [manualModel, setManualModel] = createSignal("");
  const [connectionStatus, setConnectionStatus] = createSignal<"idle" | "testing" | "ok" | "failed">("idle");
  const [connectionLatency, setConnectionLatency] = createSignal<number | null>(null);
  const [, setBalanceStatus] = createSignal<"idle" | "loading" | "supported" | "unsupported" | "rate-limited" | "error">("idle");
  const [saveStatus, setSaveStatus] = createSignal<"idle" | "saving" | "saved" | "failed">("idle");
  const [deleteConfirmUuid, setDeleteConfirmUuid] = createSignal<string | null>(null);
  const [toasts, setToasts] = createSignal<{ id: number; variant: "info" | "success" | "warning" | "destructive"; message: string }[]>([]);
  const [reorderAnnouncement, setReorderAnnouncement] = createSignal("");

  // --- async-safety: generation token + tracked timers ---
  let generation = 0;
  const timers = new Set<number>();
  const schedule = (fn: () => void, ms: number): void => {
    const myGen = generation;
    const id = window.setTimeout(() => {
      timers.delete(id);
      if (myGen !== generation) return;
      fn();
    }, ms);
    timers.add(id);
  };
  const clearAllTimers = (): void => {
    for (const id of timers) window.clearTimeout(id);
    timers.clear();
  };

  // Reset transient mock state on ProviderState change
  createEffect(() => {
    void props.state;
    generation += 1;
    clearAllTimers();
    setKeyInput("");
    setManualModel("");
    setConnectionStatus("idle");
    setConnectionLatency(null);
    setBalanceStatus("idle");
    setSaveStatus("idle");
    setDeleteConfirmUuid(null);
    setReorderAnnouncement("");
    // Restore providers to baseline for deterministic state demos
    setProviders(initialProviders());
    setSelection({
      primaryUuid: "mock-openai-1",
      parallelUuids: ["mock-deepseek"],
      fallbackUuid: "mock-google",
    });
    setConsentKey(
      consentScopeKey(buildConsentScope(
        { primaryUuid: "mock-openai-1", parallelUuids: ["mock-deepseek"], fallbackUuid: "mock-google" },
        initialProviders(),
      )),
    );
  });

  onCleanup(() => clearAllTimers());

  // --- derived role for each provider ---
  const roleFor = (uuid: string): ProviderRole => {
    const sel = selection();
    if (sel.primaryUuid === uuid) return { kind: "primary" };
    const idx = sel.parallelUuids.indexOf(uuid);
    if (idx >= 0) return { kind: "parallel", index: idx + 1 };
    if (sel.fallbackUuid === uuid) return { kind: "fallback" };
    return { kind: "none" };
  };

  const validation = createMemo(() => validateActiveSelection(selection(), providers()));
  void validation; // available for debugging; not rendered in current states
  const consentValid = createMemo(() => isConsentValid(selection(), providers(), consentKey()));
  void consentValid; // consent gating is handled by the dialog flow

  // --- toast helper ---
  let toastId = 0;
  const pushToast = (variant: "info" | "success" | "warning" | "destructive", message: string) => {
    const id = ++toastId;
    setToasts((prev) => [...prev, { id, variant, message }]);
  };
  const dismissToast = (id: number) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  };

  // --- interactions ---
  const handleToggle = (uuid: string, enabled: boolean) => {
    setProviders((prev) => prev.map((p) => (p.uuid === uuid ? { ...p, enabled } : p)));
    // Disabling clears all roles synchronously
    if (!enabled) {
      setSelection((prev) => ({
        primaryUuid: prev.primaryUuid === uuid ? null : prev.primaryUuid,
        parallelUuids: prev.parallelUuids.filter((u) => u !== uuid),
        fallbackUuid: prev.fallbackUuid === uuid ? null : prev.fallbackUuid,
      }));
    }
  };

  const handleSetPrimary = (uuid: string) => {
    setSelection((prev) => ({ ...prev, primaryUuid: uuid, parallelUuids: prev.parallelUuids.filter((u) => u !== uuid) }));
  };

  const handleAddParallel = (uuid: string) => {
    // Consent required before adding to parallel
    setPendingParallel(uuid);
    setConsentOpen(true);
  };

  const confirmConsent = () => {
    const uuid = pendingParallel();
    if (uuid) {
      setSelection((prev) =>
        prev.parallelUuids.includes(uuid) || uuid === prev.primaryUuid
          ? prev
          : { ...prev, parallelUuids: [...prev.parallelUuids, uuid] },
      );
    }
    // Save consent scope AFTER modifying parallel
    setConsentKey(consentScopeKey(buildConsentScope(selection(), providers())));
    setConsentOpen(false);
    setPendingParallel(null);
  };

  const cancelConsent = () => {
    // Cancel: no change to selection or consent
    setConsentOpen(false);
    setPendingParallel(null);
  };

  const handleRemoveParallel = (uuid: string) => {
    setSelection((prev) => ({ ...prev, parallelUuids: prev.parallelUuids.filter((u) => u !== uuid) }));
    setConsentKey(consentScopeKey(buildConsentScope(selection(), providers())));
  };

  const handleSetFallback = (uuid: string) => {
    const p = providers().find((x) => x.uuid === uuid);
    if (p && TRADITIONAL_TEMPLATES.has(p.template)) {
      setSelection((prev) => ({ ...prev, fallbackUuid: uuid }));
    }
  };

  const handleSaveKey = () => {
    setSaveStatus("saving");
    schedule(() => {
      // Key is never stored in DOM — just flip the badge
      const uuid = selectedUuid();
      setProviders((prev) => prev.map((p) => (p.uuid === uuid ? { ...p, hasKey: true } : p)));
      setKeyInput(""); // clear immediately
      setSaveStatus("saved");
      pushToast("success", props.t.keySaved);
    }, 1000);
  };

  const handleTestConnection = () => {
    setConnectionStatus("testing");
    schedule(() => {
      setConnectionLatency(42);
      setConnectionStatus("ok");
    }, 1200);
  };

  const handleDuplicate = (uuid: string) => {
    const orig = providers().find((p) => p.uuid === uuid);
    if (!orig) return;
    const copy: MockProvider = {
      ...orig,
      uuid: mockUuid(),
      name: `${orig.name} (copy)`,
      hasKey: false, // duplicate does NOT copy key
      sortOrder: providers().length,
    };
    setProviders((prev) => [...prev, copy]);
  };

  const handleDelete = (uuid: string) => {
    setDeleteConfirmUuid(uuid);
  };

  const confirmDelete = () => {
    const uuid = deleteConfirmUuid();
    if (!uuid) return;
    // Deleting: immediately disable + clear roles
    handleToggle(uuid, false);
    setProviders((prev) => prev.map((p) => (p.uuid === uuid ? { ...p, status: "deleting" } : p)));
    setDeleteConfirmUuid(null);
    schedule(() => {
      setProviders((prev) => prev.filter((p) => p.uuid !== uuid));
    }, 1500);
  };

  const cancelDelete = () => {
    setDeleteConfirmUuid(null);
  };

  // --- reorder (keyboard-first) ---
  const moveProvider = (uuid: string, dir: "up" | "down") => {
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
    setReorderAnnouncement(`${name} moved ${dir}`);
    // Simulate persist failure for the reorder-failed state
    if (props.state === "reorder-failed") {
      schedule(() => {
        setProviders(initialProviders());
        setReorderAnnouncement(props.t.reorderReverted);
        pushToast("destructive", props.t.reorderReverted);
      }, 800);
    }
  };

  // --- state-driven rendering ---
  const showEmpty = createMemo(() => props.state === "empty");
  const sortedProviders = createMemo(() =>
    [...providers()].sort((a, b) => a.sortOrder - b.sortOrder),
  );

  const selectedProvider = createMemo(() =>
    providers().find((p) => p.uuid === selectedUuid()),
  );

  // Consent recipients for the dialog
  const consentRecipients = createMemo(() => {
    const sel = pendingParallel()
      ? { ...selection(), parallelUuids: [...selection().parallelUuids, pendingParallel()!] }
      : selection();
    return buildConsentScope(sel, providers()).recipients;
  });

  return (
    <div class="pc__body" role="region" aria-label={props.t.states[props.state]}>
      <div class="pc__layout">
        {/* Sidebar: provider list */}
        <aside class="pc__sidebar" aria-label="Provider list">
          <div class="pc__sidebar-header">
            <h2 class="pc__sidebar-title">{props.t.addProvider}</h2>
            <Button variant="primary" size="sm" leftIcon={<Plus size={14} />}>
              {props.t.addProvider}
            </Button>
          </div>

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
                  <div class="pc__provider-row" data-status={p.status}>
                    <ProviderCard
                      profile={{ name: p.name, template: p.template, status: p.status }}
                      hasKey={p.hasKey}
                      role={roleFor(p.uuid)}
                      enabled={p.enabled}
                      onToggle={(en) => handleToggle(p.uuid, en)}
                      onEdit={() => setSelectedUuid(p.uuid)}
                      onDelete={() => handleDelete(p.uuid)}
                    />
                    {/* Reorder controls */}
                    <div class="pc__reorder-controls">
                      <button
                        type="button"
                        class="lr-icon-btn lr-focusable lr-icon-btn--ghost lr-icon-btn--sm"
                        aria-label={props.t.moveUp}
                        disabled={index() === 0 || p.status === "deleting"}
                        onClick={() => moveProvider(p.uuid, "up")}
                      >
                        <ArrowUp size={14} aria-hidden="true" />
                      </button>
                      <button
                        type="button"
                        class="lr-icon-btn lr-focusable lr-icon-btn--ghost lr-icon-btn--sm"
                        aria-label={props.t.moveDown}
                        disabled={index() === sortedProviders().length - 1 || p.status === "deleting"}
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
                        <Button variant="ghost" size="sm" onClick={() => handleAddParallel(p.uuid)}>
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
        <section class="pc__detail" aria-label="Provider detail">
          <Show when={selectedProvider()} fallback={<p class="pc__select-primary">{props.t.selectPrimary}</p>}>
            {(p) => (
              <div class="pc__detail-content">
                <h3 class="pc__detail-title">{p().name}</h3>

                {/* Endpoint */}
                <TextField
                  label={props.t.endpoint}
                  value={p().endpoint}
                  errorText={props.state === "endpoint-invalid" ? props.t.endpointInvalid : undefined}
                />

                {/* Model select / manual entry */}
                <Show
                  when={props.state !== "model-manual-entry" && props.state !== "model-fetch-error"}
                  fallback={
                    <TextField
                      label={props.t.manualModelEntry}
                      value={manualModel()}
                      placeholder={props.t.manualModelPlaceholder}
                      helperText={props.state === "model-fetch-error" ? props.t.modelFetchError : undefined}
                    />
                  }
                >
                  <Select
                    label={props.t.models}
                    value={p().model}
                    options={MODEL_OPTIONS}
                    onChange={() => {}}
                    loading={props.state === "loading-models"}
                    errorText={props.state === "model-fetch-error" ? props.t.modelFetchError : undefined}
                  />
                </Show>

                {/* API key */}
                <Show
                  when={p().hasKey}
                  fallback={
                    <div class="pc__key-section">
                      <TextField
                        label={props.t.apiKey}
                        type="password"
                        value={keyInput()}
                        placeholder={props.t.apiKeyPlaceholder}
                        onChange={(e) => setKeyInput(e.currentTarget.value)}
                      />
                      <Button
                        variant="primary"
                        size="md"
                        loading={saveStatus() === "saving" || props.state === "saving"}
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
                    loading={connectionStatus() === "testing" || props.state === "connection-testing"}
                    loadingLabel={props.t.testing}
                    onClick={handleTestConnection}
                  >
                    {props.t.testConnection}
                  </Button>
                  <Show when={connectionStatus() === "ok" || props.state === "connection-ok"}>
                    <span class="pc__conn-ok">
                      <Check size={14} aria-hidden="true" />
                      {props.t.connectionOk} · {connectionLatency() ?? 42}ms
                    </span>
                  </Show>
                  <Show when={connectionStatus() === "failed" || props.state === "connection-failed"}>
                    <span class="pc__conn-fail">
                      <X size={14} aria-hidden="true" />
                      {props.t.connectionFailed}
                    </span>
                  </Show>
                </div>

                {/* Balance */}
                <div class="pc__balance-section">
                  <FlowSwitch>
                    <Match when={props.state === "balance-loading"}>
                      <span class="pc__balance">{props.t.balanceLoading}</span>
                    </Match>
                    <Match when={props.state === "balance-unsupported"}>
                      <span class="pc__balance">{props.t.balanceUnsupported}</span>
                    </Match>
                    <Match when={props.state === "balance-rate-limited"}>
                      <span class="pc__balance">{props.t.balanceRateLimited}</span>
                    </Match>
                    <Match when={props.state === "balance-error"}>
                      <span class="pc__balance">{props.t.balanceError}</span>
                    </Match>
                  </FlowSwitch>
                </div>
              </div>
            )}
          </Show>
        </section>
      </div>

      {/* Save conflict banner */}
      <Show when={props.state === "save-conflict"}>
        <div class="pc__conflict-banner">
          <AlertTriangle size={16} aria-hidden="true" />
          <span>{props.t.saveConflict}</span>
          <Button variant="secondary" size="sm">{props.t.reload}</Button>
          <Button variant="ghost" size="sm">{props.t.cancel}</Button>
        </div>
      </Show>

      {/* Delete retry */}
      <Show when={props.state === "delete-retry"}>
        <div class="pc__delete-retry">
          <span>{props.t.deleteRetry}</span>
          <Button variant="destructive" size="sm">{props.t.delete}</Button>
        </div>
      </Show>

      {/* Toasts */}
      <div class="pc__toasts">
        <For each={toasts()}>
          {(toast) => (
            <Toast
              variant={toast.variant}
              message={toast.message}
              onDismiss={() => dismissToast(toast.id)}
            />
          )}
        </For>
        <Show when={props.state === "save-failed"}>
          <Toast variant="destructive" message={props.t.saveFailed} onDismiss={() => {}} />
        </Show>
        <Show when={props.state === "reorder-failed" && reorderAnnouncement()}>
          <Toast variant="destructive" message={props.t.reorderReverted} onDismiss={() => {}} />
        </Show>
      </div>

      {/* Delete confirm dialog */}
      <Confirm
        open={deleteConfirmUuid() !== null || props.state === "delete-confirm"}
        onOpenChange={(open) => { if (!open) cancelDelete(); }}
        title={props.t.deleteConfirmTitle}
        message={props.t.deleteConfirmMsg}
        confirmLabel={props.t.delete}
        cancelLabel={props.t.cancel}
        variant="destructive"
        onConfirm={confirmDelete}
        onCancel={cancelDelete}
      />

      {/* Consent dialog */}
      <Confirm
        open={consentOpen()}
        onOpenChange={(open) => { if (!open) cancelConsent(); }}
        title={props.t.consentTitle}
        message={`${props.t.consentMsg} ${consentRecipients().map((r) => r.providerUuid).join(", ")}`}
        confirmLabel={props.t.consentConfirm}
        cancelLabel={props.t.consentCancel}
        variant="primary"
        onConfirm={confirmConsent}
        onCancel={cancelConsent}
      />
    </div>
  );
};

export default ProviderCenter;
