import { describe, it, expect } from "vitest";
import { render } from "@solidjs/testing-library";
import StatusBadge from "./StatusBadge";
import { assertNoAxeViolations } from "../../test/setup";

const variants = ["success", "warning", "danger", "info", "neutral"] as const;

describe("StatusBadge", () => {
  it.each(variants)("renders %s variant with children", (variant) => {
    const { getByText } = render(() => <StatusBadge variant={variant}>Test</StatusBadge>);
    expect(getByText("Test")).toBeInTheDocument();
  });

  it("dot mode renders dot element", () => {
    const { container } = render(() => <StatusBadge variant="success" dot>OK</StatusBadge>);
    expect(container.querySelector(".status-badge__dot")).not.toBeNull();
  });

  it("icon renders when provided", () => {
    const { container } = render(() => (
      <StatusBadge variant="info" icon={<span data-testid="ic" />}>Info</StatusBadge>
    ));
    expect(container.querySelector(".status-badge__icon")).not.toBeNull();
  });

  it.each(variants)("no axe violations for %s", async (variant) => {
    render(() => <StatusBadge variant={variant}>{`${variant} badge`}</StatusBadge>);
    // Isolated component in jsdom has no landmark ancestor (region rule).
    await assertNoAxeViolations({ disableRules: ["region"] });
  });
});
