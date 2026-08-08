/**
 * Parity tests for the Settings copy dictionary (Surface 05 + 06).
 *
 * Guards against:
 *  - a key landing in one locale but not the other
 *  - placeholder token drift ({name}/{reason}/{latency}/{message}) across locales
 *  - a design-system copy key going missing
 */
import { describe, it, expect } from "vitest";
import { SETTINGS_COPY, type SettingsCopy } from "./copy";

/** Recursively collect dotted key paths whose leaves are strings. */
function leafPaths(obj: unknown, prefix = ""): string[] {
  if (typeof obj === "string") return [prefix];
  if (obj === null || typeof obj !== "object") return [];
  const out: string[] = [];
  for (const [k, v] of Object.entries(obj as Record<string, unknown>)) {
    const path = prefix ? `${prefix}.${k}` : k;
    out.push(...leafPaths(v, path));
  }
  return out;
}

describe("settings copy parity", () => {
  it("every SettingsCopy key is present in both zh and en", () => {
    const zhKeys = leafPaths(SETTINGS_COPY.zh).sort();
    const enKeys = leafPaths(SETTINGS_COPY.en).sort();
    expect(enKeys).toEqual(zhKeys);
    // Sanity: the dictionary is non-trivial.
    expect(zhKeys.length).toBeGreaterThan(40);
  });

  it("SettingsCopy type accepts the structure", () => {
    const m: SettingsCopy = SETTINGS_COPY.zh;
    expect(typeof m.window.title).toBe("string");
  });

  it("placeholder tokens ({name}/{reason}/{latency}/{message}) match across locales", () => {
    const zhStrs = leafPaths(SETTINGS_COPY.zh)
      .map((p) => ({ p, v: readPath(SETTINGS_COPY.zh, p) }))
      .filter((x) => typeof x.v === "string");
    for (const { p, v } of zhStrs) {
      const tokens = (v as string).match(/\{[a-z]+\}/g) ?? [];
      for (const tok of tokens) {
        const enVal = readPath(SETTINGS_COPY.en, p);
        if (typeof enVal !== "string") continue;
        expect(enVal, `en ${p} missing token ${tok}`).toContain(tok);
      }
    }
  });

  it("Surface 05 design copy keys are present", () => {
    // Spot-check a representative set from 05-provider-center.md copy tables.
    expect(typeof SETTINGS_COPY.en.provider.empty.title).toBe("string");
    expect(typeof SETTINGS_COPY.en.provider.saveKey).toBe("string");
    expect(typeof SETTINGS_COPY.en.provider.consent.title).toBe("string");
    expect(typeof SETTINGS_COPY.en.provider.role.primary).toBe("string");
    expect(typeof SETTINGS_COPY.en.provider.balance.unsupportedNote).toBe("string");
    expect(typeof SETTINGS_COPY.en.provider.endpoint.label).toBe("string");
  });

  it("Surface 06 design copy keys are present", () => {
    expect(typeof SETTINGS_COPY.en.keystore.title).toBe("string");
    expect(typeof SETTINGS_COPY.en.keystore.resetConfirmTitle).toBe("string");
    expect(typeof SETTINGS_COPY.en.keystore.archivedTitle).toBe("string");
  });

  it("endpoint error code map covers all domain codes in both locales", () => {
    const codes = ["endpoint-required", "endpoint-invalid-url", "endpoint-must-https"] as const;
    for (const code of codes) {
      expect(typeof SETTINGS_COPY.en.provider.endpoint.errors[code]).toBe("string");
      expect(typeof SETTINGS_COPY.zh.provider.endpoint.errors[code]).toBe("string");
    }
  });

  it("selection error code map covers all domain codes in both locales", () => {
    const codes = [
      "parallel-duplicate",
      "parallel-contains-primary",
      "role-overlap",
      "disabled-in-slot",
      "fallback-not-traditional",
      "fallback-overlaps",
    ] as const;
    for (const code of codes) {
      expect(typeof SETTINGS_COPY.en.provider.selectionErrors[code]).toBe("string");
      expect(typeof SETTINGS_COPY.zh.provider.selectionErrors[code]).toBe("string");
    }
  });
});

/** Read a dotted path from a nested object (mirrors leafPaths serialization). */
function readPath(obj: unknown, path: string): unknown {
  return path.split(".").reduce<unknown>((acc, key) => {
    if (acc && typeof acc === "object") {
      return (acc as Record<string, unknown>)[key];
    }
    return undefined;
  }, obj);
}
