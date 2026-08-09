import { describe, it, expect } from "vitest";
import { render } from "@solidjs/testing-library";
import Tooltip from "./Tooltip";
import IconButton from "./IconButton";
import { Copy } from "lucide-solid";
import { assertNoAxeViolations } from "../../test/setup";

// jsdom matchMedia mock — Kobante's tooltip queries matchMedia on mount.
if (!window.matchMedia) {
  // @ts-expect-error partial mock
  window.matchMedia = () => ({
    matches: false,
    addEventListener() {},
    removeEventListener() {},
  });
}

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

  it("as={IconButton}: focus opens tooltip and establishes aria-describedby link", async () => {
    // The tooltip content is rendered into a Portal, so query document.body.
    const { container } = render(() => (
      <Tooltip content="Copy this value" as={IconButton} triggerProps={{ "aria-label": "Copy" }}>
        <Copy size={16} />
      </Tooltip>
    ));
    const trigger = container.querySelector(".lr-tooltip__trigger") as HTMLElement;
    expect(trigger.tagName).toBe("BUTTON");

    // BEFORE open: no tooltip content in the DOM, no describedby yet.
    expect(document.body.querySelector(".lr-tooltip__content")).toBeNull();

    // Focus opens the tooltip (Kobante opens on focus OR hover by default).
    trigger.focus();
    // Let Solid + Kobante effects flush (microtask → macrotask).
    await new Promise((r) => setTimeout(r, 0));

    // AFTER open: tooltip content exists in the portal...
    const content = document.body.querySelector(".lr-tooltip__content") as HTMLElement | null;
    expect(content).toBeTruthy();
    expect(content?.textContent).toContain("Copy this value");

    // ...AND the trigger carries aria-describedby pointing at that content's id.
    const describedById = trigger.getAttribute("aria-describedby");
    expect(typeof describedById).toBe("string");
    expect(describedById!.length).toBeGreaterThan(0);
    // The referenced id MUST be the tooltip content element — proves linkage.
    expect(content?.id).toBe(describedById);
  });

  it("Esc closes an open tooltip", async () => {
    const { container } = render(() => (
      <Tooltip content="Tip" as={IconButton} triggerProps={{ "aria-label": "Copy" }}>
        <Copy size={16} />
      </Tooltip>
    ));
    const trigger = container.querySelector(".lr-tooltip__trigger") as HTMLElement;
    trigger.focus();
    await new Promise((r) => setTimeout(r, 0));
    expect(document.body.querySelector(".lr-tooltip__content")).toBeTruthy();

    // Esc dismisses the tooltip (Kobante default).
    trigger.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
    await new Promise((r) => setTimeout(r, 0));
    expect(document.body.querySelector(".lr-tooltip__content")).toBeNull();
  });

  it("blur closes an open tooltip", async () => {
    const { container } = render(() => (
      <Tooltip content="Tip" as={IconButton} triggerProps={{ "aria-label": "Copy" }}>
        <Copy size={16} />
      </Tooltip>
    ));
    const trigger = container.querySelector(".lr-tooltip__trigger") as HTMLElement;
    // Focus opens the tooltip (Kobante opens on focus OR hover by default).
    trigger.focus();
    await new Promise((r) => setTimeout(r, 0));
    expect(document.body.querySelector(".lr-tooltip__content")).toBeTruthy();

    // Blurring the trigger dismisses the tooltip (Kobante default). The tooltip
    // is NOT a menu — focus leaving the trigger must close it. We assert the
    // programmatic contract (the content leaves the portal), not a visual
    // pseudo-class like :focus-visible (unreliable under jsdom).
    trigger.dispatchEvent(new FocusEvent("blur", { bubbles: false }));
    await new Promise((r) => setTimeout(r, 0));
    expect(document.body.querySelector(".lr-tooltip__content")).toBeNull();
  });

  it("as={IconButton}: ref passed via triggerProps lands on the native button", () => {
    // The rail uses this to confirm the trigger is the real focusable element.
    const ref: { current?: HTMLElement } = {};
    render(() => (
      <Tooltip
        content="Copy"
        as={IconButton}
        triggerProps={{ "aria-label": "Copy", ref: (el: HTMLElement) => { ref.current = el; } }}
      >
        <Copy size={16} />
      </Tooltip>
    ));
    expect(ref.current).toBeTruthy();
    expect(ref.current!.tagName).toBe("BUTTON");
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
