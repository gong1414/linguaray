import { describe, it, expect } from "vitest";
import { render } from "@solidjs/testing-library";
import Spinner from "./Spinner";
import { assertNoAxeViolations } from "../../test/setup";

describe("Spinner", () => {
  it("has role=status and aria-live=polite so screen readers announce it", () => {
    const { getByRole } = render(() => <Spinner />);
    const status = getByRole("status");
    expect(status).toHaveAttribute("aria-live", "polite");
  });

  it("the accessible label is present in the DOM (sr-only by default)", () => {
    const { container } = render(() => <Spinner label="Translating…" />);
    const label = container.querySelector(".lr-spinner__label");
    expect(label).toBeTruthy();
    expect(label?.textContent).toBe("Translating…");
    // Default full-motion: visually hidden.
    expect(label?.className).toContain("lr-visually-hidden");
  });

  it("renders the spinning icon (Loader2 svg) as aria-hidden", () => {
    const { container } = render(() => <Spinner size={12} />);
    const svg = container.querySelector(".lr-spinner__icon");
    expect(svg).toBeTruthy();
    expect(svg?.getAttribute("aria-hidden")).toBe("true");
  });

  it("has no axe violations", async () => {
    render(() => <Spinner label="Loading…" />);
    await assertNoAxeViolations({
      disableRules: ["color-contrast", "landmark-one-main", "page-has-heading-one", "region"],
    });
  });
});
