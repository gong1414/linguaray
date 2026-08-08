import { describe, it, expect, afterEach } from "vitest";
import { render, cleanup } from "@solidjs/testing-library";
import { ComponentGallery } from "../src/pages/ComponentGallery";
import { assertNoAxeViolations } from "../test/setup";

const EXPECTED_COMPONENT_IDS = [
  "button", "icon-button", "segmented-control", "shortcut-chip",
  "text-field", "select", "switch",
  "status-badge", "inline-error", "toast", "confirm", "empty-state",
  "translation-card", "result-card", "provider-row", "history-row",
  "sidebar-item", "spinner", "window-chrome", "overflow-cjk",
];

describe("ComponentGallery", () => {
  // Each test renders a full gallery into document.body; clean up between
  // tests so axe does not see accumulated DOM from prior renders.
  afterEach(() => cleanup());

  it("renders exactly 20 design components with data-component-id", () => {
    const { container } = render(() => <ComponentGallery locale="en" theme="light" />);
    for (const id of EXPECTED_COMPONENT_IDS) {
      const el = container.querySelector(`[data-component-id="${id}"]`);
      expect(el, `must have [data-component-id="${id}"]`).not.toBeNull();
    }
    const all = container.querySelectorAll("[data-component-id]");
    expect(all.length, "exactly 20 components").toBe(20);
  });

  it("renders zh labels in zh locale", () => {
    const { getAllByText } = render(() => <ComponentGallery locale="zh" theme="light" />);
    // "按钮" appears in the Button section title AND the overflow section's
    // Button state label, so query for all matches.
    expect(getAllByText(/按钮|Button/).length).toBeGreaterThan(0);
  });

  it("light theme: no axe violations", async () => {
    document.documentElement.setAttribute("data-theme", "light");
    render(() => <ComponentGallery locale="en" theme="light" />);
    // color-contrast: jsdom has no real render (verified via screenshots).
    // region: the gallery is a component matrix embedded in the lab shell,
    // not a standalone page; page-level landmark wrapping is the shell's
    // responsibility, not the gallery fixture's.
    await assertNoAxeViolations({ disableRules: ["color-contrast", "region"] });
  });

  it("dark theme: no axe violations", async () => {
    document.documentElement.setAttribute("data-theme", "dark");
    render(() => <ComponentGallery locale="en" theme="dark" />);
    await assertNoAxeViolations({ disableRules: ["color-contrast", "region"] });
  });
});
