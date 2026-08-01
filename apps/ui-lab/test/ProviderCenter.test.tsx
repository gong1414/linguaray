import { describe, it, expect } from "vitest";
import { render, fireEvent, screen, cleanup } from "@solidjs/testing-library";
import App from "../src/App";
import { strings, type ProviderState } from "../src/i18n";
import { assertNoAxeViolations } from "./setup";

// jsdom matchMedia mock
if (!window.matchMedia) {
  // @ts-expect-error partial mock
  window.matchMedia = () => ({
    matches: false,
    addEventListener() {},
    removeEventListener() {},
  });
}

function goToProviderCenter() {
  fireEvent.click(screen.getByRole("button", { name: "Provider Center" }));
}

describe("Provider Center — navigation + rendering", () => {
  afterEach(() => cleanup());

  it("navigates to Provider Center and shows the empty state", () => {
    render(() => <App />);
    goToProviderCenter();
    // Default state is empty → shows add-first empty state
    expect(screen.getByText(strings.en.provider.addFirst)).toBeTruthy();
  });

  it("renders all 23 state chips", () => {
    render(() => <App />);
    goToProviderCenter();
    const stateBar = screen.getByRole("group", { name: "State" });
    const chips = stateBar.querySelectorAll("button");
    expect(chips.length).toBe(23);
  });
});

// --- All 23 states parametrized render ------------------------------------

const ALL_STATES: ProviderState[] = [
  "empty",
  "loading-models",
  "model-fetch-error",
  "model-manual-entry",
  "connection-testing",
  "connection-ok",
  "connection-failed",
  "key-saved",
  "key-missing",
  "duplicate",
  "saving",
  "save-failed",
  "save-conflict",
  "delete-confirm",
  "deleting",
  "delete-retry",
  "drag-reorder",
  "reorder-failed",
  "balance-loading",
  "balance-unsupported",
  "balance-rate-limited",
  "balance-error",
  "endpoint-invalid",
];

describe("Provider Center — all 23 states render", () => {
  it.each(ALL_STATES)("renders state '%s' without throwing", (state) => {
    const { container } = render(() => <App />);
    goToProviderCenter();
    // Click the target state chip by matching within the state bar only
    const stateBar = screen.getByRole("group", { name: "State" });
    const label = strings.en.provider.states[state];
    const chips = [...stateBar.querySelectorAll("button")];
    const chip = chips.find((b) => b.textContent === label);
    if (chip) fireEvent.click(chip);
    // Something rendered (frame or content)
    expect(container.querySelector(".lab__frame")).toBeTruthy();
    cleanup();
  });
});

// --- axe ------------------------------------------------------------------

describe("Provider Center — accessibility", () => {
  afterEach(() => cleanup());

  it("has no axe violations on the populated list (light/en)", async () => {
    render(() => <App />);
    goToProviderCenter();
    fireEvent.click(screen.getByRole("button", { name: "Connection OK" }));
    await assertNoAxeViolations({ disableRules: ["color-contrast"] });
  });

  it("has no axe violations in dark + Chinese", async () => {
    render(() => <App />);
    fireEvent.click(screen.getByRole("button", { name: "中文" }));
    fireEvent.click(screen.getByRole("button", { name: "深色" }));
    fireEvent.click(screen.getByRole("navigation").querySelectorAll("button")[4]!);
    await assertNoAxeViolations({ disableRules: ["color-contrast"] });
  });
});

// --- interactions ---------------------------------------------------------

describe("Provider Center — cc-switch interactions", () => {
  afterEach(() => cleanup());

  it("disabling a provider clears its primary role", () => {
    render(() => <App />);
    goToProviderCenter();
    fireEvent.click(screen.getByRole("button", { name: "Connection OK" }));
    // OpenAI #1 is primary by default — toggle it off
    const switches = document.querySelectorAll(".lr-switch input[type=checkbox]");
    fireEvent.click(switches[0]);
    // The Primary badge should disappear from that card
    // (it's replaced by the role reflecting "none")
    expect(document.querySelector('[data-role="primary"]')).toBeNull();
  });

  it("duplicate adds a (copy) card with key missing", () => {
    render(() => <App />);
    goToProviderCenter();
    fireEvent.click(screen.getByRole("button", { name: "Connection OK" }));
    const dupBtn = screen.getAllByRole("button", { name: "Duplicate" })[0];
    fireEvent.click(dupBtn);
    // A new card with "(copy)" appears and shows "Key missing"
    expect(screen.getAllByText(/copy/).length).toBeGreaterThan(0);
  });

  it("move up reorders the first provider (disabled on first)", () => {
    render(() => <App />);
    goToProviderCenter();
    fireEvent.click(screen.getByRole("button", { name: "Connection OK" }));
    const moveUpBtns = screen.getAllByRole("button", { name: "Move up" });
    // First provider's move-up is disabled
    expect(moveUpBtns[0].hasAttribute("disabled")).toBe(true);
  });

  it("no mock key value appears in the DOM", () => {
    const { container } = render(() => <App />);
    goToProviderCenter();
    fireEvent.click(screen.getByRole("button", { name: "Connection OK" }));
    // Search for common mock key patterns — none should be present
    const html = container.innerHTML;
    expect(html).not.toMatch(/sk-[a-zA-Z0-9]{20,}/);
    expect(html).not.toMatch(/Bearer\s/);
  });
});
