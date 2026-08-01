import { describe, it, expect } from "vitest";
import { render } from "@solidjs/testing-library";
import Tooltip from "./Tooltip";
import { assertNoAxeViolations } from "../../test/setup";

describe("Tooltip", () => {
  it("renders a trigger child", () => {
    const { getByText } = render(() => (
      <Tooltip content="Helpful tip">
        <span>Hover me</span>
      </Tooltip>
    ));
    expect(getByText("Hover me")).toBeTruthy();
  });

  it("default span trigger wraps children (non-interactive)", () => {
    const { container } = render(() => (
      <Tooltip content="Helpful tip">
        <span>Info</span>
      </Tooltip>
    ));
    const trigger = container.querySelector(".lr-tooltip__trigger");
    expect(trigger?.tagName).toBe("SPAN");
  });

  it("as={Button} renders single button (no nested interactive)", () => {
    const { container } = render(() => (
      <Tooltip content="Tip" as="button">
        <span>Click me</span>
      </Tooltip>
    ));
    // Exactly one button in the trigger area
    const triggers = container.querySelectorAll(".lr-tooltip__trigger");
    expect(triggers.length).toBe(1);
    expect(triggers[0]?.tagName).toBe("BUTTON");
    // No nested button-in-button
    expect(triggers[0]?.querySelector("button")).toBeNull();
  });

  it("has no axe violations", async () => {
    render(() => (
      <Tooltip content="Tip">
        <span>Info</span>
      </Tooltip>
    ));
    await assertNoAxeViolations({
      disableRules: ["color-contrast", "landmark-one-main", "page-has-heading-one", "region"],
    });
  });
});
