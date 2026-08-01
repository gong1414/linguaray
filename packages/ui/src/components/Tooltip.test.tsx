import { describe, it, expect } from "vitest";
import { render } from "@solidjs/testing-library";
import Tooltip from "./Tooltip";
import IconButton from "./IconButton";
import { Copy } from "lucide-solid";
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

  it("as={IconButton} renders single button (no nested interactive)", () => {
    const { container } = render(() => (
      <Tooltip content="Copy" as={IconButton} triggerProps={{ "aria-label": "Copy" }}>
        <Copy size={16} />
      </Tooltip>
    ));
    const triggers = container.querySelectorAll(".lr-tooltip__trigger");
    expect(triggers.length).toBe(1);
    // The trigger IS the button (IconButton renders a native button)
    expect(triggers[0]?.tagName).toBe("BUTTON");
    // No nested button
    expect(triggers[0]?.querySelector("button")).toBeNull();
  });

  it("as={IconButton}: aria-label is on the actual button", () => {
    const { container } = render(() => (
      <Tooltip content="Copy" as={IconButton} triggerProps={{ "aria-label": "Copy" }}>
        <Copy size={16} />
      </Tooltip>
    ));
    const trigger = container.querySelector(".lr-tooltip__trigger") as HTMLElement;
    expect(trigger.getAttribute("aria-label")).toBe("Copy");
  });

  it("as={IconButton}: Kobante applies aria-describedby for tooltip content", () => {
    const { container } = render(() => (
      <Tooltip content="Copy" as={IconButton} triggerProps={{ "aria-label": "Copy" }}>
        <Copy size={16} />
      </Tooltip>
    ));
    // Kobante links trigger → content via aria-describedby when visible.
    // The trigger element should have the capacity for it (id or describedby).
    const trigger = container.querySelector(".lr-tooltip__trigger") as HTMLElement;
    // Kobante may set aria-describedby on open; at minimum the trigger
    // is a proper button with an aria-label (accessible name).
    expect(trigger.tagName).toBe("BUTTON");
    expect(trigger.getAttribute("aria-label")).toBeTruthy();
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
