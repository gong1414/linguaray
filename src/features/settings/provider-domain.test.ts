/**
 * Unit tests for the vendored provider domain logic (ported from
 * `apps/ui-lab/test/provider-domain.test.ts`, adapted to operate on the
 * production `ProviderProfileFE` shape: `template` → `template_id`,
 * `sortOrder` → `sort_order`).
 */
import { describe, it, expect } from "vitest";
import {
  validateActiveSelection,
  buildConsentScope,
  consentScopeKey,
  resolveConsentKey,
  isConsentValid,
  validateEndpoint,
  normalizeOrigin,
  TRADITIONAL_TEMPLATES,
  type ConsentScope,
} from "./provider-domain";
import type { ProviderProfileFE, ActiveSelection } from "./provider-types";

const mkProvider = (over: Partial<ProviderProfileFE> = {}): ProviderProfileFE => ({
  uuid: "p1",
  template_id: "openai",
  name: "Test",
  endpoint: "https://api.openai.com/v1",
  model: "gpt-4o",
  enabled: true,
  is_local: false,
  hasKey: true,
  status: "active",
  sort_order: 0,
  // full ProviderProfile wire fields (defaults; not exercised by the domain fns)
  protocol: "openai_chat",
  needs_key: true,
  secret_ref: "provider/p1",
  capabilities: { balance: false, quota: false, model_list: false },
  ...over,
});

const baseProviders: ProviderProfileFE[] = [
  mkProvider({ uuid: "a", template_id: "openai", endpoint: "https://a.example.com" }),
  mkProvider({ uuid: "b", template_id: "deepseek", endpoint: "https://b.example.com" }),
  mkProvider({ uuid: "c", template_id: "google", endpoint: "https://c.example.com" }), // traditional
  mkProvider({ uuid: "d", template_id: "deepl", endpoint: "https://d.example.com" }), // traditional
];

// --- validateActiveSelection --------------------------------------------

describe("validateActiveSelection", () => {
  it("accepts a valid selection", () => {
    const sel: ActiveSelection = { primaryUuid: "a", parallelUuids: ["b"], fallbackUuid: "c" };
    expect(validateActiveSelection(sel, baseProviders).ok).toBe(true);
  });

  it("rejects disabled primary with disabled-in-slot", () => {
    const providers = [mkProvider({ uuid: "a", enabled: false })];
    const sel: ActiveSelection = { primaryUuid: "a", parallelUuids: [], fallbackUuid: null };
    const r = validateActiveSelection(sel, providers);
    expect(r.ok).toBe(false);
    if (!r.ok) {
      expect(r.errors.some((e) => e.code === "disabled-in-slot")).toBe(true);
    }
  });

  it("rejects parallel containing primary", () => {
    const sel: ActiveSelection = { primaryUuid: "a", parallelUuids: ["a", "b"], fallbackUuid: null };
    const r = validateActiveSelection(sel, baseProviders);
    expect(r.ok).toBe(false);
    if (!r.ok) {
      expect(r.errors.some((e) => e.code === "parallel-contains-primary")).toBe(true);
    }
  });

  it("rejects duplicate in parallel", () => {
    const sel: ActiveSelection = { primaryUuid: "a", parallelUuids: ["b", "b"], fallbackUuid: null };
    const r = validateActiveSelection(sel, baseProviders);
    expect(r.ok).toBe(false);
    if (!r.ok) {
      expect(r.errors.some((e) => e.code === "parallel-duplicate")).toBe(true);
    }
  });

  it("rejects non-traditional fallback with fallback-not-traditional", () => {
    const sel: ActiveSelection = { primaryUuid: "a", parallelUuids: [], fallbackUuid: "b" };
    // b = deepseek (AI, not traditional)
    const r = validateActiveSelection(sel, baseProviders);
    expect(r.ok).toBe(false);
    if (!r.ok) {
      expect(r.errors.some((e) => e.code === "fallback-not-traditional")).toBe(true);
    }
  });

  it("rejects fallback overlapping primary with fallback-overlaps", () => {
    const sel: ActiveSelection = { primaryUuid: "c", parallelUuids: [], fallbackUuid: "c" };
    const r = validateActiveSelection(sel, baseProviders);
    expect(r.ok).toBe(false);
    if (!r.ok) {
      expect(r.errors.some((e) => e.code === "fallback-overlaps")).toBe(true);
    }
  });

  it("rejects fallback overlapping parallel with fallback-overlaps", () => {
    const sel: ActiveSelection = { primaryUuid: "a", parallelUuids: ["c"], fallbackUuid: "c" };
    const r = validateActiveSelection(sel, baseProviders);
    expect(r.ok).toBe(false);
    if (!r.ok) {
      expect(r.errors.some((e) => e.code === "fallback-overlaps")).toBe(true);
    }
  });

  it("accepts traditional fallback (google, deepl membership)", () => {
    expect(TRADITIONAL_TEMPLATES.has("google")).toBe(true);
    expect(TRADITIONAL_TEMPLATES.has("deepl")).toBe(true);
    expect(TRADITIONAL_TEMPLATES.has("openai")).toBe(false);
  });

  it("rejects deleting provider in any slot", () => {
    const providers = [mkProvider({ uuid: "a", status: "deleting" })];
    const sel: ActiveSelection = { primaryUuid: "a", parallelUuids: [], fallbackUuid: null };
    expect(validateActiveSelection(sel, providers).ok).toBe(false);
  });
});

// --- Consent scope -------------------------------------------------------

describe("buildConsentScope", () => {
  it("dedupes + sorts recipients by uuid; excludes fallback", () => {
    const sel: ActiveSelection = { primaryUuid: "c", parallelUuids: ["a", "b"], fallbackUuid: "d" };
    const scope = buildConsentScope(sel, baseProviders);
    const uuids = scope.recipients.map((r) => r.providerUuid);
    expect(uuids).toEqual(["a", "b", "c"]); // sorted, deduped, fallback excluded
    expect(uuids).not.toContain("d");
  });

  it("endpoint-origin change invalidates consent even if UUID unchanged", () => {
    const sel: ActiveSelection = { primaryUuid: "a", parallelUuids: [], fallbackUuid: null };
    const key1 = consentScopeKey(buildConsentScope(sel, baseProviders));
    const changedProviders = baseProviders.map((p) =>
      p.uuid === "a" ? { ...p, endpoint: "https://different.example.com" } : p,
    );
    const key2 = consentScopeKey(buildConsentScope(sel, changedProviders));
    expect(key1).not.toBe(key2);
  });
});

describe("consentScopeKey", () => {
  it("stable across recipient reordering (same set, same key)", () => {
    const sel1: ActiveSelection = { primaryUuid: "a", parallelUuids: ["b"], fallbackUuid: null };
    const sel2: ActiveSelection = { primaryUuid: "b", parallelUuids: ["a"], fallbackUuid: null };
    const key1 = consentScopeKey(buildConsentScope(sel1, baseProviders));
    const key2 = consentScopeKey(buildConsentScope(sel2, baseProviders));
    expect(key1).toBe(key2);
  });
});

describe("resolveConsentKey", () => {
  const scopeA: ConsentScope = buildConsentScope(
    { primaryUuid: "a", parallelUuids: [], fallbackUuid: null },
    baseProviders,
  );
  const keyA = consentScopeKey(scopeA);

  it("approved key matching new scope is preserved", () => {
    const next = resolveConsentKey(null, keyA, keyA, keyA);
    expect(next).toBe(keyA);
  });

  it("approved key NOT matching new scope → null", () => {
    const next = resolveConsentKey(null, keyA, "v1:{other}", keyA);
    expect(next).toBeNull();
  });

  it("scope change without approval → null (never auto-mint)", () => {
    const next = resolveConsentKey(null, keyA, "v1:{different}");
    expect(next).toBeNull();
  });

  it("previous valid for old scope, unchanged → preserved", () => {
    const next = resolveConsentKey(keyA, keyA, keyA);
    expect(next).toBe(keyA);
  });
});

describe("isConsentValid", () => {
  it("false when no stored key", () => {
    const sel: ActiveSelection = { primaryUuid: "a", parallelUuids: [], fallbackUuid: null };
    expect(isConsentValid(sel, baseProviders, null)).toBe(false);
  });
  it("true when key matches current scope", () => {
    const sel: ActiveSelection = { primaryUuid: "a", parallelUuids: ["b"], fallbackUuid: null };
    const key = consentScopeKey(buildConsentScope(sel, baseProviders));
    expect(isConsentValid(sel, baseProviders, key)).toBe(true);
  });
});

// --- Endpoint validator -------------------------------------------------

describe("validateEndpoint", () => {
  it("accepts HTTPS", () => {
    expect(validateEndpoint("https://api.openai.com/v1").ok).toBe(true);
  });
  it("accepts HTTP localhost", () => {
    expect(validateEndpoint("http://localhost:11434/v1").ok).toBe(true);
  });
  it("accepts HTTP 127.0.0.1", () => {
    expect(validateEndpoint("http://127.0.0.1:8080").ok).toBe(true);
  });
  it("accepts HTTP [::1]", () => {
    expect(validateEndpoint("http://[::1]:8080").ok).toBe(true);
  });
  it("rejects HTTP non-loopback → endpoint-must-https", () => {
    const r = validateEndpoint("http://api.openai.com/v1");
    expect(r.ok).toBe(false);
    expect(r.ok === false && r.code).toBe("endpoint-must-https");
  });
  it("rejects localhost.evil.com (not exact loopback)", () => {
    const r = validateEndpoint("http://localhost.evil.com/v1");
    expect(r.ok === false && r.code).toBe("endpoint-must-https");
  });
  it("rejects empty → endpoint-required", () => {
    const r = validateEndpoint("");
    expect(r.ok === false && r.code).toBe("endpoint-required");
  });
  it("rejects garbage URL → endpoint-invalid-url", () => {
    const r = validateEndpoint("api.openai.com/v1");
    expect(r.ok === false && r.code).toBe("endpoint-invalid-url");
  });
  it("returns a stable code, NEVER a display string", () => {
    const r = validateEndpoint("ftp://bad.example.com");
    expect(r.ok).toBe(false);
    if (!r.ok) {
      expect(typeof r.code).toBe("string");
      expect(r.code).not.toMatch(/[A-Z][a-z]+\s/);
    }
  });
});

describe("normalizeOrigin", () => {
  it("extracts scheme + host + port", () => {
    expect(normalizeOrigin("https://api.openai.com/v1/chat")).toBe("https://api.openai.com");
    expect(normalizeOrigin("http://localhost:11434/v1")).toBe("http://localhost:11434");
  });
});
