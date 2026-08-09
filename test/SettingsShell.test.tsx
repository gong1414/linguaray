import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, fireEvent, cleanup } from "@solidjs/testing-library";
import { createSignal } from "solid-js";
import SettingsShell, { type SettingsSection } from "../src/features/settings/SettingsShell";

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
  (window.matchMedia as unknown) = vi.fn(impl);
}

beforeEach(() => installMatchMedia(true));
afterEach(() => {
  cleanup();
  // restore the setup.ts default stub
  (window.matchMedia as unknown) = () => ({
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

  it("Shortcuts and Privacy are aria-disabled placeholders (NOT native disabled)", () => {
    // rev-9: disabled nav items announce via aria-disabled but MUST stay
    // focusable (in the tab order) so keyboard + SR users can discover them.
    const { getByText } = render(() => <SettingsShell>body</SettingsShell>);
    const shortcuts = getByText("Shortcuts").closest("button") as HTMLButtonElement;
    const privacy = getByText("Privacy").closest("button") as HTMLButtonElement;
    expect(shortcuts.getAttribute("aria-disabled")).toBe("true");
    expect(privacy.getAttribute("aria-disabled")).toBe("true");
    // Native disabled would drop them from the tab order — forbidden.
    expect(shortcuts.hasAttribute("disabled")).toBe(false);
    expect(privacy.hasAttribute("disabled")).toBe(false);
  });

  it("rail mode (matchMedia wide=false) keeps an accessible name on every nav item", () => {
    installMatchMedia(false);
    const { container } = render(() => <SettingsShell>body</SettingsShell>);
    const buttons = container.querySelectorAll("nav button");
    expect(buttons.length).toBe(4);
    for (const btn of Array.from(buttons)) {
      const label = (btn.getAttribute("aria-label") ?? "").trim();
      const text = (btn.textContent ?? "").trim();
      expect(label.length + text.length, "rail nav item needs a non-empty accessible name").toBeGreaterThan(0);
    }
  });

  it("disabled nav items are aria-disabled AND focusable (NOT native disabled)", () => {
    installMatchMedia(true);
    const { container } = render(() => <SettingsShell>body</SettingsShell>);
    const disabledBtns = container.querySelectorAll('button[aria-disabled="true"]');
    expect(disabledBtns.length).toBe(2); // Shortcuts + Privacy
    for (const btn of Array.from(disabledBtns)) {
      expect(btn.hasAttribute("disabled"), "aria-disabled items must not be native-disabled").toBe(false);
      // tabindex defaults to 0 for buttons; an explicit "-1" would remove focus.
      expect(btn.getAttribute("tabindex"), "aria-disabled items must remain in tab order").not.toBe("-1");
    }
  });

  it("disabled placeholder nav item announces the real placeholderHint copy (Coming in R3b)", () => {
    const { container } = render(() => <SettingsShell>body</SettingsShell>);
    const disabledBtns = container.querySelectorAll('button[aria-disabled="true"]');
    const labels = Array.from(disabledBtns).map((b) => b.getAttribute("aria-label") ?? "");
    // Every disabled item's aria-label appends the placeholder hint.
    for (const label of labels) {
      expect(label).toContain("Coming in R3b");
    }
  });

  it("controlled activePage prop reactively updates data-page + sidebar highlight (rev-9-2)", () => {
    // Parent-owned signal drives `activePage`; switching it re-derives both the
    // shell's data-page and which SidebarItem carries aria-current="page".
    const [page, setPage] = createSignal<SettingsSection>("provider-center");
    const { container, getByText } = render(() => (
      <SettingsShell activePage={page()} onNavigate={setPage}>body</SettingsShell>
    ));
    const shell = container.querySelector("[data-page]") as HTMLElement;
    expect(shell.getAttribute("data-page")).toBe("provider-center");
    const providerBtn = getByText("Provider Center").closest("button")!;
    expect(providerBtn.getAttribute("aria-current")).toBe("page");
    // Parent flips the signal — the controlled prop re-flows WITHOUT a click.
    setPage("keystore-recovery");
    expect(shell.getAttribute("data-page")).toBe("keystore-recovery");
    expect(getByText("Keystore Recovery").closest("button")!.getAttribute("aria-current")).toBe("page");
    expect(providerBtn.getAttribute("aria-current")).toBeNull();
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
    // In rail mode each nav item is wrapped in a Tooltip so the label survives
    // the visual collapse. The Tooltip trigger wrapper (not the lazily-opened
    // portal content) is present in the rendered tree at mount.
    const triggers = container.querySelectorAll(".lr-tooltip__trigger");
    expect(triggers.length).toBeGreaterThanOrEqual(1);
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

  it("tooltip trigger carries an accessible label (aria-label)", () => {
    // Rail mode wraps every nav item in a Tooltip. The underlying SidebarItem
    // button carries a non-empty aria-label so the trigger is reachable +
    // announced even when the visible label is hidden.
    installMatchMedia(false);
    const { container } = render(() => <SettingsShell>body</SettingsShell>);
    const triggers = container.querySelectorAll(".lr-tooltip__trigger");
    expect(triggers.length).toBeGreaterThanOrEqual(1);
    // The trigger wraps a button with an aria-label.
    const labeledTriggers = Array.from(triggers).filter((tr) => {
      const btn = tr.querySelector("button[aria-label]");
      return btn && (btn.getAttribute("aria-label") ?? "").length > 0;
    });
    expect(labeledTriggers.length).toBeGreaterThanOrEqual(1);
  });
});
