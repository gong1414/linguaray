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
import { Server, Plus, Copy, ArrowUp, ArrowDown, Check, X, Globe } from "lucide-solid";
import {
  ProviderCard,
  Button,
  TextField,
  Select,
  Confirm,
  Toast,
  Banner,
  EmptyState,
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
  { template: "ollama", name: "Ollama (local)" },
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

  // Detail panel state
  const [selectedUuid, setSelectedUuid] = createSignal<string | null>("mock-openai-1");
  const [keyInput, setKeyInput] = createSignal("");
  const [manualModel, setManualModel] = createSignal("");
  const [modelFetchStatus, setModelFetchStatus] = createSignal<"idle" | "loading" | "error">("idle");
  const [connStatus, setConnStatus] = createSignal<Record<string, "idle" | "testing" | "ok" | "failed">>({});
  const [connLatency, setConnLatency] = createSignal<Record<string, number>>({});
  const [saveStatus, setSaveStatus] = createSignal<"idle" | "saving" | "saved" | "failed">("idle");
  const [deleteConfirmUuid, setDeleteConfirmUuid] = createSignal<string | null>(null);
  const [toasts, setToasts] = createSignal<{ id: number; variant: "info" | "success" | "warning" | "destructive"; message: string }[]>([]);
  const [reorderAnnouncement, setReorderAnnouncement] = createSignal("");
  const [showPresetGrid, setShowPresetGrid] = createSignal(false);

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

  // Reset transient mock state on ProviderState change, with state-specific
  // fixtures so each state demonstrates its unique contract.
  createEffect(() => {
    const state = props.state;
    void state;
    generation += 1;
    clearAllTimers();
    setKeyInput("");
    setManualModel("");
    setModelFetchStatus("idle");
    setConnStatus({});
    setConnLatency({});
    setSaveStatus("idle");
    setDeleteConfirmUuid(null);
    setReorderAnnouncement("");
    setShowPresetGrid(false);

    // State-specific fixtures
    let provs = initialProviders();
    if (state === "empty") {
      provs = [];
    } else if (state === "duplicate") {
      // Pre-create a duplicate copy
      const orig = provs.find((p) => p.uuid === "mock-openai-1")!;
      provs = [...provs, { ...orig, uuid: "mock-openai-dup", name: "OpenAI #1 (copy)", hasKey: false, sortOrder: provs.length }];
    } else if (state === "deleting") {
      // Mark OpenAI #1 as deleting
      provs = provs.map((p) => (p.uuid === "mock-openai-1" ? { ...p, status: "deleting" as const, enabled: false } : p));
    } else if (state === "key-missing") {
      // Select a provider with no key for the detail panel
      setSelectedUuid("mock-openai-2");
    }

    setProviders(provs);

    // State-specific model/connection/save status
    if (state === "loading-models") setModelFetchStatus("loading");
    else if (state === "model-fetch-error") setModelFetchStatus("error");

    if (state === "saving") {
      setSaveStatus("saving");
      setSelectedUuid("mock-openai-2"); // no key → shows save button
    }

    if (state === "connection-ok") {
      const uuid = "mock-openai-1";
      setConnStatus({ [uuid]: "ok" });
      setConnLatency({ [uuid]: 42 });
    } else if (state === "connection-failed") {
      setConnStatus({ "mock-openai-1": "failed" });
    }

    if (state !== "key-missing" && state !== "saving") {
      setSelectedUuid(DEFAULT_SELECTION.primaryUuid);
    }
    setSelection({ ...DEFAULT_SELECTION });
    setConsentKey(consentScopeKey(buildConsentScope(DEFAULT_SELECTION, initialProviders())));
  });

  onCleanup(() => clearAllTimers());

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

  // --- toast helper ---
  let toastId = 0;
  const pushToast = (variant: "info" | "success" | "warning" | "destructive", message: string) => {
    const id = ++toastId;
    setToasts((prev) => [...prev, { id, variant, message }]);
  };
  const dismissToast = (id: number) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  };

  // --- ATOMIC role commit: build candidate → validate → commit ---
  const tryCommitSelection = (candidate: ActiveSelection): boolean => {
    const result = validateActiveSelection(candidate, providers());
    if (result.ok) {
      setSelection(candidate);
      // Invalidate consent if recipient set changed
      if (!isConsentValid(candidate, providers(), consentKey())) {
        setConsentKey(null);
      }
      return true;
    }
    // Validation failed — don't commit, show error toast
    pushToast("destructive", result.errors[0]!.message);
    return false;
  };

  const handleToggle = (uuid: string, enabled: boolean) => {
    setProviders((prev) => prev.map((p) => (p.uuid === uuid ? { ...p, enabled } : p)));
    if (!enabled) {
      // Clear all roles for this provider atomically
      const candidate: ActiveSelection = {
        primaryUuid: selection().primaryUuid === uuid ? null : selection().primaryUuid,
        parallelUuids: selection().parallelUuids.filter((u) => u !== uuid),
        fallbackUuid: selection().fallbackUuid === uuid ? null : selection().fallbackUuid,
      };
      setSelection(candidate);
    }
  };

  const handleSetPrimary = (uuid: string) => {
    const prev = selection();
    // Remove from parallel and fallback if present
    const candidate: ActiveSelection = {
      primaryUuid: uuid,
      parallelUuids: prev.parallelUuids.filter((u) => u !== uuid),
      fallbackUuid: prev.fallbackUuid === uuid ? null : prev.fallbackUuid,
    };
    tryCommitSelection(candidate);
  };

  const handleAddParallel = (uuid: string) => {
    // Consent required before adding to parallel
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
        if (tryCommitSelection(candidate)) {
          // Save consent scope AFTER successful commit
          setConsentKey(consentScopeKey(buildConsentScope(candidate, providers())));
        }
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
    if (tryCommitSelection(candidate)) {
      setConsentKey(consentScopeKey(buildConsentScope(candidate, providers())));
    }
  };

  const handleSetFallback = (uuid: string) => {
    const prev = selection();
    // Fallback must not overlap primary or parallel
    const candidate: ActiveSelection = {
      primaryUuid: prev.primaryUuid === uuid ? null : prev.primaryUuid,
      parallelUuids: prev.parallelUuids.filter((u) => u !== uuid),
      fallbackUuid: uuid,
    };
    tryCommitSelection(candidate);
  };

  // --- save key: capture UUID+generation at submission ---
  const handleSaveKey = () => {
    const targetUuid = selectedUuid();
    if (!targetUuid) return;
    const opGen = generation;
    // Clear input IMMEDIATELY (don't wait for success callback)
    setKeyInput("");
    setSaveStatus("saving");
    schedule(() => {
      // Validate this operation still applies to the same provider
      if (selectedUuid() !== targetUuid || opGen !== generation) return;
      setProviders((prev) => prev.map((p) => (p.uuid === targetUuid ? { ...p, hasKey: true } : p)));
      setSaveStatus("saved");
      pushToast("success", props.t.keySaved);
    }, 1000);
  };

  // --- connection test: capture UUID+generation ---
  const handleTestConnection = () => {
    const targetUuid = selectedUuid();
    if (!targetUuid) return;
    const opGen = generation;
    setConnStatus((prev) => ({ ...prev, [targetUuid]: "testing" }));
    schedule(() => {
      if (selectedUuid() !== targetUuid || opGen !== generation) return;
      // Simulate success or failure based on state
      const fail = props.state === "connection-failed";
      if (fail) {
        setConnStatus((prev) => ({ ...prev, [targetUuid]: "failed" }));
      } else {
        setConnLatency((prev) => ({ ...prev, [targetUuid]: 42 }));
        setConnStatus((prev) => ({ ...prev, [targetUuid]: "ok" }));
      }
    }, 1200);
  };

  // --- fetch models ---
  const handleFetchModels = () => {
    const targetUuid = selectedUuid();
    if (!targetUuid) return;
    const opGen = generation;
    setModelFetchStatus("loading");
    schedule(() => {
      if (selectedUuid() !== targetUuid || opGen !== generation) return;
      // Simulate fetch error for the error state
      if (props.state === "model-fetch-error") {
        setModelFetchStatus("error");
      } else {
        setModelFetchStatus("idle");
      }
    }, 1000);
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
    const opGen = generation;
    schedule(() => {
      if (opGen !== generation) return;
      setProviders((prev) => prev.filter((p) => p.uuid !== uuid));
    }, 1500);
  };

  const cancelDelete = () => {
    setDeleteConfirmUuid(null);
  };

  // --- add provider from preset ---
  const handleAddPreset = (preset: { template: string; name: string }) => {
    const newProvider: MockProvider = {
      uuid: mockUuid(),
      template: preset.template as MockProvider["template"],
      name: preset.name,
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

  // --- reorder (keyboard-first) ---
  const moveProvider = (uuid: string, dir: "up" | "down") => {
    const opGen = generation;
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
    // Simulate persist failure for the reorder-failed state
    if (props.state === "reorder-failed") {
      schedule(() => {
        if (opGen !== generation) return;
        setProviders(initialProviders());
        setReorderAnnouncement(props.t.reorderReverted);
        pushToast("destructive", props.t.reorderReverted);
      }, 800);
    }
  };

  // --- state-driven rendering ---
  const showEmpty = createMemo(() => props.state === "empty" && providers().length === 0);
  const sortedProviders = createMemo(() =>
    [...providers()].sort((a, b) => a.sortOrder - b.sortOrder),
  );
  const selectedProvider = createMemo(() =>
    providers().find((p) => p.uuid === selectedUuid()),
  );
  const isSaving = createMemo(() => saveStatus() === "saving" || props.state === "saving");
  const connForSelected = createMemo(() => {
    const uuid = selectedUuid();
    return uuid ? (connStatus()[uuid] ?? "idle") : "idle";
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
      <div class="pc__layout">
        {/* Sidebar: provider list */}
        <aside class="pc__sidebar" aria-label={props.t.providerListLabel}>
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
                    <span>{preset.name}</span>
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
                  <div class="pc__provider-row" data-status={p.status}>
                    <ProviderCard
                      profile={{ name: p.name, template: p.template, status: p.status }}
                      hasKey={p.hasKey}
                      role={roleFor(p.uuid)}
                      enabled={p.enabled}
                      onToggle={(en) => handleToggle(p.uuid, en)}
                      onEdit={() => setSelectedUuid(p.uuid)}
                      onDelete={() => handleDelete(p.uuid)}
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
                        disabled={index() === 0 || p.status === "deleting"}
                        onClick={() => moveProvider(p.uuid, "up")}
                      >
                        <ArrowUp size={14} aria-hidden="true" />
                      </button>
                      <button
                        type="button"
                        class="lr-icon-btn lr-focusable lr-icon-btn--ghost lr-icon-btn--sm"
                        aria-label={props.t.moveDown}
                        title={props.t.moveDown}
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
        <section class="pc__detail" aria-label={props.t.detailLabel}>
          <Show when={selectedProvider()} fallback={<p class="pc__select-primary">{props.t.selectPrimary}</p>}>
            {(p) => (
              <div class="pc__detail-content">
                <h3 class="pc__detail-title">{p().name}</h3>

                {/* Endpoint */}
                <TextField
                  label={props.t.endpoint}
                  value={p().endpoint}
                  disabled={isSaving()}
                  errorText={props.state === "endpoint-invalid" ? props.t.endpointInvalid : undefined}
                />

                {/* Model select / manual entry */}
                <FlowSwitch>
                  <Match when={modelFetchStatus() === "error" || props.state === "model-fetch-error" || props.state === "model-manual-entry"}>
                    <TextField
                      label={props.t.manualModelEntry}
                      value={manualModel()}
                      placeholder={props.t.manualModelPlaceholder}
                      disabled={isSaving()}
                      helperText={modelFetchStatus() === "error" || props.state === "model-fetch-error" ? props.t.modelFetchError : undefined}
                    />
                  </Match>
                  <Match when={true}>
                    <Select
                      label={props.t.models}
                      value={p().model}
                      options={MODEL_OPTIONS}
                      onChange={() => {}}
                      disabled={isSaving()}
                      loading={modelFetchStatus() === "loading" || props.state === "loading-models"}
                      errorText={modelFetchStatus() === "error" || props.state === "model-fetch-error" ? props.t.modelFetchError : undefined}
                    />
                    <Button variant="ghost" size="sm" onClick={handleFetchModels} disabled={isSaving()}>
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
                        value={keyInput()}
                        placeholder={props.t.apiKeyPlaceholder}
                        disabled={isSaving()}
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
                    disabled={isSaving()}
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

      {/* Save conflict banner (uses Banner component) */}
      <Show when={props.state === "save-conflict"}>
        <Banner
          variant="warning"
          title={props.t.saveConflict}
          action={
            <>
              <Button variant="secondary" size="sm" onClick={() => {}}>
                {props.t.reload}
              </Button>
              <Button variant="ghost" size="sm" onClick={() => {}}>
                {props.t.cancel}
              </Button>
            </>
          }
        />
      </Show>

      {/* Delete retry */}
      <Show when={props.state === "delete-retry"}>
        <div class="pc__delete-retry">
          <span>{props.t.deleteRetry}</span>
          <Button variant="destructive" size="sm" onClick={confirmDelete}>
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
              onDismiss={() => dismissToast(toast.id)}
            />
          )}
        </For>
        <Show when={props.state === "save-failed"}>
          <Toast variant="destructive" message={props.t.saveFailed} onDismiss={() => {}} />
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
        message={props.t.consentMsg}
        confirmLabel={props.t.consentConfirm}
        cancelLabel={props.t.consentCancel}
        variant="primary"
        onConfirm={confirmConsent}
        onCancel={cancelConsent}
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
