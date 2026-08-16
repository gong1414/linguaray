import { describe, expect, it } from "vitest";
import { Linter } from "eslint";
// Flat-config array from a JS config file — no type declarations exist by
// design; the `as never` cast below feeds it to Linter.verify.
// @ts-expect-error untyped JS config module
import eslintConfig from "../eslint.config.js";

/**
 * Negative tests for the architecture-boundary lint (eslint.config.js).
 * The dead "Legacy Solid" override block was removed in refactor P0.2 —
 * these cases prove the surviving rules still fire on violations, so the
 * cleanup cannot silently weaken the boundary.
 */
const linter = new Linter({ configType: "flat" });

function violations(filename: string, code: string) {
  return linter
    .verify(code, eslintConfig as never, { filename })
    .map((m) => m.ruleId)
    .filter(Boolean);
}

describe("eslint boundary rules still fire (P0.2 negative tests)", () => {
  it("view.tsx importing the bridge is rejected", () => {
    const rules = violations(
      "src/features/shell/view.tsx",
      'import { commands } from "../../bridge/invoke";\nexport {};\n',
    );
    expect(rules).toContain("no-restricted-imports");
  });

  it("controller.ts importing the bridge is allowed", () => {
    const rules = violations(
      "src/features/shell/controller.ts",
      'import { commands } from "../../bridge/invoke";\nexport {};\n',
    );
    expect(rules).not.toContain("no-restricted-imports");
  });

  it("non-bridge file importing @tauri-apps/api is rejected", () => {
    const rules = violations(
      "src/features/shell/controller.ts",
      'import { invoke } from "@tauri-apps/api/core";\nexport {};\n',
    );
    expect(rules).toContain("no-restricted-imports");
  });

  it("bridge file importing @tauri-apps/api is allowed", () => {
    const rules = violations(
      "src/bridge/invoke.ts",
      'import { invoke as rawInvoke } from "@tauri-apps/api/core";\nexport {};\n',
    );
    expect(rules).not.toContain("no-restricted-imports");
  });
});
