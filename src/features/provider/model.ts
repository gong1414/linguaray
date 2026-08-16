/**
 * TS mirrors of the provider IPC wire shapes (decode-only).
 * Field names match the Rust structs verbatim (snake_case on the wire);
 * command PARAMETER names are camelCased by Tauri — see ./ipc.ts.
 */

export type ProviderProtocol =
  | "openai_chat"
  | "anthropic"
  | "gemini"
  | "google_translate"
  | "custom_http"
  | "deepl"
  | "microsoft"
  | "baidu"
  | "youdao"
  | "tencent";

export type ProviderStatus = "active" | "deleting" | "deleted";

export type ProviderCapabilities = {
  balance: boolean;
  quota: boolean;
  model_list: boolean;
  auth?: string;
  models_url?: string | null;
};

/** Faithful mirror of `ProviderProfile` (db/providers.rs). */
export type ProviderProfile = {
  uuid: string;
  template_id: string;
  name: string;
  protocol: ProviderProtocol;
  endpoint: string;
  model: string | null;
  enabled: boolean;
  sort_order: number;
  is_local: boolean;
  needs_key: boolean;
  secret_ref: string;
  capabilities: ProviderCapabilities;
  status: ProviderStatus;
  /** Optimistic-lock version (R2-E); echoed back as `expected_version`. */
  version: number;
};

/** `deny_unknown_fields` patch — no unknown keys. */
export type ProviderPatch = {
  name?: string;
  endpoint?: string;
  model?: string | null;
  enabled?: boolean;
  sort_order?: number;
  expected_version: number;
  protocol?: ProviderProtocol;
};

/** ProviderProfile joined with the `hasKey` bit from key_status. */
export type ProviderProfileFE = ProviderProfile & { hasKey: boolean };

/** Client session mirror of the active selection. */
export type ActiveSelection = {
  primaryUuid: string | null;
  parallelUuids: string[];
  fallbackUuid: string | null;
};

/** Mirror of db::providers::ActiveSelection (B3 wire shape). */
export type ActiveSelectionFE = {
  primary: string | null;
  parallel: string[];
  fallback: string | null;
};

export type ModelInfo = { id: string; label: string };

export type ConnectionResult = {
  ok: boolean;
  message: string;
  latency_ms?: number | null;
};

export type SetActiveResult =
  | { outcome: "written" }
  | { outcome: "needs_consent"; actual_scope: string };

export type ProviderCommandError =
  | { error: "stale_scope"; actual_scope: string }
  | { error: "stale_version"; actual_version: number }
  | { error: "db"; message: string }
  | { error: "validation"; message: string };

export type BalanceResultFE =
  | { kind: "unsupported" }
  | { kind: "ok"; balance: string; quota?: string | null }
  | { kind: "error"; message: string };

// --- Presets -------------------------------------------------------------

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

export type CatalogPresetDto = {
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

/** `name` null → render the localized Ollama label. */
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

// --- View state ----------------------------------------------------------

export type ToastVariant = "info" | "success" | "warning" | "destructive";
export type ToastEntry = { id: number; variant: ToastVariant; message: string };

export type ConsentRecipient = { name: string; localLabel: string };

/** Detail-panel state for the selected provider (pure-view contract). */
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

export type RoleState =
  | { kind: "primary" }
  | { kind: "parallel"; index: number }
  | { kind: "fallback" }
  | { kind: "none" };
