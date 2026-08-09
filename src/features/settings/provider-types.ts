/**
 * TS mirrors of the S2a/P1 provider IPC wire shapes.
 *
 * These are DECODE-ONLY types — they mirror the backend structs in
 * `src-tauri/src/db/providers.rs` and `src-tauri/src/lib.rs` exactly. Do NOT
 * redefine the contract here; if the backend changes, update this mirror.
 *
 * Tauri v2 serializes snake_case Rust fields as snake_case JSON keys (the
 * `#[serde(rename_all = "snake_case")]` on the enums is explicit), so the TS
 * field names match the Rust field names verbatim. Command *parameter* names,
 * however, are camelCased by Tauri's `tauri::command` macro on the JS side
 * (see `provider-ipc.ts` for the camelCase wrappers).
 */

// --- Enums (snake_case on the wire) -------------------------------------

/** Provider wire protocol. Matches `Protocol` enum in `db/providers.rs`. */
export type ProviderProtocol =
  | "openai_chat"
  | "anthropic"
  | "gemini"
  | "google_translate"
  | "custom_http";

/** Row lifecycle status. Matches `ProviderStatus` enum. */
export type ProviderStatus = "active" | "deleting" | "deleted";

// --- ProviderProfile + patch -------------------------------------------

export type ProviderCapabilities = {
  balance: boolean;
  quota: boolean;
  model_list: boolean;
};

/**
 * Faithful mirror of `ProviderProfile` (`db/providers.rs`). Field names and
 * order match the serialized JSON exactly.
 */
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
};

/**
 * `#[serde(deny_unknown_fields)]` patch — do not send unknown keys.
 * All fields optional; only the supplied keys are applied.
 */
export type ProviderPatch = {
  name?: string;
  endpoint?: string;
  model?: string | null;
  enabled?: boolean;
  sort_order?: number;
};

// --- Frontend-augmented profile -----------------------------------------

/**
 * `ProviderProfile` joined with the `hasKey` bit from `key_status`.
 * This is the shape the Provider Center UI renders against.
 */
export type ProviderProfileFE = ProviderProfile & {
  hasKey: boolean;
};

// --- Active selection (client session mirror) --------------------------

/**
 * Client-side mirror of the active selection. NOTE: there is NO backend
 * read-IPC exposing this in R3a, so the client tracks it for the session only.
 * Cold-load renders all roles as "none" (Known R3a limitation #1).
 */
export type ActiveSelection = {
  primaryUuid: string | null;
  parallelUuids: string[];
  fallbackUuid: string | null;
};

// --- Model info + connection test --------------------------------------

export type ModelInfo = {
  id: string;
  label: string;
};

export type ConnectionResult = {
  ok: boolean;
  message: string;
  /** Round-trip latency of a reachable probe (C3c). `null`/absent on failure
   *  arms (no probe ran). `Option<u32>` on the Rust side; serialized only when
   *  `Some` via `#[serde(skip_serializing_if = "Option::is_none")]`. */
  latency_ms?: number | null;
};

// --- set_active result + error -----------------------------------------

/**
 * `provider_set_active` tagged result. Wire shape:
 * - `{ outcome: "written" }`
 * - `{ outcome: "needs_consent", actual_scope: "..." }`
 */
export type SetActiveResult =
  | { outcome: "written" }
  | { outcome: "needs_consent"; actual_scope: string };

/**
 * `provider_confirm_and_set_active` structured error. Wire shape (tagged on
 * `error`): `{ error: "stale_scope", actual_scope }` | `{ error: "db", message }`
 * | `{ error: "validation", message }`.
 *
 * The IPC wrapper does NOT swallow this rejection — the component catches and
 * narrows on `e?.error === "stale_scope"`.
 */
export type ProviderCommandError =
  | { error: "stale_scope"; actual_scope: string }
  | { error: "db"; message: string }
  | { error: "validation"; message: string };

/** Mirror of `db::providers::ActiveSelection` (B3). */
export type ActiveSelectionFE = {
  primary: string | null;
  parallel: string[];
  fallback: string | null;
};
