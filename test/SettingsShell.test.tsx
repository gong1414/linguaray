import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, fireEvent, cleanup, waitFor } from "@solidjs/testing-library";
import { createSignal } from "solid-js";
import SettingsShell, { type SettingsSection } from "../src/features/settings/SettingsShell";

// --- Tauri API mocks (C6: SettingsShell statically imports invoke +
// getCurrentWindow + openUrl at module load, so the mocks must be hoisted
// above the component import). ---
const { invokeMock, onFocusChangedMock, unlistenMock, openUrlMock, focusSlot } = vi.hoisted(() => {
  // focusSlot holds the focus handler the component registers so tests can
  // fire it to simulate a window focus event without reaching into Tauri.
  const focusSlot: { cb?: (e: { payload: boolean }) => void } = {};
  const unlistenMock = vi.fn();
  return {
    invokeMock: vi.fn(async (_cmd: string, _args?: unknown): Promise<unknown> => true),
    onFocusChangedMock: vi.fn(async (cb: (e: { payload: boolean }) => void) => {
      focusSlot.cb = cb;
      return unlistenMock;
    }),
    unlistenMock,
    openUrlMock: vi.fn(async () => undefined),
    focusSlot,
  };
});

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    onFocusChanged: onFocusChangedMock,
    close: vi.fn(async () => {}),
    minimize: vi.fn(async () => {}),
  }),
}));
vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: openUrlMock }));

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

beforeEach(() => {
  installMatchMedia(true);
  // Reset all Tauri mocks + default a11y_status to granted so the existing
  // (pre-C6) tests do not render the permission banner. C6 tests override
  // the a11y_status return per-test.
  invokeMock.mockReset();
  invokeMock.mockResolvedValue(true);
  // mockClear (NOT mockReset) preserves the hoisted implementation that
  // captures the focus callback into focusSlot; mockReset would wipe it.
  onFocusChangedMock.mockClear();
  unlistenMock.mockReset();
  openUrlMock.mockReset();
  openUrlMock.mockResolvedValue(undefined);
  focusSlot.cb = undefined;
});
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

  it("Shortcuts and Privacy are enabled R3b destinations", () => {
    const { getByText } = render(() => <SettingsShell>body</SettingsShell>);
    const shortcuts = getByText("Shortcuts").closest("button") as HTMLButtonElement;
    const privacy = getByText("Privacy").closest("button") as HTMLButtonElement;
    expect(shortcuts.getAttribute("aria-disabled")).not.toBe("true");
    expect(privacy.getAttribute("aria-disabled")).not.toBe("true");
  });

  it("rail mode (matchMedia wide=false) keeps an accessible name on every nav item", () => {
    installMatchMedia(false);
    const { container } = render(() => <SettingsShell>body</SettingsShell>);
    const buttons = container.querySelectorAll("nav button");
    // 8 nav items after R5 added the Updater section.
    expect(buttons.length).toBe(8);
    for (const btn of Array.from(buttons)) {
      const label = (btn.getAttribute("aria-label") ?? "").trim();
      const text = (btn.textContent ?? "").trim();
      expect(label.length + text.length, "rail nav item needs a non-empty accessible name").toBeGreaterThan(0);
    }
  });

  it("has no disabled placeholder nav items after R3b routes go live", () => {
    installMatchMedia(true);
    const { container } = render(() => <SettingsShell>body</SettingsShell>);
    const disabledBtns = container.querySelectorAll('button[aria-disabled="true"]');
    expect(disabledBtns.length).toBe(0);
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

describe("SettingsShell — macOS Accessibility permission (C6)", () => {
  it("shows the macOS Accessibility permission warning when not granted", async () => {
    // a11y_status resolves false → banner renders with the localized title +
    // hint + Re-check + Open System Settings actions. Default locale is en.
    invokeMock.mockResolvedValue(false);
    const { getByText, getByTestId } = render(() => <SettingsShell>body</SettingsShell>);
    await waitFor(() => expect(getByTestId("a11y-banner")).toBeInTheDocument());
    expect(getByText("Accessibility permission needed")).toBeInTheDocument();
    expect(getByText("Re-check")).toBeInTheDocument();
    expect(getByText("System Settings")).toBeInTheDocument();
  });

  it("Re-check re-invokes a11y_status", async () => {
    invokeMock.mockResolvedValue(false);
    const { getByTestId } = render(() => <SettingsShell>body</SettingsShell>);
    await waitFor(() => expect(getByTestId("a11y-banner")).toBeInTheDocument());
    // Clear the onMount call so the next a11y_status is attributable to the
    // Re-check button click.
    invokeMock.mockClear();
    fireEvent.click(getByTestId("a11y-recheck"));
    await waitFor(() =>
      expect(invokeMock.mock.calls.some((c) => c[0] === "a11y_status")).toBe(true),
    );
  });

  it("registers exactly one onFocusChanged listener and re-checks on focus (P1-9)", async () => {
    invokeMock.mockResolvedValue(false);
    render(() => <SettingsShell>body</SettingsShell>);
    // P1-9: assert exactly ONE registration before testing behavior.
    await waitFor(() => expect(onFocusChangedMock).toHaveBeenCalledTimes(1));
    // Fire the captured focus callback → recheckA11y re-invokes a11y_status.
    invokeMock.mockClear();
    expect(focusSlot.cb).toBeDefined();
    focusSlot.cb!({ payload: true });
    await waitFor(() =>
      expect(invokeMock.mock.calls.some((c) => c[0] === "a11y_status")).toBe(true),
    );
  });

  it("onCleanup calls the unlisten returned by onFocusChanged (P1-9)", async () => {
    invokeMock.mockResolvedValue(true);
    render(() => <SettingsShell>body</SettingsShell>);
    await waitFor(() => expect(onFocusChangedMock).toHaveBeenCalledTimes(1));
    expect(unlistenMock).not.toHaveBeenCalled();
    // Unmounting the shell runs the onCleanup that calls the focus unlisten.
    cleanup();
    expect(unlistenMock).toHaveBeenCalledTimes(1);
  });

  it("hardens onCleanup against the async-resolution race: unlisten IS called when onFocusChanged resolves AFTER unmount", async () => {
    // Regression for the C6 race: if the component unmounts BEFORE the
    // dynamic import + onFocusChanged() promise resolves, the old onCleanup
    // captured an undefined `unlisten` and silently leaked the listener.
    // The fix sets a `cancelled` flag so the resolve path tears down a
    // listener that arrived late. Here we keep onFocusChanged pending across
    // the unmount, then resolve it and assert unlisten IS invoked.
    invokeMock.mockResolvedValue(false);
    // Swap the hoisted mock for THIS test only: return a controllable promise
    // that stays pending until we resolve it manually. The resolved value is
    // the unlisten function (mirrors the real Tauri onFocusChanged contract).
    let resolveFocus!: (u: () => void) => void;
    const pendingFocus = new Promise<() => void>((res) => {
      resolveFocus = res;
    });
    onFocusChangedMock.mockImplementationOnce((cb: (e: { payload: boolean }) => void) => {
      focusSlot.cb = cb;
      return pendingFocus as unknown as ReturnType<typeof onFocusChangedMock>;
    });
    render(() => <SettingsShell>body</SettingsShell>);
    // onFocusChanged was called (registration happened) but its promise is
    // still pending — do NOT await it.
    await waitFor(() => expect(onFocusChangedMock).toHaveBeenCalledTimes(1));
    expect(unlistenMock).not.toHaveBeenCalled();
    // Unmount BEFORE the focus promise resolves. The old code would leave
    // `unlisten` undefined here and leak the listener.
    cleanup();
    expect(unlistenMock).not.toHaveBeenCalled();
    // Now resolve the pending onFocusChanged promise. The cancelled-flag path
    // must immediately invoke unlisten so the late-arriving listener is torn
    // down instead of leaked.
    resolveFocus(unlistenMock);
    await waitFor(() => expect(unlistenMock).toHaveBeenCalledTimes(1));
  });
});
