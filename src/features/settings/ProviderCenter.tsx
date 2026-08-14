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
  onCleanup,
  untrack,
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
import { invoke } from "@tauri-apps/api/core";
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
export type SupportTier = "ready" | "setup_required" | "unverified";
export type Preset = {
  templateId: string;
  name: string | null;
  endpoint: string;
  model: string | null;
  needsKey: boolean;
  auth: string;
  requiresUserEndpoint: boolean;
  notes: string | null;
  supportTier: SupportTier;
  icon: string | null;
};

type CatalogPresetDto = {
  id: string;
  label: string;
  endpoint: string;
  default_model: string;
  needs_key: boolean;
  auth: string;
  requires_user_endpoint: boolean;
  notes: string | null;
  console_url: string | null;
  support_tier: SupportTier;
  icon: string | null;
};

export function catalogDtoToPreset(dto: CatalogPresetDto): Preset {
  return {
    templateId: dto.id,
    name: dto.id === "ollama" ? null : dto.label,
    endpoint: dto.endpoint,
    model: dto.default_model || null,
    needsKey: dto.needs_key,
    auth: dto.auth,
    requiresUserEndpoint: dto.requires_user_endpoint,
    notes: dto.notes,
    supportTier: dto.support_tier,
    icon: dto.icon,
  };
}

/** Escape a literal string for use inside a CSS attribute selector. Provider
 *  names may contain characters that break `button[aria-label="..."]` otherwise. */
const cssEscape = (s: string): string =>
  s.replace(/["\\]/g, (c) => `\\${c}`);

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
  // R7-P1-1: serial operation queue busy signal — true while ANY mutation or
  // refresh is in-flight (the async mutex is held). While true, ALL sidebar
  // action buttons for ALL providers AND ALL detail-panel controls are disabled
  // (prevents a concurrent mutation from being overwritten by a refresh's
  // setProviders, and vice versa). No button appears enabled but silently
  // returns — disabled is the sole gate.
  exclusiveBusy: boolean;
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
  onToggleCustomAnthropic?: (uuid: string, anthropic: boolean) => void;
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

  // R7-P1-1: while the serial operation queue is busy (any mutation or refresh
  // is in-flight), every sidebar action for EVERY provider AND every detail-
  // panel control is disabled. Composed with the per-row deleting state below.
  const locked = () => props.exclusiveBusy;

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
        <InlineError>{t().loadFailed}</InlineError>
        <div class="pc__retry">
          <Button variant="primary" size="sm" disabled={locked()} onClick={() => props.onReloadFromError()}>
            {t().reload}
          </Button>
        </div>
      </Show>

      {/* Cold-load failure: selection read failed → fail-closed. Role mutations
          are disabled (see handler guards) until a successful retry. */}
      <Show when={props.selectionError}>
        <div class="pc__retry" role="alert">
          <span class="pc__load-failed">{t().loadFailed}</span>
          <Button variant="secondary" size="sm" disabled={locked()} onClick={() => props.onRetrySelectionLoad()}>
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
                          needsKey={p.needs_key}
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
                  title={preset.notes ?? undefined}
                >
                  <Plus size={12} />
                  <span>{preset.name ?? "Ollama"}</span>
                  <Show when={preset.supportTier !== "ready"}>
                    <span class="pc__preset-tier">
                      {preset.supportTier === "setup_required"
                        ? t().tier.setupRequired
                        : t().tier.unverified}
                    </span>
                  </Show>
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
              // R8-P1: the Test button is blocked while the user has unsaved
              // drafts — `providerTestConnection` probes the BACKEND's stored
              // config, so testing with unsaved edits would probe a config the
              // user no longer sees. The detail memo resolves each draft against
              // the stored value, so a draft equal to the stored value (a no-op
              // edit) does NOT count as unsaved.
              const hasUnsavedDrafts = () => {
                const p = d().provider;
                return (
                  d().nameDraft !== p.name ||
                  d().endpointDraft !== p.endpoint ||
                  d().modelDraft !== (p.model ?? "") ||
                  d().keyText.length > 0
                );
              };
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
                        disabled={isReloading() || locked()}
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
                    disabled={d().saveState === "saving" || isReloading() || locked()}
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
                    disabled={d().saveState === "saving" || isReloading() || locked()}
                    onInput={(e) =>
                      props.onEndpointInput(uuid(), e.currentTarget.value)
                    }
                  />
                  <Show when={d().provider.template_id === "azure-openai"}>
                    <Button
                      variant="ghost"
                      size="sm"
                      disabled={d().saveState === "saving" || isReloading() || locked()}
                      onClick={() =>
                        props.onEndpointInput(
                          uuid(),
                          "https://{resource}.openai.azure.com/openai/v1/chat/completions",
                        )
                      }
                    >
                      {t().insertAzureTemplate}
                    </Button>
                  </Show>
                  <Show when={d().provider.template_id === "kimi"}>
                    <Button
                      variant="ghost"
                      size="sm"
                      disabled={d().saveState === "saving" || isReloading() || locked()}
                      onClick={() =>
                        props.onEndpointInput(
                          uuid(),
                          "https://api.moonshot.ai/v1/chat/completions",
                        )
                      }
                    >
                      {t().useKimiGlobal}
                    </Button>
                  </Show>
                  <Show when={d().provider.template_id === "custom"}>
                    <label class="pc__anthropic-toggle">
                      <input
                        type="checkbox"
                        checked={d().provider.protocol === "anthropic"}
                        disabled={d().saveState === "saving" || isReloading() || locked()}
                        onChange={(e) =>
                          props.onToggleCustomAnthropic?.(
                            uuid(),
                            e.currentTarget.checked,
                          )
                        }
                      />
                      {t().customAnthropic}
                    </label>
                  </Show>

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
                        disabled={d().saveState === "saving" || isReloading() || locked()}
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
                        disabled={d().saveState === "saving" || isReloading() || locked()}
                        onChange={(v) =>
                          props.onModelChange(uuid(), v)
                        }
                      />
                      <Button
                        id={`fetch-models-btn-${uuid()}`}
                        variant="ghost"
                        size="sm"
                        disabled={locked() || hasUnsavedDrafts()}
                        aria-describedby={hasUnsavedDrafts() ? `fetch-hint-${uuid()}` : undefined}
                        onClick={() => props.onFetchModels(uuid())}
                      >
                        {t().fetchModels}
                      </Button>
                      {/* R9-fix: same aria-describedby pattern as the Test button.
                          Fetch reads the BACKEND's stored config, so fetching with
                          unsaved edits would return models for a config the user
                          no longer sees — the hint explains why Fetch is disabled. */}
                      <Show when={hasUnsavedDrafts()}>
                        <span id={`fetch-hint-${uuid()}`} class="pc__save-first-hint" role="status">
                          {t().saveFirstToFetch}
                        </span>
                      </Show>
                    </div>
                  </Show>

                  <div class="pc__save-row">
                    <Button
                      variant="primary"
                      size="sm"
                      loading={d().saveState === "saving"}
                      disabled={isReloading() || locked()}
                      onClick={() => props.onSaveProfile(uuid())}
                    >
                      {t().saveProfile}
                    </Button>
                    <Show when={d().saveState === "saved"}>
                      <span class="pc__saved-note">{t().profileSaved}</span>
                    </Show>
                  </div>

                  {/* Key section — R11 three-state.
                      needs_key=false → "No key required" text (no input/button).
                      needs_key=true + hasKey → "Key saved" badge.
                      needs_key=true + no key → key input + Save button. */}
                  <div class="pc__key-section">
                    <Show
                      when={d().provider.needs_key}
                      fallback={
                        <span class="pc__key-not-required">
                          {t().noKeyRequired}
                        </span>
                      }
                    >
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
                              disabled={d().saveState === "saving" || locked()}
                              onInput={(e) => {
                                props.onKeyInput(uuid(), e.currentTarget.value);
                              }}
                            />
                            <Button
                              variant="primary"
                              size="sm"
                              disabled={d().keyText.trim().length === 0 || locked()}
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
                    </Show>
                  </div>

                  {/* Connection test */}
                  <div class="pc__conn-section">
                    {/* R8-P1: `providerTestConnection` probes the BACKEND's stored
                        config, NOT the user's unsaved drafts. `hasUnsavedDrafts`
                        blocks the Test button and surfaces `saveFirstToTest` so
                        the user saves before probing. */}
                    <Button
                      id={`test-conn-btn-${uuid()}`}
                      variant="secondary"
                      size="sm"
                      loading={d().conn === "testing"}
                      disabled={locked() || hasUnsavedDrafts()}
                      aria-describedby={hasUnsavedDrafts() ? `save-first-hint-${uuid()}` : undefined}
                      onClick={() => props.onTestConnection(uuid())}
                    >
                      {t().testConnection}
                    </Button>
                    <Show when={hasUnsavedDrafts()}>
                      <span id={`save-first-hint-${uuid()}`} class="pc__save-first-hint" role="status">
                        {t().saveFirstToTest}
                      </span>
                    </Show>
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
          <Button variant="secondary" size="sm" disabled={locked()} onClick={() => props.onRetryDelete()}>
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

  // R7-P1-1: disposal flag — prevents stale setTimeout callbacks (e.g. the
  // post-delete focus restoration) from running after the component unmounts
  // and stealing focus in a subsequently-rendered instance (test isolation).
  let disposed = false;
  onCleanup(() => { disposed = true; });

  // --- Core state ---
  const [presets, setPresets] = createSignal<Preset[]>([]);
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
  // R8-P2-1: per-UUID request counter for Fetch Models. Same ABA rationale as
  // `connRequestIdByUuid`: a stale completion (from an earlier Fetch whose await
  // resolved after a newer Fetch, OR after a save bumped the config version) is
  // discarded. Paired with the config-version guard in handleFetchModels.
  const [modelRequestIdByUuid, setModelRequestIdByUuid] = createSignal<Record<string, number>>({});
  // R9: unified configEpoch — a single monotonic counter per UUID. Bumped on
  // ANY config-relevant change (draft edit, model select, key save, provider
  // update, delete). Test/Fetch capture the epoch at start; on completion they
  // discard if the epoch changed. This replaces the per-field version/endpoint/
  // model guards from R8, which could not catch draft edits or key saves (those
  // don't change `providers()` committed state). The epoch is monotonic — it
  // catches changes even if the draft reverts to the original value.
  const [configEpochByUuid, setConfigEpochByUuid] = createSignal<Record<string, number>>({});

  /** Bump the config epoch for a UUID and clear stale conn/model state. Called
   *  on ANY config-relevant change so in-flight Test/Fetch completions are
   *  discarded by the epoch guard, and the UI does not show stale results. */
  const bumpConfigEpoch = (uuid: string) => {
    setConfigEpochByUuid((prev) => ({ ...prev, [uuid]: (prev[uuid] ?? 0) + 1 }));
    setConnByUuid((prev) => { const n = { ...prev }; delete n[uuid]; return n; });
    setModelOptionsByUuid((prev) => { const n = { ...prev }; delete n[uuid]; return n; });
    setModelFetchByUuid((prev) => { const n = { ...prev }; delete n[uuid]; return n; });
  };

  /** Unified list-replacement entry point (R10). Bumps the config epoch for ALL
   *  old UUIDs (conservative — any field may have changed on the backend) and
   *  cleans up per-UUID request counters for deleted UUIDs. Every full-list
   *  refresh from the backend MUST go through this — never call
   *  `setProviders(list)` directly for a backend list.
   *
   *  The old refreshCore diff only checked version/endpoint/model, missing
   *  enabled/hasKey/protocol/capabilities. Bumping ALL old UUIDs eliminates
   *  field-by-field comparison gaps entirely. configEpochByUuid is intentionally
   *  NOT deleted for removed UUIDs — the bumped value must remain so the epoch
   *  guard in in-flight Test/Fetch sees a changed epoch and discards the stale
   *  completion. bumpConfigEpoch already clears conn/modelOptions/modelFetch. */
  const applyProviderList = (newList: ProviderProfileFE[]) => {
    const oldList = providers();
    for (const p of oldList) {
      bumpConfigEpoch(p.uuid);
    }
    for (const old of oldList) {
      if (!newList.some((p) => p.uuid === old.uuid)) {
        setConnRequestIdByUuid((prev) => { const n = { ...prev }; delete n[old.uuid]; return n; });
        setModelRequestIdByUuid((prev) => { const n = { ...prev }; delete n[old.uuid]; return n; });
      }
    }
    setProviders(newList);
  };

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
  // R7-P1-1: serial operation queue (async mutex). ALL provider mutations AND
  // refresh run exclusively — no two operations overlap, so a concurrent
  // mutation's setProviders(...) can never be overwritten by a refresh's
  // setProviders(list) resolving mid-mutation. Mutations that need a re-fetch
  // call refreshCore() directly (NOT refresh, which re-enters the mutex →
  // deadlock). The boolean lock from R6 could not handle: (a) mutations already
  // in-flight when Reload starts, (b) overlapping refreshes, (c) mutations that
  // internally call refresh (create/delete/duplicate). The async mutex handles
  // all three by serializing.
  let exclusiveInProgress = false;
  const exclusiveQueue: Array<() => void> = [];
  const [exclusiveBusy, setExclusiveBusy] = createSignal(false);

  async function runExclusive<T>(fn: () => Promise<T>): Promise<T> {
    // Wait for any in-flight operation to finish before starting.
    while (exclusiveInProgress) {
      await new Promise<void>((resolve) => exclusiveQueue.push(resolve));
    }
    exclusiveInProgress = true;
    setExclusiveBusy(true);
    try {
      return await fn();
    } finally {
      exclusiveInProgress = false;
      setExclusiveBusy(false);
      // Wake the next queued operation.
      const next = exclusiveQueue.shift();
      if (next) next();
    }
  }
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
  /** Raw re-fetch of the provider list + stored selection (NO mutex management).
   *  Returns `true` on success, `false` on failure. The error is surfaced via
   *  loadError + selectionError signals (which render the banners); callers
   *  that need a toast branch on the boolean. refreshCore does NOT push toasts
   *  itself (R9: previously it pushed a destructive `saveFailed` toast, which
   *  contradicted mutation handlers that also pushed `mutationSuccessReloadFailed`
   *  on the same failure — the user saw both).
   *
   *  R7-P1-1: callers that are ALREADY inside `runExclusive` (mutation handlers
   *  that need a post-mutation re-fetch) call this directly. The public
   *  `refresh()` wraps this in the mutex. Never call `refresh()` from inside a
   *  `runExclusive` body — it re-enters the mutex and deadlocks. */
  const refreshCore = async (): Promise<boolean> => {
    setSelectionLoading(true);
    setSelectionError(false);
    try {
      const [list, active] = await Promise.all([
        loadProviders(),
        providerGetActiveSelection(),
      ]);
      // R10: unified applyProviderList bumps the epoch for ALL old UUIDs
      // (conservative) and cleans up deleted UUIDs — replacing the narrow
      // version/endpoint/model diff that missed enabled/hasKey/protocol changes.
      applyProviderList(list);
      setSelection({
        primaryUuid: active.primary,
        parallelUuids: active.parallel,
        fallbackUuid: active.fallback,
      });
      setLoadError(false);
      return true;
    } catch {
      setLoadError(true);
      setSelectionError(true);
      return false;
    } finally {
      setSelectionLoading(false);
    }
  };

  /** Public refresh entry — acquires the serial-operation mutex, then runs
   *  refreshCore. Use for top-level triggers (onMount, error-retry banners).
   *  Mutation handlers that are already inside `runExclusive` must call
   *  `refreshCore()` directly to avoid re-entering the mutex. */
  const refresh = (): Promise<boolean> => runExclusive(() => refreshCore());

  onMount(() => {
    void invoke<CatalogPresetDto[]>("provider_list_presets")
      .then((rows) => setPresets(rows.map(catalogDtoToPreset)))
      .catch(() => setPresets([]));
    // R10: reloadFailed for cold-load failure (a READ failure, not a save).
    void refresh().then((ok) => {
      if (!ok) pushToast("destructive", t.reloadFailed);
    });
  });

  const selectedProvider = createMemo(() =>
    providers().find((p) => p.uuid === selectedUuid()),
  );

  // --- Mutations ---

  /** Toggle: optimistic flip → IPC → revert + toast on error. */
  const handleToggle = async (uuid: string, enabled: boolean) => {
    await runExclusive(async () => {
      // R10: toggle changes `enabled` — bump the configEpoch BEFORE the optimistic
      // setProviders so a pending Test/Fetch (probing the old enabled state) is
      // invalidated. bumpConfigEpoch also clears stale conn/model state. On
      // rollback the epoch is NOT rolled back (monotonic — correct).
      bumpConfigEpoch(uuid);
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
      } catch {
        setProviders(prev); // rollback
        pushToast("destructive", t.saveFailed);
      }
    });
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
    await runExclusive(async () => {
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
      } catch {
        pushToast("destructive", t.saveFailed);
      }
    });
  };

  const handleAddParallel = (uuid: string, triggerEl?: HTMLElement) => {
    if (selectionLoading() || selectionError()) return;
    if (triggerEl) consentTriggerRef.current = triggerEl;
    void runExclusive(async () => {
      const candidate: ActiveSelection = {
        ...selection(),
        parallelUuids: [...selection().parallelUuids, uuid],
        fallbackUuid: selection().fallbackUuid === uuid ? null : selection().fallbackUuid,
      };
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
      } catch {
        pushToast("destructive", t.saveFailed);
      }
    });
  };

  const confirmConsent = async () => {
    const uuid = pendingParallelUuid();
    if (!uuid) return;
    if (selectionLoading() || selectionError()) return;
    await runExclusive(async () => {
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
      } catch {
        pushToast("destructive", t.saveFailed);
        setConsentOpen(false);
        setPendingParallelUuid(null);
        setConsentActualScope(null);
      }
    });
  };

  const cancelConsent = () => {
    setConsentOpen(false);
    setPendingParallelUuid(null);
    setConsentActualScope(null);
  };

  const handleSetFallback = async (uuid: string) => {
    if (selectionLoading() || selectionError()) return;
    await runExclusive(async () => {
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
      } catch {
        pushToast("destructive", t.saveFailed);
      }
    });
  };

  const handleRemoveParallel = async (uuid: string) => {
    if (selectionLoading() || selectionError()) return;
    await runExclusive(async () => {
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
      } catch {
        pushToast("destructive", t.saveFailed);
      }
    });
  };

  const handleAddPreset = async (preset: Preset) => {
    await runExclusive(async () => {
      const name = preset.name ?? "Ollama";
      try {
        await providerCreate(preset.templateId, name, preset.endpoint, preset.model ?? undefined);
        // R8-P2-2: refreshCore never rejects — it surfaces the error via the
        // loadError banner and returns false. Branch on the boolean so a
        // successful create whose list-refresh failed does NOT also show a
        // misleading success toast.
        const ok = await refreshCore();
        if (ok) {
          pushToast("success", t.profileSaved);
        } else {
          pushToast("warning", t.mutationSuccessReloadFailed);
        }
      } catch {
        pushToast("destructive", t.saveFailed);
      }
    });
  };

  /** Duplicate a provider: new UUID, new secret_ref, keyless. Re-fetches the
   *  list so the clone appears. */
  const handleDuplicate = async (uuid: string) => {
    await runExclusive(async () => {
      try {
        await providerDuplicate(uuid);
        // R8-P2-2: same boolean branch as handleAddPreset.
        const ok = await refreshCore();
        if (ok) {
          pushToast("success", t.profileSaved);
        } else {
          pushToast("warning", t.mutationSuccessReloadFailed);
        }
      } catch {
        pushToast("destructive", t.saveFailed);
      }
    });
  };

  const handleToggleCustomAnthropic = async (uuid: string, anthropic: boolean) => {
    await runExclusive(async () => {
      const provider = providers().find((p) => p.uuid === uuid);
      if (!provider || provider.template_id !== "custom") return;
      try {
        const updated = await providerUpdate(uuid, {
          expected_version: provider.version,
          protocol: anthropic ? "anthropic" : "openai_chat",
        });
        setProviders((prev) =>
          prev.map((p) => (p.uuid === uuid ? { ...p, ...updated, hasKey: p.hasKey } : p)),
        );
        bumpConfigEpoch(uuid);
        pushToast("success", t.profileSaved);
      } catch {
        pushToast("destructive", t.saveFailed);
      }
    });
  };

  /** Save profile: validate endpoint locally (reactive epError already shown),
   *  then IPC. Aborts on invalid endpoint or a duplicate-name conflict. */
  const handleSaveProfile = async (uuid: string) => {
    await runExclusive(async () => {
      const draft = endpointDraft()[uuid];
      const modelDraft = modelDraftByUuid()[uuid];
      const nameDraft = nameDraftByUuid()[uuid];
      const provider = providers().find((p) => p.uuid === uuid);
      if (!provider) return;
      const effectiveEndpoint = draft ?? provider.endpoint;
      const allowEmpty =
        provider.template_id === "custom" || provider.template_id === "azure-openai";
      const epCheck = validateEndpoint(effectiveEndpoint, { allowEmpty });
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
        // R9: the save bumped the config version, so any in-flight or completed
        // connection-test / model-fetch result is now for a stale config.
        // bumpConfigEpoch invalidates pending completions via the epoch guard
        // and clears the cached results so the UI does not show a stale badge
        // or model list next to the freshly-saved row.
        bumpConfigEpoch(uuid);
        pushToast("success", t.profileSaved);
      } catch (e) {
        // R2-E: a structured stale_version rejection = save conflict. Keep the
        // user's draft intact (do NOT overwrite) and surface a conflict banner
        // with a Reload button so they can pull fresh data and reconcile.
        const err = e as { error?: string };
        if (err?.error === "stale_version") {
          // R10: the remote config changed — bump the configEpoch so any pending
          // Test/Fetch started before this rejected save is invalidated when it
          // completes.
          bumpConfigEpoch(uuid);
          setSaveByUuid((prev) => ({ ...prev, [uuid]: "failed" }));
          setSaveConflictUuid(uuid);
        } else {
          setSaveByUuid((prev) => ({ ...prev, [uuid]: "failed" }));
          pushToast("destructive", t.saveFailed);
        }
      }
    });
  };

  /**
   * Save key: clear the input IMMEDIATELY (never re-readable), then IPC.
   * On success re-fetch key_status to update `hasKey`. The key is NEVER
   * re-read from the input after submit — the input stays cleared.
   */
  const handleSaveKey = async (uuid: string) => {
    const provider = providers().find((p) => p.uuid === uuid);
    if (!provider) return;
    // R11 (P1): fail-closed — a keyless provider (needs_key=false) must never
    // save a key. The detail Key section hides the input for needs_key=false,
    // so this guard is defense-in-depth: a programmatic call could otherwise
    // write a dangling secret the provider will never read (and the backend now
    // rejects it too — see provider_set_key / set_key_blocking).
    if (!provider.needs_key) return;
    // R11 (P1): fail-closed — an empty/whitespace key must never reach the
    // backend. The Save button is disabled while the input is empty, but guard
    // anyway so a race or a programmatic call cannot send an empty key.
    const pendingKey = keyInputByUuid()[uuid];
    if (typeof pendingKey !== "string" || pendingKey.trim().length === 0) return;
    // R9: bump configEpoch at the START (before the await) — the key is
    // changing, so any in-flight Test/Fetch started with the old key must be
    // invalidated. The key is never re-readable, so this bump must happen
    // before the await to guarantee it precedes any pending completion.
    bumpConfigEpoch(uuid);
    await runExclusive(async () => {
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
        // Re-fetch to update hasKey. R10: route through applyProviderList (NOT
        // a bare setProviders) so the epoch is bumped for ALL old UUIDs — this
        // invalidates any pending Test/Fetch for any provider whose config may
        // have changed in the refresh (e.g. this key save flipping hasKey).
        const list = await loadProviders();
        applyProviderList(list);
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
    });
  };

  const handleFetchModels = async (uuid: string) => {
    // R9: `providerGetModels` reads the BACKEND's stored config. Capture the
    // configEpoch at fetch start so a completion that lands after ANY
    // config-relevant change (draft edit, model select, key save, provider
    // update) is discarded instead of writing a model list for a config the
    // user no longer sees.
    const provider = providers().find((p) => p.uuid === uuid);
    if (!provider) return;
    const epoch = configEpochByUuid()[uuid] ?? 0;
    // Bump the request counter so a stale completion (from an earlier Fetch
    // whose await resolved after a newer Fetch started) is discarded.
    const requestId = (modelRequestIdByUuid()[uuid] ?? 0) + 1;
    setModelRequestIdByUuid((prev) => ({ ...prev, [uuid]: requestId }));
    setModelFetchByUuid((prev) => ({ ...prev, [uuid]: "loading" }));
    try {
      const models = await providerGetModels(uuid);
      // Guard 1: config changed since fetch start (epoch bumped). Both sides
      // use `?? 0` so an unset epoch (undefined) compares equal to itself.
      if ((configEpochByUuid()[uuid] ?? 0) !== epoch) return;
      // Guard 2: a newer Fetch superseded this one.
      if (modelRequestIdByUuid()[uuid] !== requestId) return;
      setModelOptionsByUuid((prev) => ({ ...prev, [uuid]: models }));
      setModelFetchByUuid((prev) => ({ ...prev, [uuid]: "idle" }));
    } catch {
      if ((configEpochByUuid()[uuid] ?? 0) !== epoch) return;
      if (modelRequestIdByUuid()[uuid] !== requestId) return;
      setModelFetchByUuid((prev) => ({ ...prev, [uuid]: "error" }));
      // Surface the failure so the user knows why no dropdown appeared.
      pushToast("warning", t.modelFetchError);
    }
  };

  const handleTestConnection = async (uuid: string) => {
    // R9: `providerTestConnection` probes the BACKEND's stored config, NOT the
    // user's unsaved drafts. Capture the configEpoch at test start so a
    // completion that lands AFTER any config-relevant change (draft edit, model
    // select, key save, provider update) is discarded — otherwise a stale
    // "Connected" from the old config would mislead the user into trusting a
    // config they already replaced.
    const provider = providers().find((p) => p.uuid === uuid);
    if (!provider) return;
    const epoch = configEpochByUuid()[uuid] ?? 0;
    // Bump the request counter so a stale completion (from an earlier Test
    // click whose await resolved after a newer Test) is discarded.
    const requestId = (connRequestIdByUuid()[uuid] ?? 0) + 1;
    setConnRequestIdByUuid((prev) => ({ ...prev, [uuid]: requestId }));
    setConnByUuid((prev) => ({ ...prev, [uuid]: "testing" }));
    try {
      const result = await providerTestConnection(uuid);
      // Guard 1: config changed since test start (epoch bumped). Both sides
      // use `?? 0` so an unset epoch (undefined) compares equal to itself.
      if ((configEpochByUuid()[uuid] ?? 0) !== epoch) return;
      // Guard 2: a newer Test superseded this one.
      if (connRequestIdByUuid()[uuid] !== requestId) return;
      setConnByUuid((prev) => ({ ...prev, [uuid]: result }));
    } catch {
      if ((configEpochByUuid()[uuid] ?? 0) !== epoch) return;
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
    // R9: invalidate any in-flight Test/Fetch for the deleted UUID.
    bumpConfigEpoch(uuid);
    await runExclusive(async () => {
      setDeletingUuid(uuid);
      try {
        await providerDelete(uuid);
        setDeleteError(false);
        setDeleteFailedUuid(null);
        setDeleteConfirmUuid(null);
        // R8-P2-2: refreshCore never rejects — branch on its boolean. The row
        // was deleted on the backend; only the list refresh can fail. On
        // failure the provider list is in an unknown state, so the DOM-dependent
        // focus restoration (querySelector for the next row's Edit button) is
        // UNSAFE — skip it. refreshCore already set loadError; the warning toast
        // below is the sole user-facing signal for this failure.
        const ok = await refreshCore();
        if (!ok) {
          pushToast("warning", t.mutationSuccessReloadFailed);
          return;
        }
        // R6-P1-3: after a successful delete the trigger button's row is removed
        // by refreshCore(). Kobalte's Dialog onCloseAutoFocus tried to restore
        // focus to the trigger BEFORE refresh detached it, so focus is now lost
        // to body. Restore focus to a safe fallback: the first remaining
        // provider's Edit button, or the first preset button if the list is
        // empty.
        // R7-P1-1: setTimeout (not queueMicrotask) so this runs AFTER all
        // microtask-based dialog-close focus restoration settles — the async
        // mutex changes the microtask ordering, and a macrotask guarantees our
        // focus wins over Kobalte's close-transition auto-focus. The disposed
        // guard prevents this callback from stealing focus in a new instance
        // after unmount (test isolation).
        setTimeout(() => {
          if (disposed) return;
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
      } catch {
        // Close the dialog and surface a Retry banner in the main area.
        // Kobalte's Dialog sets body pointer-events:none during its close
        // transition, so an in-dialog Retry button would have its clicks
        // swallowed.
        setDeleteError(true);
        setDeleteFailedUuid(uuid);
        setDeleteConfirmUuid(null);
        pushToast("destructive", t.saveFailed);
      } finally {
        setDeletingUuid(null);
      }
    });
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
    await runExclusive(async () => {
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
      } catch {
        // Revert to snapshot order.
        setProviders(
          snapshot.map((p, i) => ({ ...p, sort_order: i })),
        );
        pushToast("destructive", t.reorderReverted);
      }
    });
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
      exclusiveBusy={exclusiveBusy()}
      presets={presets()}
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
      onEndpointInput={(uuid, value) => {
        setEndpointDraft((prev) => ({ ...prev, [uuid]: value }));
        // R9: an endpoint draft means the displayed config no longer matches
        // what was last tested / fetched against. bumpConfigEpoch invalidates
        // any in-flight Test/Fetch (whose completion would be stale) and clears
        // the cached results so a stale "Connected" badge or model list is not
        // shown next to the edited endpoint.
        bumpConfigEpoch(uuid);
      }}
      onModelInput={(uuid, value) => {
        setModelDraftByUuid((prev) => ({ ...prev, [uuid]: value }));
        // R9: same rationale as onEndpointInput — a model draft invalidates
        // the last-tested / last-fetched config.
        bumpConfigEpoch(uuid);
      }}
      onModelChange={(uuid, value) => {
        // Idempotent: the Kobalte Select re-emits `onChange` with the current
        // value whenever the model `value`/`options` prop changes reference
        // (which happens on every `detail` memo recompute, since `selectOptions`
        // returns a fresh array). Without this guard the write always produced a
        // new Record → `detail` recompute → Select value-ref change → onChange
        // → infinite update loop (R2-H).
        //
        // R9: when the value IS different, bumpConfigEpoch invalidates any
        // in-flight Test/Fetch and clears the stale model list. The effective
        // model (draft ?? stored) is compared so that the Select's initial
        // onChange re-emission (when the draft is undefined but the value
        // matches the stored model) does NOT trigger a spurious epoch bump.
        // `untrack` prevents reactive dependencies if Kobalte calls onChange
        // inside a createEffect.
        const effectiveModel = untrack(() => {
          const draft = modelDraftByUuid()[uuid];
          if (draft !== undefined) return draft;
          return providers().find((p) => p.uuid === uuid)?.model ?? "";
        });
        if (effectiveModel === value) return;
        setModelDraftByUuid((prev) => ({ ...prev, [uuid]: value }));
        bumpConfigEpoch(uuid);
      }}
      onKeyInput={(uuid, value) => {
        setKeyInputByUuid((prev) => ({ ...prev, [uuid]: value }));
        // R10: a non-empty key draft is an unsaved config change — bump the
        // configEpoch so any in-flight Test/Fetch started against the old
        // (keyless or old-key) config is invalidated, and clear stale conn/model
        // results. An empty value (cleared draft) does NOT bump — no change.
        if (value.length > 0) bumpConfigEpoch(uuid);
        if (keyErrorByUuid()[uuid]) {
          setKeyErrorByUuid((prev) => {
            const n = { ...prev };
            delete n[uuid];
            return n;
          });
        }
      }}
      onSaveProfile={(uuid) => void handleSaveProfile(uuid)}
      onToggleCustomAnthropic={(uuid, on) => void handleToggleCustomAnthropic(uuid, on)}
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
        // R7-P1-1: the whole body runs inside runExclusive (serial operation
        // queue) and calls refreshCore directly (NOT refresh — the mutex is
        // already held here, re-entering would deadlock).
        if (reloadingUuid()) return; // prevent double-click re-entry
        setReloadingUuid(uuid);
        void runExclusive(async () => {
          const ok = await refreshCore();
          if (!ok) {
            // R10: reloadFailed for the save-conflict Reload failure (a READ
            // failure). Keep banner + drafts + errors. Restore editability.
            pushToast("destructive", t.reloadFailed);
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
        });
      }}
      onReloadFromError={() => {
        // R10: reloadFailed for the top-level Reload failure (a READ failure).
        void refresh().then((ok) => {
          if (!ok) pushToast("destructive", t.reloadFailed);
        });
      }}
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
