import { describe, expect, it } from "vitest";
import { SHORTCUTS_COPY } from "./shortcuts-copy";
import { SHORTCUT_ACTIONS } from "./shortcut-types";

describe("Shortcuts copy", () => {
  it("keeps the same complete action-key set in English and Chinese", () => {
    for (const locale of ["en", "zh"] as const) {
      expect(Object.keys(SHORTCUTS_COPY[locale].actions).sort()).toEqual(
        [...SHORTCUT_ACTIONS].sort(),
      );
      for (const action of SHORTCUT_ACTIONS) {
        expect(SHORTCUTS_COPY[locale].actions[action].trim()).not.toBe("");
      }
    }
  });

  it("keeps required interpolation placeholders", () => {
    for (const locale of ["en", "zh"] as const) {
      expect(SHORTCUTS_COPY[locale].changeLabel).toContain("{action}");
      expect(SHORTCUTS_COPY[locale].conflictMessage).toContain("{action}");
      expect(SHORTCUTS_COPY[locale].clearLabel).toContain("{action}");
    }
  });
});
