import { describe, it, expect } from "vitest";
import {
  resolveConsentKey,
  buildConsentScope,
  consentScopeKey,
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
  mkProvider({ uuid: "c", template: "google", endpoint: "https://c.example.com" }),
];

function scopeKey(sel: ActiveSelection, provs: MockProvider[]): string {
  return consentScopeKey(buildConsentScope(sel, provs));
}

describe("resolveConsentKey — production consent transition", () => {
  it("null consent stays null when scope unchanged (no auto-mint)", () => {
    const sel = { primaryUuid: "a", parallelUuids: ["b"], fallbackUuid: "c" };
    const key = scopeKey(sel, providers);
    // Profile save: previous=null, old=new=key, no approved → should stay null
    const result = resolveConsentKey(null, key, key);
    expect(result).toBeNull();
  });

  it("approved consent preserved when scope unchanged", () => {
    const sel = { primaryUuid: "a", parallelUuids: ["b"], fallbackUuid: "c" };
    const key = scopeKey(sel, providers);
    // Previous was approved for this scope, scope unchanged → preserve
    const result = resolveConsentKey(key, key, key);
    expect(result).toBe(key);
  });

  it("approved consent invalidated when recipient removed", () => {
    const selAB = { primaryUuid: "a", parallelUuids: ["b"], fallbackUuid: "c" };
    const keyAB = scopeKey(selAB, providers);
    const selA = { primaryUuid: "a", parallelUuids: [], fallbackUuid: "c" };
    const keyA = scopeKey(selA, providers);
    // handleRemoveParallel: previous=keyAB, old=keyAB, new=keyA → null
    const result = resolveConsentKey(keyAB, keyAB, keyA);
    expect(result).toBeNull();
  });

  it("approved consent invalidated when endpoint origin changes", () => {
    const sel = { primaryUuid: "a", parallelUuids: [], fallbackUuid: null };
    const key1 = scopeKey(sel, providers);
    const changed = providers.map((p) =>
      p.uuid === "a" ? { ...p, endpoint: "https://different.example.com" } : p,
    );
    const key2 = scopeKey(sel, changed);
    // Profile save: previous=key1, old=key1, new=key2 → null
    const result = resolveConsentKey(key1, key1, key2);
    expect(result).toBeNull();
  });

  it("Consent Confirm creates approved key only if it matches new scope", () => {
    const sel = { primaryUuid: "a", parallelUuids: ["b"], fallbackUuid: null };
    const key = scopeKey(sel, providers);
    // Approved key matches new scope → use it
    expect(resolveConsentKey(null, "different", key, key)).toBe(key);
    // Approved key does NOT match new scope → null (reject arbitrary key)
    expect(resolveConsentKey(null, "different", key, "arbitrary")).toBeNull();
  });

  it("null consent with scope change stays null (not auto-minted)", () => {
    const sel1 = { primaryUuid: "a", parallelUuids: ["b"], fallbackUuid: null };
    const key1 = scopeKey(sel1, providers);
    const sel2 = { primaryUuid: "a", parallelUuids: [], fallbackUuid: null };
    const key2 = scopeKey(sel2, providers);
    // previous=null, old=key1, new=key2 → null
    expect(resolveConsentKey(null, key1, key2)).toBeNull();
  });
});
