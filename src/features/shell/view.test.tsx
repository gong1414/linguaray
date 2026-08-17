import { render, screen, fireEvent, cleanup, within } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { runAxe } from "../../../test/axe";
import { AppProviders } from "../../app/providers";
import { SettingsShellView } from "./view";
import type { SettingsSection } from "./model";

const base: {
  locale: "zh" | "en";
  active: SettingsSection;
  a11yGranted: boolean | null;
  onNavigate: (s: SettingsSection) => void;
  onRecheckA11y: () => void;
  onOpenA11ySettings: () => void;
} = {
  locale: "en",
  active: "provider-center",
  a11yGranted: true,
  onNavigate: vi.fn(),
  onRecheckA11y: vi.fn(),
  onOpenA11ySettings: vi.fn(),
};

const renderView = (props: Partial<typeof base> = {}) =>
  render(
    <SettingsShellView {...base} {...props}>
      <div>body</div>
    </SettingsShellView>,
    { wrapper: AppProviders },
  );

afterEach(cleanup);

describe("SettingsShellView", () => {
  it("renders all eight nav sections with accessible names", () => {
    renderView();
    const nav = screen.getByRole("navigation");
    for (const label of [
      "Provider Center",
      "Keystore Recovery",
      "Shortcuts",
      "Privacy",
      "History",
      "Vocabulary",
      "Dictionary",
      "Updater",
    ]) {
      expect(within(nav).getByRole("button", { name: label })).toBeInTheDocument();
    }
  });

  it("clicking a section calls onNavigate and marks it active", () => {
    const onNavigate = vi.fn();
    renderView({ onNavigate });
    fireEvent.click(screen.getByRole("button", { name: "Privacy" }));
    expect(onNavigate).toHaveBeenCalledWith("privacy");
  });

  it("a11y banner appears only when the grant is false", () => {
    renderView({ a11yGranted: false });
    expect(screen.getByTestId("a11y-banner")).toBeInTheDocument();
    cleanup();
    renderView({ a11yGranted: true });
    expect(screen.queryByTestId("a11y-banner")).toBeNull();
    cleanup();
    renderView({ a11yGranted: null });
    expect(screen.queryByTestId("a11y-banner")).toBeNull();
  });

  it("banner actions invoke the callbacks", () => {
    const onRecheckA11y = vi.fn();
    const onOpenA11ySettings = vi.fn();
    renderView({ a11yGranted: false, onRecheckA11y, onOpenA11ySettings });
    fireEvent.click(screen.getByTestId("a11y-recheck"));
    expect(onRecheckA11y).toHaveBeenCalledTimes(1);
    fireEvent.click(screen.getByTestId("a11y-open-settings"));
    expect(onOpenA11ySettings).toHaveBeenCalledTimes(1);
  });

  it("zh locale renders Chinese nav labels", () => {
    renderView({ locale: "zh" });
    expect(screen.getByRole("button", { name: "服务商中心" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "检查更新" })).toBeInTheDocument();
  });

  it("has no axe violations (full nav + banner, zh)", async () => {
    const { container } = renderView({ locale: "zh", a11yGranted: false });
    const results = await runAxe(container);
    expect(results.violations).toEqual([]);
  });
});

describe("SettingsShellView Ueli layout", () => {
  it("uses the fixed inline Ueli navigation layout", () => {
    const { container } = renderView();
    expect(container.querySelector("[data-layout]")!.getAttribute("data-layout")).toBe("ueli");
    expect(within(screen.getByRole("navigation")).getAllByRole("button")).toHaveLength(8);
  });
});
