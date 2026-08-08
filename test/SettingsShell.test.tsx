import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, fireEvent, cleanup } from "@solidjs/testing-library";
import SettingsShell from "../src/features/settings/SettingsShell";

// matchMedia is stubbed globally in test/setup.ts (matches:false). Per-test we
// install a fresh implementation to simulate the two breakpoints.
function installMatchMedia(matchesWide: boolean) {
  const impl = (query: string) => ({
    matches: query.includes("700") ? matchesWide : false,
    media: query,
    onchange: null,
    addEventListener() {},
    removeEventListener() {},
    addListener() {},
    removeListener() {},
    dispatchEvent: () => false,
  });
  // @ts-expect-error partial mock
  window.matchMedia = vi.fn(impl);
}

beforeEach(() => installMatchMedia(true));
afterEach(() => {
  cleanup();
  // restore the setup.ts default stub
  // @ts-expect-error partial mock
  window.matchMedia = () => ({
    matches: false,
    addEventListener() {},
    removeEventListener() {},
    onchange: null,
    dispatchEvent: () => false,
    addListener() {},
    removeListener() {},
  });
});

describe("SettingsShell", () => {
  it("renders WindowChrome with title LinguaRay", () => {
    const { getByText } = render(() => <SettingsShell>body</SettingsShell>);
    expect(getByText("LinguaRay")).toBeInTheDocument();
  });

  it("renders all four nav items", () => {
    const { getByText } = render(() => <SettingsShell>body</SettingsShell>);
    const labels = ["Provider Center", "Keystore Recovery", "Shortcuts", "Privacy"];
    for (const label of labels) {
      const node = getByText(label);
      const btn = node.closest("button");
      expect(btn, `${label} should be in a button`).not.toBeNull();
    }
  });

  it("Shortcuts and Privacy are disabled (placeholder)", () => {
    const { getByText } = render(() => <SettingsShell>body</SettingsShell>);
    const shortcuts = getByText("Shortcuts").closest("button") as HTMLButtonElement;
    const privacy = getByText("Privacy").closest("button") as HTMLButtonElement;
    expect(shortcuts.disabled).toBe(true);
    expect(privacy.disabled).toBe(true);
  });

  it("clicking Provider Center calls onNavigate with provider-center", () => {
    const onNavigate = vi.fn();
    const { getByText } = render(() => (
      <SettingsShell onNavigate={onNavigate}>body</SettingsShell>
    ));
    fireEvent.click(getByText("Provider Center").closest("button")!);
    expect(onNavigate).toHaveBeenCalledWith("provider-center");
  });

  it("clicking Keystore Recovery calls onNavigate with keystore-recovery", () => {
    const onNavigate = vi.fn();
    const { getByText } = render(() => (
      <SettingsShell onNavigate={onNavigate}>body</SettingsShell>
    ));
    fireEvent.click(getByText("Keystore Recovery").closest("button")!);
    expect(onNavigate).toHaveBeenCalledWith("keystore-recovery");
  });

  it("at >=700px viewport, sidebar labels are visible and data-layout=full", () => {
    installMatchMedia(true);
    const { container, getByText } = render(() => <SettingsShell>body</SettingsShell>);
    const root = container.querySelector("[data-layout]");
    expect(root).not.toBeNull();
    expect(root!.getAttribute("data-layout")).toBe("full");
    // label is visible (no aria-hidden, not display:none)
    const label = getByText("Provider Center");
    expect(label).not.toHaveAttribute("hidden");
  });

  it("at 600-699px viewport, sidebar collapses to icon rail (data-layout=rail)", () => {
    installMatchMedia(false);
    const { container } = render(() => <SettingsShell>body</SettingsShell>);
    const root = container.querySelector("[data-layout]");
    expect(root).not.toBeNull();
    expect(root!.getAttribute("data-layout")).toBe("rail");
    // In rail mode each nav item is wrapped in a Tooltip; the tooltip content
    // carries the label so the accessible name survives the visual collapse.
    const tooltips = container.querySelectorAll(".lr-tooltip__content");
    expect(tooltips.length).toBeGreaterThanOrEqual(1);
  });

  it("initial active section defaults to provider-center", () => {
    const { getByText } = render(() => <SettingsShell>body</SettingsShell>);
    const btn = getByText("Provider Center").closest("button")!;
    expect(btn.getAttribute("aria-current")).toBe("page");
  });

  it("clicking a nav item moves aria-current to the new section", () => {
    const { getByText } = render(() => <SettingsShell>body</SettingsShell>);
    const providerBtn = getByText("Provider Center").closest("button")!;
    const keystoreBtn = getByText("Keystore Recovery").closest("button")!;
    expect(providerBtn.getAttribute("aria-current")).toBe("page");
    fireEvent.click(keystoreBtn);
    expect(keystoreBtn.getAttribute("aria-current")).toBe("page");
    expect(providerBtn.getAttribute("aria-current")).toBeNull();
  });
});
