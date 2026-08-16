import axe from "axe-core";

/**
 * jsdom has no canvas implementation, so axe's color-contrast rule cannot
 * produce a trustworthy result there and emits a noisy not-implemented error.
 * Browser/Storybook tests own color contrast; unit tests cover every other
 * deterministic axe rule.
 */
export const runAxe = (root: Element) =>
  axe.run(root, {
    rules: {
      "color-contrast": { enabled: false },
    },
  });
