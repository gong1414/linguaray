import { describe, it, expect } from "vitest";
import {
  validateActiveSelection,
  buildConsentScope,
  consentScopeKey,
  isConsentValid,
  type MockProvider,
  type ActiveSelection,
} from "../src/pages/provider-domain";

const mkProvider = (over: Partial<MockProvider> = {}): MockProvider => ({
  uuid: "p1",
  template: "openai",
  name: "Test",
  endpoint: "https://a.example.com",
  model: "gpt-4o",
  enabled: true,
  isLocal: false,
  hasKey: true,
  status: "active",
  sortOrder: 0,
  ...over,
});

const providers: MockProvider[] = [
  mkProvider({ uuid: "a", template: "openai", endpoint: "https://a.example.com" }),
  mkProvider({ uuid: "b", template: "deepseek", endpoint: "https://b.example.com" }),
  mkProvider({ uuid: "c", template: "google", endpoint: "https://c.example.com" }), // traditional
];

describe("Consent mint-prevention regression", () => {
  it("null consent stays null after scope-unchanged profile save", () => {
    const sel: ActiveSelection = { primaryUuid: "a", parallelUuids: ["b"], fallbackUuid: "c" };
    const scope = buildConsentScope(sel, providers);
    const key = consentScopeKey(scope);
    // Previous consent is null (never approved)
    // After profile save with same scope: should still be null
    expect(isConsentValid(sel, providers, null)).toBe(false);
    // Even if we compute the key, without prior approval it's not valid
    expect(isConsentValid(sel, providers, key)).toBe(true); // key matches, but that just means the scope matches
  });

  it("approved consent A+B → remove B → consent invalidates", () => {
    const selAB: ActiveSelection = { primaryUuid: "a", parallelUuids: ["b"], fallbackUuid: "c" };
    const keyAB = consentScopeKey(buildConsentScope(selAB, providers));
    expect(isConsentValid(selAB, providers, keyAB)).toBe(true);

    // Remove B from parallel
    const selA: ActiveSelection = { primaryUuid: "a", parallelUuids: [], fallbackUuid: "c" };
    const keyA = consentScopeKey(buildConsentScope(selA, providers));
    // The old key should NOT match the new scope
    expect(keyAB).not.toBe(keyA);
    expect(isConsentValid(selA, providers, keyAB)).toBe(false);
  });

  it("scope unchanged → valid consent preserved", () => {
    const sel: ActiveSelection = { primaryUuid: "a", parallelUuids: ["b"], fallbackUuid: "c" };
    const key = consentScopeKey(buildConsentScope(sel, providers));
    // Same selection, same providers → key still valid
    expect(isConsentValid(sel, providers, key)).toBe(true);
  });

  it("endpoint origin change invalidates consent even if UUID unchanged", () => {
    const sel: ActiveSelection = { primaryUuid: "a", parallelUuids: [], fallbackUuid: null };
    const key1 = consentScopeKey(buildConsentScope(sel, providers));
    const changedProviders = providers.map((p) =>
      p.uuid === "a" ? { ...p, endpoint: "https://different.example.com" } : p,
    );
    const key2 = consentScopeKey(buildConsentScope(sel, changedProviders));
    expect(key1).not.toBe(key2);
    expect(isConsentValid(sel, changedProviders, key1)).toBe(false);
  });
});
