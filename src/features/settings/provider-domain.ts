/**
 * Provider Center domain logic — pure functions.
 *
 * Vendored from `apps/ui-lab/src/pages/provider-domain.ts`, adapted to operate
 * on the production `ProviderProfileFE` shape (the lab's `MockProvider.template`
 * → `template_id`, `MockProvider.sortOrder` → `sort_order`). These mirror the
 * frozen S0 constraints the Settings UI must enforce BEFORE calling the
 * backend. They are unit-testable in isolation and produce STABLE ERROR CODES
 * (never display strings) so the i18n layer is the only place text lives.
 *
 * NOT the production Rust implementation — that lives in `src-tauri`. These are
 * the JS-side validation rules.
 */

import type { ActiveSelection, ProviderProfileFE } from "./provider-types";

// --- Traditional templates ----------------------------------------------

/**
 * Template-ids allowed in the `fallback` slot (traditional MT engines only).
 * Source of truth: `TRADITIONAL_TEMPLATES` in `src-tauri/src/db/providers.rs`.
 * Stored as `Set<string>` since template_ids are wire strings.
 */
export const TRADITIONAL_TEMPLATES: ReadonlySet<string> = new Set([
  "google",
  "deepl",
  "microsoft",
  "baidu",
  "youdao",
  "tencent",
]);

// --- ActiveSelection validator -----------------------------------------

export type ActiveSelectionErrorCode =
  | "parallel-duplicate"
  | "parallel-contains-primary"
  | "role-overlap"
  | "disabled-in-slot"
  | "fallback-not-traditional"
  | "fallback-overlaps";

export type ActiveSelectionError = {
  code: ActiveSelectionErrorCode;
  /** Stable, non-localized developer message — NEVER shown to users.
   *  UI must map `code` through i18n. Kept for debugging/tests only. */
  message: string;
};

export type ValidationResult =
  | { ok: true }
  | { ok: false; errors: ActiveSelectionError[] };

/**
 * Validates an ActiveSelection against the frozen constraints. REJECTS illegal
 * input (returns errors) — does NOT silently dedupe or rewrite order.
 *
 * Constraints:
 * - disabled/deleting/deleted providers cannot be in any slot
 * - parallel is ordered, deduped, excludes primary
 * - fallback must be an enabled traditional MT engine
 * - fallback must not overlap primary or parallel
 */
export function validateActiveSelection(
  selection: ActiveSelection,
  providers: ProviderProfileFE[],
): ValidationResult {
  const errors: ActiveSelectionError[] = [];
  const byUuid = new Map(providers.map((p) => [p.uuid, p]));

  const isCallable = (p: ProviderProfileFE | undefined): boolean =>
    !!p && p.enabled && p.status === "active";

  const primary = selection.primaryUuid
    ? byUuid.get(selection.primaryUuid)
    : undefined;
  if (selection.primaryUuid && !isCallable(primary)) {
    errors.push({
      code: "disabled-in-slot",
      message: "Primary provider is disabled or deleted",
    });
  }

  // parallel: no duplicates, no primary, all callable
  const seen = new Set<string>();
  for (const uuid of selection.parallelUuids) {
    if (uuid === selection.primaryUuid) {
      errors.push({
        code: "parallel-contains-primary",
        message: "Parallel list must not contain the primary provider",
      });
    }
    if (seen.has(uuid)) {
      errors.push({
        code: "parallel-duplicate",
        message: "Parallel list contains a duplicate provider",
      });
    }
    seen.add(uuid);
    if (!isCallable(byUuid.get(uuid))) {
      errors.push({
        code: "disabled-in-slot",
        message: "A parallel provider is disabled or deleted",
      });
    }
  }

  // fallback: traditional MT only, enabled, not overlapping
  if (selection.fallbackUuid) {
    const fb = byUuid.get(selection.fallbackUuid);
    if (!isCallable(fb)) {
      errors.push({
        code: "disabled-in-slot",
        message: "Fallback provider is disabled or deleted",
      });
    } else if (!TRADITIONAL_TEMPLATES.has(fb!.template_id)) {
      errors.push({
        code: "fallback-not-traditional",
        message: "Fallback must be a traditional MT engine",
      });
    }
    if (selection.fallbackUuid === selection.primaryUuid) {
      errors.push({
        code: "fallback-overlaps",
        message: "Fallback must not be the primary provider",
      });
    }
    if (selection.parallelUuids.includes(selection.fallbackUuid)) {
      errors.push({
        code: "fallback-overlaps",
        message: "Fallback must not be in the parallel list",
      });
    }
  }

  return errors.length === 0 ? { ok: true } : { ok: false, errors };
}

// --- Consent scope ------------------------------------------------------

/**
 * Frozen canonical consent scope (S0 multi-engine consent).
 *
 * recipients = primary + parallel, deduped, sorted by UUID.
 * fallback is EXCLUDED (not part of normal parallel send).
 * endpointOrigin change invalidates consent even if UUID is unchanged.
 *
 * The canonical key is a stable string — fixed field order, no Map/object
 * enumeration order dependence. Uses new URL(endpoint).origin normalization.
 */
export type ConsentRecipient = {
  providerUuid: string;
  endpointOrigin: string;
  isLocal: boolean;
};

export type ConsentScope = {
  version: 1;
  recipients: ConsentRecipient[];
};

export function buildConsentScope(
  selection: ActiveSelection,
  providers: ProviderProfileFE[],
): ConsentScope {
  const byUuid = new Map(providers.map((p) => [p.uuid, p]));
  const recipientUuids = [
    selection.primaryUuid,
    ...selection.parallelUuids,
  ].filter((u): u is string => u !== null);

  const recipients: ConsentRecipient[] = recipientUuids
    .map((uuid) => {
      const p = byUuid.get(uuid);
      if (!p) return null;
      return {
        providerUuid: uuid,
        endpointOrigin: normalizeOrigin(p.endpoint),
        isLocal: p.is_local,
      };
    })
    .filter((r): r is ConsentRecipient => r !== null);

  // Dedupe by UUID, sort by UUID (stable, no enumeration-order dependence).
  const deduped = new Map(recipients.map((r) => [r.providerUuid, r]));
  const sorted = [...deduped.values()].sort((a, b) =>
    a.providerUuid < b.providerUuid ? -1 : a.providerUuid > b.providerUuid ? 1 : 0,
  );

  return { version: 1, recipients: sorted };
}

/**
 * Canonical string key for a ConsentScope. Fixed field order serialization.
 * Two scopes with the same recipients (same UUIDs + origins) produce the same
 * key. Sort-order changes do NOT change the key (recipients are re-sorted).
 */
export function consentScopeKey(scope: ConsentScope): string {
  const parts = scope.recipients.map(
    (r) => `${r.providerUuid}|${r.endpointOrigin}|${r.isLocal}`,
  );
  return `v${scope.version}:{${parts.join(",")}}`;
}

/** Returns true if the stored consent key matches the current selection scope. */
export function isConsentValid(
  selection: ActiveSelection,
  providers: ProviderProfileFE[],
  storedKey: string | null,
): boolean {
  if (!storedKey) return false;
  const currentKey = consentScopeKey(buildConsentScope(selection, providers));
  return currentKey === storedKey;
}

/**
 * Pure consent transition function. Used to determine the next consent key
 * after a state change.
 *
 * Rules:
 * - If an approved key is provided AND it matches the new scope → use it.
 *   (Only Consent Confirm provides approvedKey.)
 * - If previous consent matched the OLD scope AND the new scope is identical
 *   → preserve (scope unchanged).
 * - Otherwise → null (invalidated). NEVER auto-mint.
 *
 * @param previous    The current consent key (may be null = never approved).
 * @param oldScopeKey The canonical key of the OLD provider/selection state.
 * @param newScopeKey The canonical key of the NEW provider/selection state.
 * @param approved    Optional approved key from Consent Confirm.
 * @returns The next consent key (null if invalidated/unapproved).
 */
export function resolveConsentKey(
  previous: string | null,
  oldScopeKey: string,
  newScopeKey: string,
  approved?: string,
): string | null {
  // Approved key: only valid if it matches the new scope exactly
  if (approved !== undefined) {
    return approved === newScopeKey ? approved : null;
  }
  // Preserve: previous was valid for the old scope, and scope hasn't changed
  if (previous !== null && previous === oldScopeKey && newScopeKey === oldScopeKey) {
    return previous;
  }
  // Otherwise: invalidate (null). Never auto-mint.
  return null;
}

// --- Endpoint validator -------------------------------------------------

/** Stable, non-localized error code for endpoint validation.
 *  UI maps this through i18n. NEVER render the raw code to users. */
export type EndpointErrorCode =
  | "endpoint-required"
  | "endpoint-invalid-url"
  | "endpoint-must-https";

export type EndpointValidationResult =
  | { ok: true }
  | { ok: false; code: EndpointErrorCode };

/**
 * Validates a provider endpoint. Global HTTPS; HTTP only for exact loopback
 * hosts (localhost, 127.0.0.1, [::1]). Rejects localhost.evil.com etc.
 *
 * Returns a STABLE ERROR CODE (not a display string) — the caller must map
 * `code` through the i18n dictionary before showing it to users.
 */
export function validateEndpoint(endpoint: string): EndpointValidationResult {
  const trimmed = endpoint.trim();
  if (!trimmed) return { ok: false, code: "endpoint-required" };

  let url: URL;
  try {
    url = new URL(trimmed);
  } catch {
    return { ok: false, code: "endpoint-invalid-url" };
  }

  const protocol = url.protocol;
  const host = url.hostname.toLowerCase();

  if (protocol === "https:") return { ok: true };
  if (protocol === "http:") {
    if (host === "localhost" || host === "127.0.0.1" || host === "[::1]") {
      return { ok: true };
    }
    return { ok: false, code: "endpoint-must-https" };
  }
  return { ok: false, code: "endpoint-must-https" };
}

/** Normalizes an endpoint to its origin (scheme + host + port). */
export function normalizeOrigin(endpoint: string): string {
  try {
    return new URL(endpoint).origin;
  } catch {
    return endpoint;
  }
}
