/**
 * Vitest + axe setup for UI Lab page-level tests.
 *
 * Unlike the isolated component tests, page tests run axe against the FULL
 * rendered page. We disable only rules that are genuinely unverifiable in jsdom:
 *  - color-contrast: jsdom has no real layout/render; verified deterministically
 *    via the MASTER token contrast table and by browser screenshot instead.
 */
import "@testing-library/jest-dom";
import axe from "axe-core";

export async function assertNoAxeViolations(
  options: { disableRules?: string[] } = {},
): Promise<void> {
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
