/**
 * Vitest + axe helper.
 *
 * axe-core runs against a real DOM (jsdom here). We wrap it so component tests
 * can assert WCAG violations with a single call.
 */
import "@testing-library/jest-dom";
import axe from "axe-core";

export type AxeOptions = {
  // Rules to disable when testing an isolated component that is incomplete
  // outside its real context (e.g. a <button> with no landmark ancestor).
  disableRules?: string[];
};

/**
 * Run axe against the current document body and return violations.
 * Fails the test (via expect) if any violations match the configured rules.
 */
export async function assertNoAxeViolations(
  options: AxeOptions = {},
): Promise<void> {
  // Required for SolidJS + jsdom: ensure axe sees the rendered tree.
  const results = await axe.run(document.body, {
    rules: Object.fromEntries(
      (options.disableRules ?? []).map((rule) => [rule, { enabled: false }]),
    ),
  });

  const violations = results.violations;
  if (violations.length > 0) {
    const formatted = violations
      .map(
        (v) =>
          `  ✗ ${v.id} (${v.impact}): ${v.description}\n` +
          v.nodes
            .map((n) => `      target: ${JSON.stringify(n.target)}`)
            .join("\n"),
      )
      .join("\n");
    throw new Error(
      `axe-core found ${violations.length} accessibility violation(s):\n${formatted}`,
    );
  }
}
