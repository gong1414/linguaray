import { type Component } from "solid-js";
// rev-8-2: the `@app` alias resolves to <repo>/src. The lab imports the
// PRODUCTION presentational View (ProviderCenterView) and feeds canned data —
// no second mock UI, no OpRegistry, no mock timers. This is the InputPanelView
// pattern applied to Surface 05. Production CSS (ProviderCenter.css) loads via
// this import; the lab no longer owns a provider-center stylesheet.
import {
  ProviderCenterView,
  PRESETS,
  type ProviderCenterViewProps,
  type ProviderDetailState,
} from "@app/features/settings/ProviderCenter";
import { SETTINGS_COPY } from "@app/features/settings/copy";
import type {
  ProviderProfileFE,
  ActiveSelection,
  ConnectionResult,
} from "@app/features/settings/provider-types";
import type { ProviderState } from "../i18n";

export type ProviderCenterProps = {
  state: ProviderState;
};

// rev-7-7: FIXED canned data — NO invoke calls. The lab is a pure renderer.
// The shape mirrors ProviderProfileFE (the production View's prop type) so a
// contract change surfaces here as a compile error.

const CAPS_AI = { balance: true, quota: false, model_list: true };
const CAPS_TRAD = { balance: false, quota: false, model_list: false };

/** Canned provider list (NO real keys — secret_ref is a non-sensitive label). */
function initialProviders(): ProviderProfileFE[] {
  return [
    { uuid: "mock-openai-1", template_id: "openai", name: "OpenAI #1", protocol: "openai_chat", endpoint: "https://api.openai.com/v1/chat/completions", model: "gpt-4o", enabled: true, sort_order: 0, is_local: false, needs_key: true, secret_ref: "key/mock-openai-1", capabilities: CAPS_AI, status: "active", version: 1, hasKey: true },
    { uuid: "mock-openai-2", template_id: "openai", name: "OpenAI #2", protocol: "openai_chat", endpoint: "https://api.openai.com/v1/chat/completions", model: "gpt-4o-mini", enabled: true, sort_order: 1, is_local: false, needs_key: true, secret_ref: "", capabilities: CAPS_AI, status: "active", version: 1, hasKey: false },
    { uuid: "mock-deepseek", template_id: "deepseek", name: "DeepSeek", protocol: "openai_chat", endpoint: "https://api.deepseek.com/v1/chat/completions", model: "deepseek-chat", enabled: true, sort_order: 2, is_local: false, needs_key: true, secret_ref: "key/mock-deepseek", capabilities: CAPS_AI, status: "active", version: 1, hasKey: true },
    { uuid: "mock-google", template_id: "google", name: "Google Translate", protocol: "google_translate", endpoint: "https://translation.googleapis.com/", model: null, enabled: true, sort_order: 3, is_local: false, needs_key: true, secret_ref: "key/mock-google", capabilities: CAPS_TRAD, status: "active", version: 1, hasKey: true },
    { uuid: "mock-ollama", template_id: "ollama", name: "Ollama", protocol: "custom_http", endpoint: "http://localhost:11434/v1/chat/completions", model: "llama3", enabled: false, sort_order: 4, is_local: true, needs_key: false, secret_ref: "", capabilities: CAPS_AI, status: "active", version: 1, hasKey: false },
  ];
}

const DEFAULT_SELECTION: ActiveSelection = {
  primaryUuid: "mock-openai-1",
  parallelUuids: ["mock-deepseek"],
  fallbackUuid: "mock-google",
};

const T = SETTINGS_COPY.en.provider;

const noop = () => {};

/** Build the canned detail-panel state for the selected provider, with
 *  per-ProviderState tweaks for the visually distinct demo states. States not
 *  tested by the visual baseline degrade to a clean idle populated view. */
function detailFor(
  p: ProviderProfileFE,
  state: ProviderState,
): ProviderDetailState {
  let saveState: ProviderDetailState["saveState"] = "idle";
  if (state === "saving") saveState = "saving";
  else if (state === "save-failed") saveState = "failed";
  else if (state === "key-saved") saveState = "saved";

  let conn: ProviderDetailState["conn"] = "idle";
  if (state === "connection-testing") conn = "testing";
  else if (state === "connection-ok") {
    const r: ConnectionResult = { ok: true, message: T.connectionOk, latency_ms: 42 };
    conn = r;
  } else if (state === "connection-failed") {
    const r: ConnectionResult = { ok: false, message: T.connectionFailed };
    conn = r;
  }

  let modelFetch: ProviderDetailState["modelFetch"] = "idle";
  if (state === "loading-models") modelFetch = "loading";
  else if (state === "model-fetch-error") modelFetch = "error";

  return {
    provider: p,
    nameDraft: p.name,
    endpointDraft: p.endpoint,
    modelDraft: p.model ?? "",
    keyText: "",
    nameError: undefined,
    keyError: undefined,
    endpointError: state === "endpoint-invalid" ? T.endpoint.errors["endpoint-must-https"] : undefined,
    saveState,
    conn,
    modelOptions: [],
    modelFetch,
    saveConflict: state === "save-conflict",
  };
}

const ProviderCenter: Component<ProviderCenterProps> = (props) => {
  const propsFor = (): ProviderCenterViewProps => {
    const t = T;

    // empty: no providers, no selection, no detail.
    if (props.state === "empty") {
      return {
        t,
        providers: [],
        selection: { primaryUuid: null, parallelUuids: [], fallbackUuid: null },
        selectedUuid: null,
        loadError: false,
        selectionError: false,
        selectionLoading: false,
        deletingUuid: null,
        reloadingUuid: null,
        presets: PRESETS,
        detail: null,
        deleteConfirmUuid: null,
        deleteError: false,
        deleteFailedUuid: null,
        consentOpen: false,
        consentRecipients: [],
        toasts: [],
        deleteTriggerRef: {},
        consentTriggerRef: {},
        onToggle: noop,
        onEdit: noop,
        onDelete: noop,
        onSetPrimary: noop,
        onAddParallel: noop,
        onRemoveParallel: noop,
        onSetFallback: noop,
        onDuplicate: noop,
        onMoveUp: noop,
        onMoveDown: noop,
        onAddPreset: noop,
        onNameInput: noop,
        onEndpointInput: noop,
        onModelInput: noop,
        onModelChange: noop,
        onKeyInput: noop,
        onSaveProfile: noop,
        onSaveKey: noop,
        onFetchModels: noop,
        onTestConnection: noop,
        onResolveSaveConflict: noop,
        onReloadFromError: noop,
        onRetrySelectionLoad: noop,
        onConfirmDelete: noop,
        onCancelDelete: noop,
        onRetryDelete: noop,
        onDismissDeleteError: noop,
        onConfirmConsent: noop,
        onCancelConsent: noop,
        onDismissToast: noop,
      };
    }

    // Populated fixture. State-specific tweaks:
    //  - key-missing / saving / save-failed: select OpenAI #2 (no key) so the
    //    key-entry form + save state are visible.
    //  - duplicate: append a duplicated OpenAI #1 clone.
    //  - deleting / delete-retry: mark OpenAI #1 as deleting.
    //  - delete-confirm: open the delete dialog on OpenAI #1.
    let providers = initialProviders();
    let selection: ActiveSelection = { ...DEFAULT_SELECTION };
    let selectedUuid: string | null = "mock-openai-1";
    let deletingUuid: string | null = null;
    let deleteConfirmUuid: string | null = null;
    let deleteError = false;
    let deleteFailedUuid: string | null = null;

    if (props.state === "duplicate") {
      const orig = providers.find((p) => p.uuid === "mock-openai-1")!;
      providers = [
        ...providers,
        { ...orig, uuid: "mock-openai-dup", name: `${orig.name} ${T.copySuffix}`, hasKey: false, secret_ref: "", sort_order: providers.length },
      ];
    } else if (props.state === "deleting" || props.state === "delete-retry") {
      providers = providers.map((p) =>
        p.uuid === "mock-openai-1" ? { ...p, status: "deleting" as const, enabled: false } : p,
      );
      selection = {
        primaryUuid: null,
        parallelUuids: selection.parallelUuids.filter((u) => u !== "mock-openai-1"),
        fallbackUuid: selection.fallbackUuid,
      };
      deletingUuid = "mock-openai-1";
      selectedUuid = "mock-openai-2";
      if (props.state === "delete-retry") {
        deleteError = true;
        deleteFailedUuid = "mock-openai-1";
      }
    } else if (props.state === "delete-confirm") {
      deleteConfirmUuid = "mock-openai-1";
    }

    if (props.state === "key-missing" || props.state === "saving" || props.state === "save-failed") {
      selectedUuid = "mock-openai-2";
    }

    const selected = providers.find((p) => p.uuid === selectedUuid) ?? null;
    const detail = selected ? detailFor(selected, props.state) : null;

    return {
      t,
      providers,
      selection,
      selectedUuid,
      loadError: false,
      selectionError: false,
      selectionLoading: false,
      deletingUuid,
      reloadingUuid: null,
      presets: PRESETS,
      detail,
      deleteConfirmUuid,
      deleteError,
      deleteFailedUuid,
      consentOpen: false,
      consentRecipients: [],
      toasts: [],
      deleteTriggerRef: {},
      consentTriggerRef: {},
      onToggle: noop,
      onEdit: noop,
      onDelete: noop,
      onSetPrimary: noop,
      onAddParallel: noop,
      onRemoveParallel: noop,
      onSetFallback: noop,
      onDuplicate: noop,
      onMoveUp: noop,
      onMoveDown: noop,
      onAddPreset: noop,
      onNameInput: noop,
      onEndpointInput: noop,
      onModelInput: noop,
      onModelChange: noop,
      onKeyInput: noop,
      onSaveProfile: noop,
      onSaveKey: noop,
      onFetchModels: noop,
      onTestConnection: noop,
      onResolveSaveConflict: noop,
      onReloadFromError: noop,
      onRetrySelectionLoad: noop,
      onConfirmDelete: noop,
      onCancelDelete: noop,
      onRetryDelete: noop,
      onDismissDeleteError: noop,
      onConfirmConsent: noop,
      onCancelConsent: noop,
      onDismissToast: noop,
    };
  };

  return (
    <ProviderCenterView {...propsFor()} />
  );
};

export default ProviderCenter;
