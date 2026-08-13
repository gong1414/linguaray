/**
 * axe-core assertion helper for production component a11y tests.
 *
 * Ported from the deleted `apps/ui-lab/test/setup.ts` (commit 7f21adc). Runs
 * axe against document.body and throws a formatted report on any violation.
 *
 * R7-P1-3: `aria-allowed-role` was previously disabled because PopupView used
 * `<main role="region">` (main has an implicit role; adding role="region" is
 * a redundant override → aria-allowed-role violation). R7 changed the element
 * to `<section>` (section has no implicit role; role="region" is allowed via
 * aria-label), so `aria-allowed-role` is now ENFORCED. Callers only need to
 * disable `color-contrast` (jsdom has no real layout/render; it is verified
 * deterministically via the MASTER token contrast table and by browser
 * screenshots in the acceptance report).
 */
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
          `  x ${v.id} (${v.impact}): ${v.description}\n` +
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
