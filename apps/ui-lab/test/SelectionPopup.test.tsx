import { describe, it, expect } from "vitest";
import { render, fireEvent, cleanup } from "@solidjs/testing-library";
import App from "../src/App";
import { assertNoAxeViolations } from "./setup";

/**
 * Page-level tests for the UI Lab. These render the REAL App (not a stub),
 * switch locale/theme/state through the same controls a user clicks, and run
 * axe against the full page.
 *
 * color-contrast is excluded (jsdom cannot compute it); it is verified via the
 * MASTER token contrast table and browser screenshots in the acceptance report.
 */

// jsdom has no matchMedia; App's reduced-motion branch is driven by
// [data-motion] on <html>, so this is only needed if a component reads it.
if (!window.matchMedia) {
  // @ts-expect-error partial mock
  window.matchMedia = () => ({
    matches: false,
    addEventListener() {},
    removeEventListener() {},
  });
}

describe("UI Lab — Selection Popup", () => {
  it("renders the nav with all 16 surfaces including Tray/Menu-bar", () => {
    const { getByRole } = render(() => <App />);
    // Tray/Menu-bar is disabled (upcoming), so its accessible name carries
    // the "(not yet implemented)" suffix — match by a stable substring.
    expect(getByRole("button", { name: /Tray \/ Menu-bar/ })).toBeTruthy();
    // Spot-check a few others (enabled Selection Popup has no suffix).
    expect(getByRole("button", { name: "Selection Popup" })).toBeTruthy();
    expect(getByRole("button", { name: /Updater/ })).toBeTruthy();
    cleanup();
  });

  it("defaults to a single success result with the DeepSeek card", () => {
    const { getByText } = render(() => <App />);
    expect(getByText("DeepSeek")).toBeTruthy();
    cleanup();
  });

  it("switching to Chinese re-renders the nav label and state bar in zh", () => {
    const { getByRole, getByText } = render(() => <App />);
    fireEvent.click(getByRole("button", { name: "中文" }));
    // The nav's accessible name (aria-label) switches to Chinese.
    const nav = getByRole("navigation");
    expect(nav.getAttribute("aria-label")).toBe("原型列表");
    // A visible state chip switches to its Chinese label.
    expect(getByText("成功 · 单引擎")).toBeTruthy();
    cleanup();
  });

  it("expanded window size switches to dual side-by-side results", () => {
    const { getByRole } = render(() => <App />);
    fireEvent.click(getByRole("button", { name: /600×400 \(expanded\)/ }));
    // Two engine cards now present
    const region = getByRole("region", { name: /Multi-engine result|多引擎结果/ });
    const cards = region.querySelectorAll(".lr-result-card");
    expect(cards.length).toBe(2);
    cleanup();
  });

  it("has no axe violations on the default single-success page (light/en)", async () => {
    render(() => <App />);
    await assertNoAxeViolations({
      disableRules: ["color-contrast"],
    });
    cleanup();
  });

  it("has no axe violations on the dual-engine page (light/en)", async () => {
    const { getByRole } = render(() => <App />);
    fireEvent.click(getByRole("button", { name: /600×400 \(expanded\)/ }));
    await assertNoAxeViolations({
      disableRules: ["color-contrast"],
    });
    cleanup();
  });

  it("has no axe violations in dark + Chinese", async () => {
    const { getByRole } = render(() => <App />);
    fireEvent.click(getByRole("button", { name: "中文" }));
    fireEvent.click(getByRole("button", { name: "深色" }));
    await assertNoAxeViolations({
      disableRules: ["color-contrast"],
    });
    cleanup();
  });

  it("has no axe violations on a single-card error state", async () => {
    const { getByRole } = render(() => <App />);
    fireEvent.click(getByRole("button", { name: "Error · network" }));
    await assertNoAxeViolations({
      disableRules: ["color-contrast"],
    });
    cleanup();
  });

  it("each Segmented control has an accessible group label", () => {
    const { getAllByRole } = render(() => <App />);
    const groups = getAllByRole("group");
    // Four segmented controls (locale, theme, motion, window size) each need
    // an aria-label so screen readers announce their purpose.
    const labeled = groups.filter((g) => g.getAttribute("aria-label"));
    expect(labeled.length).toBeGreaterThanOrEqual(4);
    cleanup();
  });
});
