import { describe, it, expect } from "vitest";
import { COPY, type CopyMap } from "./copy";
import type { CopyKey } from "./types";

describe("copy map", () => {
  it("every CopyKey exists in zh and en", () => {
    const keys: CopyKey[] = [
      "selection.loading", "selection.error.network", "selection.error.config.key",
      "selection.error.config.auth", "selection.error.noSelection",
      "selection.error.noPermission", "selection.error.keystore",
      "selection.error.keystore.cta", "selection.error.offline",
      "selection.action.copy", "selection.action.copied", "selection.action.speak",
      "selection.action.stop", "selection.action.pin", "selection.action.unpin",
      "selection.action.favorite", "selection.action.favorited",
      "selection.action.retry", "selection.multi.title",
      "input.title", "input.placeholder", "input.action.translate",
      "input.action.clear", "input.result.label", "input.error.offline",
    ];
    for (const k of keys) {
      expect(COPY.zh[k], `zh missing ${k}`).toBeTypeOf("string");
      expect(COPY.en[k], `en missing ${k}`).toBeTypeOf("string");
    }
  });

  it("CopyMap type accepts the structure", () => {
    const m: CopyMap = COPY;
    expect(m.zh["selection.loading"]).toBe("翻译中…");
    expect(m.en["selection.loading"]).toBe("Translating…");
  });
});
