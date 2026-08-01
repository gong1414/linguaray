/**
 * Provider Center domain logic — pure functions for the mock UI Lab.
 *
 * These mirror the frozen S0 constraints so the prototype exercises the real
 * product rules, not ad-hoc state. They are unit-testable in isolation.
 *
 * NOT the production Rust implementation — that lives in src-tauri. These are
 * the JS-side validation rules the Settings UI must enforce before calling
 * the backend.
 */

// --- Types ----------------------------------------------------------------

export type ProviderTemplate =
  | "openai"
  | "anthropic"
  | "gemini"
  | "deepseek"
  | "google"
  | "deepl"
  | "microsoft"
  | "baidu"
  | "youdao"
  | "tencent"
  | "ollama"
  | "custom";

/** Traditional MT engines (eligible for fallback per S0). */
export const TRADITIONAL_TEMPLATES: ReadonlySet<ProviderTemplate> = new Set([
  "google",
  "deepl",
  "microsoft",
  "baidu",
  "youdao",
  "tencent",
]);

export type ProviderStatus = "active" | "deleting" | "deleted";

export type MockProvider = {
  uuid: string;
  template: ProviderTemplate;
  name: string;
  endpoint: string;
  model: string | null;
  enabled: boolean;
  isLocal: boolean;
  hasKey: boolean;
  status: ProviderStatus;
  sortOrder: number;
};

export type ActiveSelection = {
  primaryUuid: string | null;
  parallelUuids: string[];
  fallbackUuid: string | null;
};

export type ActiveSelectionError = {
  code:
    | "parallel-duplicate"
    | "parallel-contains-primary"
    | "role-overlap"
    | "disabled-in-slot"
    | "fallback-not-traditional"
    | "fallback-overlaps";
  message: string;
};

export type ValidationResult =
  | { ok: true }
  | { ok: false; errors: ActiveSelectionError[] };

// --- ActiveSelection validator -------------------------------------------

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
  providers: MockProvider[],
): ValidationResult {
  const errors: ActiveSelectionError[] = [];
  const byUuid = new Map(providers.map((p) => [p.uuid, p]));

  const isCallable = (p: MockProvider | undefined): boolean =>
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
    } else if (!TRADITIONAL_TEMPLATES.has(fb!.template)) {
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

// --- Consent scope --------------------------------------------------------

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
  providers: MockProvider[],
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
        isLocal: p.isLocal,
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
  providers: MockProvider[],
  storedKey: string | null,
): boolean {
  if (!storedKey) return false;
  const currentKey = consentScopeKey(buildConsentScope(selection, providers));
  return currentKey === storedKey;
}

/**
 * Pure consent transition function. Used by commitProviderState and
 * handleSaveProfile to determine the next consent key after a state change.
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

// --- Endpoint validator ---------------------------------------------------

/**
 * Validates a provider endpoint. Global HTTPS; HTTP only for exact loopback
 * hosts (localhost, 127.0.0.1, [::1]). Rejects localhost.evil.com etc.
 */
export function validateEndpoint(endpoint: string): { ok: boolean; error?: string } {
  const trimmed = endpoint.trim();
  if (!trimmed) return { ok: false, error: "Endpoint is required" };

  let url: URL;
  try {
    url = new URL(trimmed);
  } catch {
    return { ok: false, error: "Invalid URL" };
  }

  const protocol = url.protocol;
  const host = url.hostname.toLowerCase();

  if (protocol === "https:") return { ok: true };
  if (protocol === "http:") {
    if (host === "localhost" || host === "127.0.0.1" || host === "[::1]") {
      return { ok: true };
    }
    return {
      ok: false,
      error: "Must be HTTPS (or localhost)",
    };
  }
  return { ok: false, error: "Must be HTTPS (or localhost)" };
}

/** Normalizes an endpoint to its origin (scheme + host + port). */
export function normalizeOrigin(endpoint: string): string {
  try {
    return new URL(endpoint).origin;
  } catch {
    return endpoint;
  }
}
