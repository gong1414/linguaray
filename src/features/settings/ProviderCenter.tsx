/**
 * Provider Center (Surface 05) — production component on real IPC.
 *
 * rev-7-3: the presentational body is extracted into `ProviderCenterView`
 * (a pure props-driven View, shared with the ui-lab visual fixture). The
 * default export is the controller: it owns the signals + IPC and renders
 * `<ProviderCenterView ... />` with signal-derived props.
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
  type JSX,
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
import { SETTINGS_COPY, type SettingsCopy } from "./copy";
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
export type Preset = { templateId: string; name: string | null; endpoint: string; model: string | null };

/** Escape a literal string for use inside a CSS attribute selector. Provider
 *  names may contain characters that break `button[aria-label="..."]` otherwise. */
const cssEscape = (s: string): string =>
  s.replace(/["\\]/g, (c) => `\\${c}`);

export const PRESETS: Preset[] = [
  { templateId: "openai", name: "OpenAI", endpoint: "https://api.openai.com/v1/chat/completions", model: "gpt-4o-mini" },
  { templateId: "anthropic", name: "Anthropic", endpoint: "https://api.anthropic.com/v1/messages", model: "claude-sonnet-4-5" },
  { templateId: "gemini", name: "Gemini", endpoint: "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions", model: "gemini-3.6-flash" },
  { templateId: "ollama", name: null, endpoint: "http://localhost:11434/v1/chat/completions", model: "qwen2.5:7b" },
];

export type ToastVariant = "info" | "success" | "warning" | "destructive";
export type ToastEntry = { id: number; variant: ToastVariant; message: string };
export type ConsentRecipient = { name: string; localLabel: string };

/**
 * The detail-panel state for the currently-selected provider. The controller
 * gathers every per-UUID draft/error/status for the selected provider into this
 * object so the View is pure (no signal reads inside the panel).
 */
export type ProviderDetailState = {
  provider: ProviderProfileFE;
  nameDraft: string;
  endpointDraft: string;
  modelDraft: string;
  keyText: string;
  nameError?: string;
  keyError?: string;
  endpointError?: string;
  saveState: "idle" | "saving" | "saved" | "failed";
  conn: ConnectionResult | "testing" | "idle";
  modelOptions: ModelInfo[];
  modelFetch: "idle" | "loading" | "error";
  saveConflict: boolean;
};

/**
 * Pure presentational View for Surface 05 (Provider Center). Shared by the
 * production controller (default export below) + the ui-lab visual fixture
 * (apps/ui-lab/src/pages/ProviderCenter.tsx). No signals, no IPC — all data
 * and mutations flow through props.
 */
export type ProviderCenterViewProps = {
  t: SettingsCopy["provider"];
  providers: ProviderProfileFE[];
  selection: ActiveSelection;
  selectedUuid: string | null;
  loadError: boolean;
  selectionError: boolean;
  selectionLoading: boolean;
  deletingUuid: string | null;
  // R5-P1-1: provider whose save-conflict Reload is in-flight. While set, the
  // detail-panel fields for that UUID are disabled (prevents the user from
  // typing drafts that the in-flight refresh would unconditionally discard).
  reloadingUuid: string | null;
  // R6-P1-1: global mutation lock — true while ANY refresh() is in-flight
  // (initial load, Reload, post-create/delete/duplicate re-fetch). While true,
  // ALL sidebar action buttons for ALL providers are disabled (prevents a
  // concurrent mutation from being overwritten by the refresh's setProviders).
  globalMutationLock: boolean;
  presets: Preset[];
  detail: ProviderDetailState | null;
  // dialogs + toasts
  deleteConfirmUuid: string | null;
  deleteError: boolean;
  deleteFailedUuid: string | null;
  consentOpen: boolean;
  consentRecipients: ConsentRecipient[];
  toasts: ToastEntry[];
  // refs forwarded to Confirm (focus restore)
  deleteTriggerRef: { current?: HTMLElement };
  consentTriggerRef: { current?: HTMLElement };
  // --- sidebar row callbacks ---
  onToggle: (uuid: string, enabled: boolean) => void;
  onEdit: (uuid: string) => void;
  onDelete: (uuid: string) => void;
  onSetPrimary: (uuid: string) => void;
  onAddParallel: (uuid: string, triggerEl?: HTMLElement) => void;
  onRemoveParallel: (uuid: string) => void;
  onSetFallback: (uuid: string) => void;
  onDuplicate: (uuid: string) => void;
  onMoveUp: (uuid: string) => void;
  onMoveDown: (uuid: string) => void;
  onAddPreset: (preset: Preset) => void;
  // --- detail-panel callbacks ---
  onNameInput: (uuid: string, value: string) => void;
  onEndpointInput: (uuid: string, value: string) => void;
  onModelInput: (uuid: string, value: string) => void;
  onModelChange: (uuid: string, value: string) => void;
  onKeyInput: (uuid: string, value: string) => void;
  onSaveProfile: (uuid: string) => void;
  onSaveKey: (uuid: string) => void;
  onFetchModels: (uuid: string) => void;
  onTestConnection: (uuid: string) => void;
  onResolveSaveConflict: (uuid: string) => void;
  // --- top-level error + dialog callbacks ---
  onReloadFromError: () => void;
  onRetrySelectionLoad: () => void;
  onConfirmDelete: () => void;
  onCancelDelete: () => void;
  onRetryDelete: () => void;
  onDismissDeleteError: () => void;
  onConfirmConsent: () => void;
  onCancelConsent: () => void;
  onDismissToast: (id: number) => void;
};

export function ProviderCenterView(props: ProviderCenterViewProps): JSX.Element {
  const t = () => props.t;

  // R6-P1-1: while the global mutation lock is held (a refresh is in-flight),
  // every sidebar action for EVERY provider is disabled. Composed with the
  // per-row deleting state below.
  const locked = () => props.globalMutationLock;

  const sortedProviders = createMemo(() =>
    [...props.providers].sort((a, b) => a.sort_order - b.sort_order),
  );

  // --- Derived role for a provider (session mirror) ---
  const roleFor = (uuid: string): ProviderRole => {
    const sel = props.selection;
    if (sel.primaryUuid === uuid) return { kind: "primary" };
    const idx = sel.parallelUuids.indexOf(uuid);
    if (idx >= 0) return { kind: "parallel", index: idx + 1 };
    if (sel.fallbackUuid === uuid) return { kind: "fallback" };
    return { kind: "none" };
  };

  // --- Row labels for ProviderRow (per-provider: {name} interpolated) ---
  const rowLabelsFor = (name: string): ProviderRowLabels => ({
    edit: t().cardEdit.replace("{name}", name),
    delete: t().cardDelete.replace("{name}", name),
    enabled: t().enabled,
    statusText: {
      active: t().role.primary,
      available: t().role.none,
      "key-missing": t().keyMissing,
      disabled: t().disabled,
    },
  });

  return (
    <div class="pc__body" role="region" aria-label={t().providerListLabel}>
      <Show when={props.loadError}>
        <InlineError>{t().saveFailed}</InlineError>
        <div class="pc__retry">
          <Button variant="primary" size="sm" onClick={() => props.onReloadFromError()}>
            {t().reload}
          </Button>
        </div>
      </Show>

      {/* Cold-load failure: selection read failed → fail-closed. Role mutations
          are disabled (see handler guards) until a successful retry. */}
      <Show when={props.selectionError}>
        <div class="pc__retry" role="alert">
          <span class="pc__load-failed">{t().loadFailed}</span>
          <Button variant="secondary" size="sm" onClick={() => props.onRetrySelectionLoad()}>
            {t().retry}
          </Button>
        </div>
      </Show>

      <div class="pc__layout">
        {/* Sidebar: provider list */}
        <aside class="pc__sidebar" aria-label={t().providerListLabel}>
          <div class="pc__sidebar-header">
            <h2 class="pc__sidebar-title">{t().addProvider}</h2>
          </div>

          <Show
            when={props.providers.length > 0}
            fallback={
              <EmptyState
                icon={<Server size={32} />}
                title={t().empty.title}
                description={t().empty.description}
              />
            }
          >
            <ul class="pc__provider-list" role="list">
              <For each={sortedProviders()}>
                {(p) => {
                  // role is reactive: re-evaluated when props.selection changes.
                  const role = () => roleFor(p.uuid);
                  const isDeleting = () => props.deletingUuid === p.uuid;
                  // R6-P1-1: disable the row + every sibling icon button while the
                  // global mutation lock is held OR this row is being deleted.
                  const rowDisabled = () => isDeleting() || locked();
                  return (
                    <li class="pc__provider-row-wrapper" data-status={isDeleting() ? "deleting" : p.status}>
                      <div class="pc__provider-row-main">
                        <ProviderRow
                          name={p.name}
                          template={p.template_id}
                          hasKey={p.hasKey}
                          role={role()}
                          enabled={p.enabled}
                          active={props.selectedUuid === p.uuid}
                          disabled={rowDisabled()}
                          labels={rowLabelsFor(p.name)}
                          onToggle={(enabled) => props.onToggle(p.uuid, enabled)}
                          onEdit={() => props.onEdit(p.uuid)}
                          onDelete={() => props.onDelete(p.uuid)}
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
                                aria-label={t().setPrimary}
                                title={t().setPrimary}
                                disabled={rowDisabled()}
                                onClick={() => props.onSetPrimary(p.uuid)}
                              >
                                <Star size={14} />
                              </button>
                            </Show>
                            <Show when={role().kind === "parallel"}>
                              <button
                                type="button"
                                class="pc__icon-btn"
                                aria-label={t().removeParallel}
                                title={t().removeParallel}
                                disabled={rowDisabled()}
                                onClick={() => props.onRemoveParallel(p.uuid)}
                              >
                                <Layers size={14} />
                              </button>
                            </Show>
                            <Show when={role().kind !== "parallel" && role().kind !== "primary"}>
                              <button
                                type="button"
                                class="pc__icon-btn"
                                aria-label={t().addParallel}
                                title={t().addParallel}
                                disabled={rowDisabled()}
                                onClick={(e) =>
                                  props.onAddParallel(
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
                                aria-label={t().setFallback}
                                title={t().setFallback}
                                disabled={rowDisabled()}
                                onClick={() => props.onSetFallback(p.uuid)}
                              >
                                <CornerDownLeft size={14} />
                              </button>
                            </Show>
                          </Show>
                          {/* Duplicate */}
                          <button
                            type="button"
                            class="pc__icon-btn"
                            aria-label={t().duplicate}
                            title={t().duplicate}
                            disabled={rowDisabled()}
                            onClick={() => props.onDuplicate(p.uuid)}
                          >
                            <Copy size={14} />
                          </button>
                          {/* Reorder */}
                          <button
                            type="button"
                            class="pc__icon-btn"
                            aria-label={t().moveUp}
                            title={t().moveUp}
                            disabled={rowDisabled()}
                            onClick={() => props.onMoveUp(p.uuid)}
                          >
                            <ArrowUp size={14} />
                          </button>
                          <button
                            type="button"
                            class="pc__icon-btn"
                            aria-label={t().moveDown}
                            title={t().moveDown}
                            disabled={rowDisabled()}
                            onClick={() => props.onMoveDown(p.uuid)}
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
                            <StatusBadge variant="success">{t().role.primary}</StatusBadge>
                          </Show>
                          <Show when={role().kind === "parallel"}>
                            <StatusBadge variant="info">
                              {t().role.parallel} {(role() as { kind: "parallel"; index: number }).index}
                            </StatusBadge>
                          </Show>
                          <Show when={role().kind === "fallback"}>
                            <StatusBadge variant="neutral">{t().role.fallback}</StatusBadge>
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
            <For each={props.presets}>
              {(preset) => (
                <button
                  type="button"
                  class="pc__preset"
                  disabled={locked()}
                  onClick={() => props.onAddPreset(preset)}
                >
                  <Plus size={12} />
                  <span>{preset.name ?? "Ollama"}</span>
                </button>
              )}
            </For>
          </div>
        </aside>

        {/* Detail panel */}
        <section class="pc__detail" aria-label={t().detailLabel}>
          <Show
            when={props.detail}
            fallback={
              <EmptyState icon={<Server size={32} />} title={t().selectPrimary} />
            }
          >
            {(d) => {
              // R5-P1-1: reactive uuid accessor — d() is reactive so this always
              // reflects the CURRENTLY-selected provider, even after the user
              // navigates from one provider to another without the detail panel
              // unmounting. A captured const would go stale and route Save/Reload
              // to the wrong provider.
              const uuid = () => d().provider.uuid;
              // R5-P1-1: while THIS provider's save-conflict Reload is in-flight,
              // every editable field + Save/Reload is disabled (a draft typed
              // during the await refresh() would be unconditionally discarded on
              // success). Other providers' fields are unaffected.
              const isReloading = () => props.reloadingUuid === uuid();
              const canListModels = () => d().provider.capabilities.model_list;
              const selectOptions = createMemo<SelectOption[]>(() => {
                const opts = d().modelOptions;
                if (opts && opts.length > 0) {
                  return opts.map((m) => ({ value: m.id, label: m.label, disabled: false }));
                }
                // Manual entry fallback: current model as the only option.
                return [{ value: d().modelDraft || "—", label: d().modelDraft || "—", disabled: false }];
              });
              return (
                <div class="pc__detail-content">
                  {/* R2-E: save-conflict banner. Surfaces when an optimistic-lock
                      CAS rejected this save (`stale_version`). The user's draft
                      is preserved (not overwritten); Reload re-fetches fresh data
                      so they can reconcile against the other writer's version. */}
                  <Show when={d().saveConflict}>
                    <div class="pc__retry" role="alert">
                      <span class="pc__load-failed">{t().saveConflict}</span>
                      <Button
                        variant="primary"
                        size="sm"
                        loading={isReloading()}
                        disabled={isReloading()}
                        loadingLabel={t().reloading}
                        onClick={() => props.onResolveSaveConflict(uuid())}
                      >
                        {t().reload}
                      </Button>
                    </div>
                  </Show>
                  {/* Name (editable — backend supports `name` patches). */}
                  <TextField
                    label={t().name}
                    value={d().nameDraft}
                    errorText={d().nameError ?? undefined}
                    disabled={d().saveState === "saving" || isReloading()}
                    onInput={(e) => {
                      props.onNameInput(uuid(), e.currentTarget.value);
                    }}
                  />

                  {/* Endpoint */}
                  <TextField
                    label={t().endpoint.label}
                    value={d().endpointDraft}
                    placeholder={t().endpoint.placeholder}
                    errorText={d().endpointError ?? undefined}
                    disabled={d().saveState === "saving" || isReloading()}
                    onInput={(e) =>
                      props.onEndpointInput(uuid(), e.currentTarget.value)
                    }
                  />

                  {/* Model: dropdown + Fetch models only when the provider
                      advertises model_list; otherwise a manual-entry input.
                      A fetch error also falls back to the manual input. */}
                  <Show
                    when={d().modelFetch !== "error" && canListModels()}
                    fallback={
                      <TextField
                        label={t().models}
                        value={d().modelDraft}
                        placeholder={t().manualModelPlaceholder}
                        disabled={d().saveState === "saving" || isReloading()}
                        onInput={(e) =>
                          props.onModelInput(uuid(), e.currentTarget.value)
                        }
                      />
                    }
                  >
                    <div class="pc__model-row">
                      <Select
                        label={t().models}
                        value={d().modelDraft || null}
                        options={selectOptions()}
                        loading={d().modelFetch === "loading"}
                        loadingLabel={t().loadingModels}
                        disabled={d().saveState === "saving" || isReloading()}
                        onChange={(v) =>
                          props.onModelChange(uuid(), v)
                        }
                      />
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => props.onFetchModels(uuid())}
                      >
                        {t().fetchModels}
                      </Button>
                    </div>
                  </Show>

                  <div class="pc__save-row">
                    <Button
                      variant="primary"
                      size="sm"
                      loading={d().saveState === "saving"}
                      disabled={isReloading()}
                      onClick={() => props.onSaveProfile(uuid())}
                    >
                      {t().saveProfile}
                    </Button>
                    <Show when={d().saveState === "saved"}>
                      <span class="pc__saved-note">{t().profileSaved}</span>
                    </Show>
                  </div>

                  {/* Key section */}
                  <div class="pc__key-section">
                    <Show
                      when={d().provider.hasKey}
                      fallback={
                        <>
                          <TextField
                            label={t().apiKey}
                            type="password"
                            value={d().keyText}
                            placeholder={t().apiKeyPlaceholder}
                            errorText={d().keyError ?? undefined}
                            disabled={d().saveState === "saving"}
                            onInput={(e) => {
                              props.onKeyInput(uuid(), e.currentTarget.value);
                            }}
                          />
                          <Button
                            variant="primary"
                            size="sm"
                            disabled={d().provider.needs_key && d().keyText.length === 0}
                            loading={d().saveState === "saving"}
                            onClick={() => props.onSaveKey(uuid())}
                          >
                            {t().saveKey}
                          </Button>
                        </>
                      }
                    >
                      <span class="pc__key-saved-badge">
                        {t().keySaved}
                      </span>
                    </Show>
                  </div>

                  {/* Connection test */}
                  <div class="pc__conn-section">
                    <Button
                      variant="secondary"
                      size="sm"
                      loading={d().conn === "testing"}
                      onClick={() => props.onTestConnection(uuid())}
                    >
                      {t().testConnection}
                    </Button>
                    <Show when={d().conn && d().conn !== "testing" && d().conn !== "idle"}>
                      <StatusBadge
                        variant={
                          d().conn !== "testing" && d().conn !== "idle" && (d().conn as ConnectionResult).ok
                            ? "success"
                            : "danger"
                        }
                      >
                        {d().conn !== "testing" && d().conn !== "idle" && (d().conn as ConnectionResult).ok
                          ? t().connectionOk
                          : t().connectionFailed}
                      </StatusBadge>
                      <span class="pc__conn-message">
                        {(d().conn as ConnectionResult).message}
                        <Show when={typeof (d().conn as ConnectionResult).latency_ms === "number"}>
                          <span class="pc__conn-latency">
                            {" · "}{(d().conn as ConnectionResult).latency_ms}ms
                          </span>
                        </Show>
                      </span>
                    </Show>
                  </div>

                  {/* Balance — R3a limitation: muted TODO note, no fetch. */}
                  <div class="pc__balance-section">
                    <span class="pc__balance-title">{t().balance.title}</span>
                    <span class="pc__balance-note">
                      {/* TODO(r3b): balance/quota IPC not yet implemented. */}
                      {t().balance.unsupportedNote}
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
        open={!!props.deleteConfirmUuid}
        onOpenChange={(o) => {
          if (!o) props.onCancelDelete();
        }}
        title={t().deleteConfirmTitle}
        message={t().deleteConfirmMsg}
        confirmLabel={t().delete}
        cancelLabel={t().cancel}
        variant="destructive"
        onConfirm={() => props.onConfirmDelete()}
        onCancel={() => props.onCancelDelete()}
        triggerRef={props.deleteTriggerRef}
      />

      {/* Delete-error Retry banner: surfaced after a failed delete. Lives in the
          main layout (NOT inside the Kobalte Dialog) because the Dialog sets
          body pointer-events:none during its close transition, which would
          swallow clicks on an in-dialog Retry button. */}
      <Show when={props.deleteError}>
        <div class="pc__delete-error-banner" role="alert">
          <span class="pc__delete-error-msg">{t().saveFailed}</span>
          <Button variant="secondary" size="sm" onClick={() => props.onRetryDelete()}>
            {t().retry}
          </Button>
          <Button variant="ghost" size="sm" onClick={() => props.onDismissDeleteError()}>
            {t().cancel}
          </Button>
        </div>
      </Show>

      {/* Consent Confirm */}
      <Confirm
        open={props.consentOpen}
        onOpenChange={(o) => !o && props.onCancelConsent()}
        title={t().consent.title}
        message={t().consent.message}
        confirmLabel={t().consent.confirm}
        cancelLabel={t().consent.cancel}
        onConfirm={() => props.onConfirmConsent()}
        onCancel={() => props.onCancelConsent()}
        triggerRef={props.consentTriggerRef}
      >
        <ul class="pc__consent-recipients">
          <For each={props.consentRecipients}>
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
        <For each={props.toasts}>
          {(toast) => (
            <Toast
              variant={toast.variant}
              message={toast.message}
              onDismiss={() => props.onDismissToast(toast.id)}
              dismissLabel={t().toastDismiss}
            />
          )}
        </For>
      </div>
    </div>
  );
}

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
  // R6-P1-3: per-UUID request counter for connection tests. Each Test click
  // bumps the counter; the await resolution only applies if the counter is
  // unchanged (no newer Test started during the await). This prevents a stale
  // completion from overwriting a newer result (connection-test ABA).
  const [connRequestIdByUuid, setConnRequestIdByUuid] = createSignal<Record<string, number>>({});
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
  // Per-provider deleting busy state. Set while `provider_delete` is in-flight
  // so the row's action buttons are locked (prevents double-delete / races).
  const [deletingUuid, setDeletingUuid] = createSignal<string | null>(null);
  // R2-E: save-conflict signal. Set when an optimistic-lock CAS rejects a save
  // (`stale_version` — the provider was modified elsewhere). The banner offers a
  // Reload that re-fetches fresh data; the user's in-progress draft is PRESERVED
  // so they can reconcile manually instead of losing their edits.
  const [saveConflictUuid, setSaveConflictUuid] = createSignal<string | null>(null);
  // R5-P1-1: provider whose save-conflict Reload is in-flight. While set, the
  // detail-panel fields for that UUID are disabled (prevents the user from
  // typing drafts that the in-flight refresh would unconditionally discard on
  // success) and a second Reload click is blocked (re-entrancy guard).
  const [reloadingUuid, setReloadingUuid] = createSignal<string | null>(null);
  // R6-P1-1: global mutation lock. While ANY refresh() is in-flight (initial
  // load, Reload, post-create/delete/duplicate re-fetch), ALL provider mutations
  // are blocked. This prevents the race where a concurrent mutation's
  // setProviders(...) is overwritten when the refresh's setProviders(list)
  // resolves. `reloadingUuid` (above) only disables ONE provider's detail panel;
  // this lock disables every sidebar action for EVERY provider.
  const [globalMutationLock, setGlobalMutationLock] = createSignal(false);
  const [consentOpen, setConsentOpen] = createSignal(false);
  const [pendingParallelUuid, setPendingParallelUuid] = createSignal<string | null>(null);
  const [consentActualScope, setConsentActualScope] = createSignal<string | null>(null);
  const consentTriggerRef: { current?: HTMLElement } = {};
  const [toasts, setToasts] = createSignal<ToastEntry[]>([]);

  let toastId = 0;
  const pushToast = (
    variant: ToastVariant,
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
  /** Re-fetch the provider list + stored selection. Returns `true` on success,
   *  `false` on failure (the error is already surfaced via loadError + a toast;
   *  callers that need to react to failure — e.g. the save-conflict Reload —
   *  branch on the boolean instead of try/catch, since refresh never rejects). */
  const refresh = async (): Promise<boolean> => {
    // R6-P1-1: acquire the global mutation lock for the full duration of the
    // await. While held, every sidebar mutation handler early-returns so a
    // concurrent setProviders(...) from a save/toggle/delete/reorder cannot be
    // overwritten by this refresh's setProviders(list) when it resolves.
    setGlobalMutationLock(true);
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
      return true;
    } catch (e) {
      setLoadError(true);
      setSelectionError(true);
      pushToast("destructive", t.saveFailed);
      return false;
    } finally {
      setSelectionLoading(false);
      setGlobalMutationLock(false);
    }
  };

  onMount(() => {
    void refresh();
  });

  const selectedProvider = createMemo(() =>
    providers().find((p) => p.uuid === selectedUuid()),
  );

  // --- Mutations ---

  /** Toggle: optimistic flip → IPC → revert + toast on error. */
  const handleToggle = async (uuid: string, enabled: boolean) => {
    if (globalMutationLock()) return; // R6-P1-1
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
    if (globalMutationLock()) return; // R6-P1-1
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
    if (globalMutationLock()) return; // R6-P1-1
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
    if (globalMutationLock()) return; // R6-P1-1
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
    if (globalMutationLock()) return; // R6-P1-1
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
    if (globalMutationLock()) return; // R6-P1-1
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
    if (globalMutationLock()) return; // R6-P1-1
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
    if (globalMutationLock()) return; // R6-P1-1
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
    if (globalMutationLock()) return; // R6-P1-1
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
        // R2-E optimistic lock: echo back the last-read version. A mismatch
        // (someone else saved first) rejects with `stale_version` below.
        expected_version: provider.version,
      });
      setProviders((prev) => prev.map((p) => (p.uuid === uuid ? { ...p, ...updated, hasKey: p.hasKey } : p)));
      setSaveByUuid((prev) => ({ ...prev, [uuid]: "saved" }));
      // A successful save clears any prior conflict banner for this provider.
      setSaveConflictUuid((prev) => (prev === uuid ? null : prev));
      pushToast("success", t.profileSaved);
    } catch (e) {
      // R2-E: a structured stale_version rejection = save conflict. Keep the
      // user's draft intact (do NOT overwrite) and surface a conflict banner
      // with a Reload button so they can pull fresh data and reconcile.
      const err = e as { error?: string };
      if (err?.error === "stale_version") {
        setSaveByUuid((prev) => ({ ...prev, [uuid]: "failed" }));
        setSaveConflictUuid(uuid);
      } else {
        setSaveByUuid((prev) => ({ ...prev, [uuid]: "failed" }));
        pushToast("destructive", t.saveFailed);
      }
    }
  };

  /**
   * Save key: clear the input IMMEDIATELY (never re-readable), then IPC.
   * On success re-fetch key_status to update `hasKey`. The key is NEVER
   * re-read from the input after submit — the input stays cleared.
   */
  const handleSaveKey = async (uuid: string) => {
    if (globalMutationLock()) return; // R6-P1-1
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
      // Surface the failure so the user knows why no dropdown appeared.
      pushToast("warning", t.modelFetchError);
    }
  };

  const handleTestConnection = async (uuid: string) => {
    // R6-P1-3: bump the request counter so a stale completion (from an earlier
    // Test click whose await resolved after a newer Test) is discarded.
    const requestId = (connRequestIdByUuid()[uuid] ?? 0) + 1;
    setConnRequestIdByUuid((prev) => ({ ...prev, [uuid]: requestId }));
    setConnByUuid((prev) => ({ ...prev, [uuid]: "testing" }));
    try {
      const result = await providerTestConnection(uuid);
      // Only apply if this is still the latest request for this UUID.
      if (connRequestIdByUuid()[uuid] !== requestId) return;
      setConnByUuid((prev) => ({ ...prev, [uuid]: result }));
    } catch (e) {
      if (connRequestIdByUuid()[uuid] !== requestId) return;
      setConnByUuid((prev) => ({
        ...prev,
        [uuid]: { ok: false, message: t.connectionFailed },
      }));
    }
  };

  const confirmDelete = async () => {
    const uuid = deleteConfirmUuid() ?? deleteFailedUuid();
    if (!uuid) return;
    if (globalMutationLock()) return; // R6-P1-1
    setDeletingUuid(uuid);
    try {
      await providerDelete(uuid);
      setDeleteError(false);
      setDeleteFailedUuid(null);
      setDeleteConfirmUuid(null);
      await refresh();
      // R6-P1-3: after a successful delete the trigger button's row is removed
      // by refresh(). Kobalte's Dialog onCloseAutoFocus tried to restore focus
      // to the trigger BEFORE refresh detached it, so focus is now lost to
      // body. Restore focus to a safe fallback: the first remaining provider's
      // Edit button, or the first preset button if the list is now empty.
      queueMicrotask(() => {
        if (deleteTriggerRef.current && document.contains(deleteTriggerRef.current)) {
          // Trigger still in the DOM → focus is already restored.
          return;
        }
        const firstEdit = document.querySelector<HTMLButtonElement>(
          'button[aria-label^="Edit "]',
        );
        if (firstEdit) {
          firstEdit.focus();
          return;
        }
        const firstPreset = document.querySelector<HTMLButtonElement>(".pc__preset");
        firstPreset?.focus();
      });
    } catch (e) {
      // Close the dialog and surface a Retry banner in the main area. Kobalte's
      // Dialog sets body pointer-events:none during its close transition, so an
      // in-dialog Retry button would have its clicks swallowed.
      setDeleteError(true);
      setDeleteFailedUuid(uuid);
      setDeleteConfirmUuid(null);
      pushToast("destructive", t.saveFailed);
    } finally {
      setDeletingUuid(null);
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

  /** Delete trigger: capture the trigger button (by aria-label) for focus
   *  restore, then open the Confirm. */
  const handleDelete = (uuid: string) => {
    const p = providers().find((x) => x.uuid === uuid);
    if (!p) return;
    const label = t.cardDelete.replace("{name}", p.name);
    const btn = document.querySelector<HTMLButtonElement>(
      `button[aria-label="${cssEscape(label)}"]`,
    );
    deleteTriggerRef.current = btn ?? undefined;
    setDeleteError(false);
    setDeleteFailedUuid(null);
    setDeleteConfirmUuid(uuid);
  };

  /** Reorder: optimistic local swap → IPC → revert + toast on error. */
  const moveProvider = async (uuid: string, dir: "up" | "down") => {
    if (globalMutationLock()) return; // R6-P1-1
    const ordered = [...providers()].sort((a, b) => a.sort_order - b.sort_order);
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

  // --- Detail state memo (gathered for the View) ---
  const detail = createMemo<ProviderDetailState | null>(() => {
    const p = selectedProvider();
    if (!p) return null;
    const uuid = p.uuid;
    const draftEndpoint = endpointDraft()[uuid] ?? p.endpoint;
    const draftModel = modelDraftByUuid()[uuid] ?? p.model ?? "";
    const draftName = nameDraftByUuid()[uuid] ?? p.name;
    // Reactive endpoint validation (only for drafts that differ from stored).
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
      keyText: keyInputByUuid()[uuid] ?? "",
      nameError: nameErrorByUuid()[uuid],
      keyError: keyErrorByUuid()[uuid],
      endpointError,
      saveState: saveByUuid()[uuid] ?? "idle",
      conn: connByUuid()[uuid] ?? "idle",
      modelOptions: modelOptionsByUuid()[uuid] ?? [],
      modelFetch: modelFetchByUuid()[uuid] ?? "idle",
      saveConflict: saveConflictUuid() === uuid,
    };
  });

  // --- Consent recipients for the dialog ---
  const consentRecipients = createMemo<ConsentRecipient[]>(() => {
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
    <ProviderCenterView
      t={t}
      providers={providers()}
      selection={selection()}
      selectedUuid={selectedUuid()}
      loadError={loadError()}
      selectionError={selectionError()}
      selectionLoading={selectionLoading()}
      deletingUuid={deletingUuid()}
      reloadingUuid={reloadingUuid()}
      globalMutationLock={globalMutationLock()}
      presets={PRESETS}
      detail={detail()}
      deleteConfirmUuid={deleteConfirmUuid()}
      deleteError={deleteError()}
      deleteFailedUuid={deleteFailedUuid()}
      consentOpen={consentOpen()}
      consentRecipients={consentRecipients()}
      toasts={toasts()}
      deleteTriggerRef={deleteTriggerRef}
      consentTriggerRef={consentTriggerRef}
      onToggle={(uuid, enabled) => void handleToggle(uuid, enabled)}
      onEdit={(uuid) => setSelectedUuid(uuid)}
      onDelete={(uuid) => handleDelete(uuid)}
      onSetPrimary={(uuid) => void handleSetPrimary(uuid)}
      onAddParallel={(uuid, triggerEl) => handleAddParallel(uuid, triggerEl)}
      onRemoveParallel={(uuid) => void handleRemoveParallel(uuid)}
      onSetFallback={(uuid) => void handleSetFallback(uuid)}
      onDuplicate={(uuid) => void handleDuplicate(uuid)}
      onMoveUp={(uuid) => void moveProvider(uuid, "up")}
      onMoveDown={(uuid) => void moveProvider(uuid, "down")}
      onAddPreset={(preset) => void handleAddPreset(preset)}
      onNameInput={(uuid, value) =>
        setNameDraftByUuid((prev) => {
          if (nameErrorByUuid()[uuid]) {
            const ne = { ...prev };
            delete ne[uuid];
            return { ...ne, [uuid]: value };
          }
          return { ...prev, [uuid]: value };
        })
      }
      onEndpointInput={(uuid, value) =>
        setEndpointDraft((prev) => ({ ...prev, [uuid]: value }))
      }
      onModelInput={(uuid, value) =>
        setModelDraftByUuid((prev) => ({ ...prev, [uuid]: value }))
      }
      onModelChange={(uuid, value) =>
        // Idempotent: return the SAME `prev` reference when the value is
        // unchanged. The Kobalte Select re-emits `onChange` with the current
        // value whenever the model `value`/`options` prop changes reference
        // (which happens on every `detail` memo recompute, since `selectOptions`
        // returns a fresh array). Without this guard the write always produced a
        // new Record → `detail` recompute → Select value-ref change → onChange
        // → infinite update loop (R2-H). Solid's setter short-circuits when the
        // updater returns the identical reference, breaking the cycle.
        setModelDraftByUuid((prev) =>
          prev[uuid] === value ? prev : { ...prev, [uuid]: value },
        )
      }
      onKeyInput={(uuid, value) => {
        setKeyInputByUuid((prev) => ({ ...prev, [uuid]: value }));
        if (keyErrorByUuid()[uuid]) {
          setKeyErrorByUuid((prev) => {
            const n = { ...prev };
            delete n[uuid];
            return n;
          });
        }
      }}
      onSaveProfile={(uuid) => void handleSaveProfile(uuid)}
      onSaveKey={(uuid) => void handleSaveKey(uuid)}
      onFetchModels={(uuid) => void handleFetchModels(uuid)}
      onTestConnection={(uuid) => void handleTestConnection(uuid)}
      onResolveSaveConflict={(uuid) => {
        // R5-P1-1: Reload race-condition fix. Set reloadingUuid BEFORE the await
        // so (a) a second Reload click is a no-op (re-entrancy guard) and (b) the
        // detail-panel fields for this UUID are disabled while refresh is
        // in-flight (a draft typed during the await would be unconditionally
        // discarded on success). On failure, restore editability but keep the
        // banner + drafts (refresh already surfaced the error). On success,
        // clear this UUID's drafts/errors and the conflict banner ONLY if the
        // conflict is still for THIS uuid — a different provider's conflict may
        // have appeared during the reload and must survive.
        if (reloadingUuid()) return; // prevent double-click re-entry
        setReloadingUuid(uuid);
        void (async () => {
          const ok = await refresh();
          if (!ok) {
            // Reload failed: keep banner + drafts + errors. Restore editability.
            setReloadingUuid(null);
            return;
          }
          setNameDraftByUuid((prev) => {
            const next = { ...prev };
            delete next[uuid];
            return next;
          });
          setEndpointDraft((prev) => {
            const next = { ...prev };
            delete next[uuid];
            return next;
          });
          setModelDraftByUuid((prev) => {
            const next = { ...prev };
            delete next[uuid];
            return next;
          });
          // R4-P2-2: also clear any stale name-conflict error + save state left
          // over from the rejected save. Without this, a name error from the
          // stale write would persist over the freshly-reloaded row, and the
          // failed saveState would block the next save (which echoes the
          // refreshed expected_version and should succeed).
          setNameErrorByUuid((prev) => {
            const next = { ...prev };
            delete next[uuid];
            return next;
          });
          setSaveByUuid((prev) => {
            const next = { ...prev };
            delete next[uuid];
            return next;
          });
          // R5-P1-1: conditional conflict clear — only if the conflict is still
          // for THIS uuid. A different provider's conflict may have appeared
          // during the reload and must NOT be clobbered.
          setSaveConflictUuid((prev) => (prev === uuid ? null : prev));
          setReloadingUuid(null);
        })();
      }}
      onReloadFromError={() => void refresh()}
      onRetrySelectionLoad={() => void refresh()}
      onConfirmDelete={() => void confirmDelete()}
      onCancelDelete={() => cancelDelete()}
      onRetryDelete={() => retryDelete()}
      onDismissDeleteError={() => dismissDeleteError()}
      onConfirmConsent={() => void confirmConsent()}
      onCancelConsent={() => cancelConsent()}
      onDismissToast={(id) => dismissToast(id)}
    />
  );
};

export default ProviderCenter;
