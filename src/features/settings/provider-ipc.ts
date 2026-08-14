/**
 * Typed `invoke` wrappers for the S2a/P1 provider + keystore IPC commands.
 *
 * Tauri v2's `tauri::command` macro camelCases snake_case Rust parameter names
 * on the JS side (e.g. `template_id: String` → `{ templateId }`). The wrapper
 * argument names below match the camelCased JS keys Tauri expects.
 *
 * `ProviderCommandError` (stale_scope) is NOT swallowed here — the rejection
 * propagates as-is so the component can catch and narrow on `e?.error ===
 * "stale_scope"`. Document this at the call site.
 */

import { invoke } from "@tauri-apps/api/core";
import type {
  ProviderProfile,
  ProviderProfileFE,
  ProviderPatch,
  ModelInfo,
  ConnectionResult,
  SetActiveResult,
  ActiveSelectionFE,
} from "./provider-types";

// Re-export so components import the error type from one place.
export type { ProviderCommandError } from "./provider-types";

/**
 * Loads providers and joins each with its `hasKey` bit from `key_status`.
 * The two reads run in parallel (they are independent read-only commands).
 */
export async function loadProviders(): Promise<ProviderProfileFE[]> {
  const [profiles, keyMap] = await Promise.all([
    invoke<ProviderProfile[]>("provider_list"),
    invoke<Record<string, boolean>>("key_status"),
  ]);
  return profiles.map((p) => ({ ...p, hasKey: !!keyMap[p.secret_ref] }));
}

/** Create a provider from a preset template id. `model` defaults to null. */
export const providerCreate = (
  templateId: string,
  name: string,
  endpoint: string,
  model?: string,
): Promise<ProviderProfile> =>
  invoke<ProviderProfile>("provider_create", {
    templateId,
    name,
    endpoint,
    model: model ?? null,
  });

/** Apply a partial patch (`#[serde(deny_unknown_fields)]` — no unknown keys). */
export const providerUpdate = (
  uuid: string,
  patch: ProviderPatch,
): Promise<ProviderProfile> =>
  invoke<ProviderProfile>("provider_update", { uuid, patch });

/** Duplicate a provider (new UUID, new secret_ref, keyless). */
export const providerDuplicate = (uuid: string): Promise<ProviderProfile> =>
  invoke<ProviderProfile>("provider_duplicate", { uuid });

/** Delete a provider (3-step tombstone; backend returns `()`). */
export const providerDelete = (uuid: string): Promise<void> =>
  invoke<void>("provider_delete", { uuid });

/** Reorder — `uuids` must be exactly the active UUIDs. */
export const providerReorder = (uuids: string[]): Promise<void> =>
  invoke<void>("provider_reorder", { uuids });

/** Toggle enabled. */
export const providerToggle = (uuid: string, enabled: boolean): Promise<void> =>
  invoke<void>("provider_toggle", { uuid, enabled });

export type BalanceResultFE =
  | { kind: "unsupported" }
  | { kind: "ok"; balance: string; quota?: string | null }
  | { kind: "error"; message: string };

export const providerGetBalance = (uuid: string): Promise<BalanceResultFE> =>
  invoke<BalanceResultFE>("provider_get_balance", { uuid });

/** Set the API key. Rejects if the provider is not `status="active"`. */
export const providerSetKey = (uuid: string, key: string): Promise<void> =>
  invoke<void>("provider_set_key", { uuid, key });

/**
 * Set the active selection. Returns `{ outcome: "written" }` or
 * `{ outcome: "needs_consent", actual_scope }` (the latter only when parallel
 * is non-empty and the stored consent scope doesn't match).
 */
export const providerSetActive = (
  primary: string,
  parallel: string[],
  fallback: string | null,
): Promise<SetActiveResult> =>
  invoke<SetActiveResult>("provider_set_active", { primary, parallel, fallback });

/**
 * Confirm consent for a parallel selection and write it atomically. Returns the
 * bumped consent version (i64). Rejects with `ProviderCommandError`
 * (`{ error: "stale_scope", actual_scope }` etc.) — the rejection propagates;
 * the caller narrows on `e?.error`.
 */
export const providerConfirmAndSetActive = (
  primary: string,
  parallel: string[],
  fallback: string | null,
  expectedScope: string,
): Promise<number> =>
  invoke<number>("provider_confirm_and_set_active", {
    primary,
    parallel,
    fallback,
    expectedScope,
  });

/** Local list plus same-origin HTTP GET /models. */
export const providerGetModels = (uuid: string): Promise<ModelInfo[]> =>
  invoke<ModelInfo[]>("provider_get_models", { uuid });

/** Best-effort reachability probe. */
export const providerTestConnection = (uuid: string): Promise<ConnectionResult> =>
  invoke<ConnectionResult>("provider_test_connection", { uuid });

// --- Keystore (Surface 06) ---------------------------------------------

/** `""` = healthy/first-run; non-empty = fail-closed reason. */
export const keystoreHealth = (): Promise<string> =>
  invoke<string>("keystore_health");

/** Returns the archived path. */
export const archiveKeystore = (): Promise<string> =>
  invoke<string>("archive_keystore");

/** Returns the archived path, or null if nothing was archived. */
export const resetKeystore = (): Promise<string | null> =>
  invoke<string | null>("reset_keystore");

/** Key presence map keyed by `secret_ref`. */
export const keyStatus = (): Promise<Record<string, boolean>> =>
  invoke<Record<string, boolean>>("key_status");

/** B3: read the active (primary/parallel/fallback) selection. */
export const providerGetActiveSelection = (): Promise<ActiveSelectionFE> =>
  invoke<ActiveSelectionFE>("provider_get_active_selection");
