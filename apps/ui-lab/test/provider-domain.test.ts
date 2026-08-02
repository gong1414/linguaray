import { describe, it, expect } from "vitest";
import {
  validateActiveSelection,
  buildConsentScope,
  consentScopeKey,
  isConsentValid,
  validateEndpoint,
  normalizeOrigin,
  TRADITIONAL_TEMPLATES,
  type MockProvider,
  type ActiveSelection,
} from "../src/pages/provider-domain";

const mkProvider = (over: Partial<MockProvider> = {}): MockProvider => ({
  uuid: "p1",
  template: "openai",
  name: "Test",
  endpoint: "https://api.openai.com/v1",
  model: "gpt-4o",
  enabled: true,
  isLocal: false,
  hasKey: true,
  status: "active",
  sortOrder: 0,
  ...over,
});

const baseProviders: MockProvider[] = [
  mkProvider({ uuid: "a", template: "openai", endpoint: "https://a.example.com" }),
  mkProvider({ uuid: "b", template: "deepseek", endpoint: "https://b.example.com" }),
  mkProvider({ uuid: "c", template: "google", endpoint: "https://c.example.com" }), // traditional
  mkProvider({ uuid: "d", template: "deepl", endpoint: "https://d.example.com" }), // traditional
];

// --- ActiveSelection validator -------------------------------------------

describe("validateActiveSelection", () => {
  it("accepts a valid selection", () => {
    const sel: ActiveSelection = { primaryUuid: "a", parallelUuids: ["b"], fallbackUuid: "c" };
    expect(validateActiveSelection(sel, baseProviders).ok).toBe(true);
  });

  it("rejects disabled provider in primary slot", () => {
    const providers = [mkProvider({ uuid: "a", enabled: false })];
    const sel: ActiveSelection = { primaryUuid: "a", parallelUuids: [], fallbackUuid: null };
    const r = validateActiveSelection(sel, providers);
    expect(r.ok).toBe(false);
  });

  it("rejects parallel containing primary", () => {
    const sel: ActiveSelection = { primaryUuid: "a", parallelUuids: ["a", "b"], fallbackUuid: null };
    const r = validateActiveSelection(sel, baseProviders);
    expect(r.ok).toBe(false);
  });

  it("rejects parallel duplicate", () => {
    const sel: ActiveSelection = { primaryUuid: "a", parallelUuids: ["b", "b"], fallbackUuid: null };
    const r = validateActiveSelection(sel, baseProviders);
    expect(r.ok).toBe(false);
  });

  it("rejects non-traditional fallback", () => {
    const sel: ActiveSelection = { primaryUuid: "a", parallelUuids: [], fallbackUuid: "b" };
    // b = deepseek (AI, not traditional)
    const r = validateActiveSelection(sel, baseProviders);
    expect(r.ok).toBe(false);
  });

  it("rejects fallback overlapping primary", () => {
    const sel: ActiveSelection = { primaryUuid: "c", parallelUuids: [], fallbackUuid: "c" };
    const r = validateActiveSelection(sel, baseProviders);
    expect(r.ok).toBe(false);
  });

  it("rejects fallback overlapping parallel", () => {
    const sel: ActiveSelection = { primaryUuid: "a", parallelUuids: ["c"], fallbackUuid: "c" };
    const r = validateActiveSelection(sel, baseProviders);
    expect(r.ok).toBe(false);
  });

  it("accepts traditional fallback (google, deepl)", () => {
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

// --- Consent scope --------------------------------------------------------

describe("consent scope", () => {
  it("includes primary + parallel, excludes fallback", () => {
    const sel: ActiveSelection = { primaryUuid: "a", parallelUuids: ["b"], fallbackUuid: "c" };
    const scope = buildConsentScope(sel, baseProviders);
    const uuids = scope.recipients.map((r) => r.providerUuid);
    expect(uuids).toContain("a");
    expect(uuids).toContain("b");
    expect(uuids).not.toContain("c"); // fallback excluded
  });

  it("sorts recipients by UUID (stable, order-independent)", () => {
    const sel: ActiveSelection = { primaryUuid: "c", parallelUuids: ["a", "b"], fallbackUuid: null };
    const scope = buildConsentScope(sel, baseProviders);
    const uuids = scope.recipients.map((r) => r.providerUuid);
    expect(uuids).toEqual(["a", "b", "c"]); // sorted, not c-a-b
  });

  it("endpoint-origin change invalidates consent even if UUID unchanged", () => {
    const sel: ActiveSelection = { primaryUuid: "a", parallelUuids: [], fallbackUuid: null };
    const key1 = consentScopeKey(buildConsentScope(sel, baseProviders));
    // Same UUID, different endpoint origin
    const changedProviders = baseProviders.map((p) =>
      p.uuid === "a" ? { ...p, endpoint: "https://different.example.com" } : p,
    );
    const key2 = consentScopeKey(buildConsentScope(sel, changedProviders));
    expect(key1).not.toBe(key2);
  });

  it("sort-only change does NOT invalidate consent", () => {
    const sel1: ActiveSelection = { primaryUuid: "a", parallelUuids: ["b"], fallbackUuid: null };
    const sel2: ActiveSelection = { primaryUuid: "b", parallelUuids: ["a"], fallbackUuid: null };
    // Different order but same recipient set {a, b}
    const key1 = consentScopeKey(buildConsentScope(sel1, baseProviders));
    const key2 = consentScopeKey(buildConsentScope(sel2, baseProviders));
    expect(key1).toBe(key2);
  });

  it("isConsentValid returns false when no stored key", () => {
    const sel: ActiveSelection = { primaryUuid: "a", parallelUuids: [], fallbackUuid: null };
    expect(isConsentValid(sel, baseProviders, null)).toBe(false);
  });

  it("isConsentValid returns true when key matches", () => {
    const sel: ActiveSelection = { primaryUuid: "a", parallelUuids: ["b"], fallbackUuid: null };
    const key = consentScopeKey(buildConsentScope(sel, baseProviders));
    expect(isConsentValid(sel, baseProviders, key)).toBe(true);
  });
});

// --- Endpoint validator ---------------------------------------------------

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

  it("rejects HTTP non-loopback (stable code)", () => {
    const r = validateEndpoint("http://api.openai.com/v1");
    expect(r.ok).toBe(false);
    expect(r.ok === false && r.code).toBe("endpoint-must-https");
  });

  it("rejects localhost.evil.com (not exact loopback)", () => {
    const r = validateEndpoint("http://localhost.evil.com/v1");
    expect(r.ok).toBe(false);
    expect(r.ok === false && r.code).toBe("endpoint-must-https");
  });

  it("rejects empty (stable code)", () => {
    const r = validateEndpoint("");
    expect(r.ok).toBe(false);
    expect(r.ok === false && r.code).toBe("endpoint-required");
  });

  it("rejects no-scheme (stable code)", () => {
    const r = validateEndpoint("api.openai.com/v1");
    expect(r.ok).toBe(false);
    expect(r.ok === false && r.code).toBe("endpoint-invalid-url");
  });

  it("returns a code, NEVER a display string (i18n boundary)", () => {
    // The result must carry a stable code, not a localized message —
    // so the domain layer never leaks English into a Chinese UI.
    const r = validateEndpoint("ftp://bad.example.com");
    expect(r.ok).toBe(false);
    if (!r.ok) {
      expect(typeof r.code).toBe("string");
      // code is a stable identifier, not an English sentence
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

// --- i18n error-code coverage ---------------------------------------------
// The domain layer returns stable codes; the UI layer must map every code to
// a localized string. This guards against a new code landing in the domain
// without a matching dictionary entry (which would render "undefined").

describe("error code → i18n coverage", () => {
  it("every endpoint error code has a zh + en message", async () => {
    const { strings } = await import("../src/i18n");
    const codes = ["endpoint-required", "endpoint-invalid-url", "endpoint-must-https"] as const;
    for (const locale of ["en", "zh"] as const) {
      for (const code of codes) {
        const msg = strings[locale].provider.endpointErrors[code];
        expect(typeof msg).toBe("string");
        expect(msg.length).toBeGreaterThan(0);
      }
    }
  });

  it("every selection error code has a zh + en message", async () => {
    const { strings } = await import("../src/i18n");
    const codes = [
      "parallel-duplicate",
      "parallel-contains-primary",
      "role-overlap",
      "disabled-in-slot",
      "fallback-not-traditional",
      "fallback-overlaps",
    ] as const;
    for (const locale of ["en", "zh"] as const) {
      for (const code of codes) {
        const msg = strings[locale].provider.selectionErrors[code];
        expect(typeof msg).toBe("string");
        expect(msg.length).toBeGreaterThan(0);
      }
    }
  });
});
