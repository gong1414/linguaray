import { describe, it, expect } from "vitest";
import { render, screen } from "@solidjs/testing-library";
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

  it("trigger is keyboard-focusable (keyboard path contract)", async () => {
    render(() => (
      <Tooltip content="Helpful tip">
        <span>Info</span>
      </Tooltip>
    ));
    const trigger = screen.getByText("Info");
    // The Tooltip.Trigger renders as a focusable element (button by default)
    expect(trigger.closest("button")).toBeTruthy();
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
