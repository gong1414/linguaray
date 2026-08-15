import { render, screen, fireEvent, cleanup } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import axe from "axe-core";
import { AppProviders } from "../../app/providers";
import { OnboardingView, type OnboardingViewProps } from "./view";

const base: OnboardingViewProps = {
  step: "welcome",
  locale: "en",
  a11y: "granted",
  screenCapture: "granted",
  providerCount: 1,
  historyBusy: false,
  shortcuts: [],
  advancing: false,
  error: null,
  onOpenA11ySettings: () => {},
  onOpenScreenCaptureSettings: () => {},
  onRecheckPermissions: () => {},
  onOpenProviderSettings: () => {},
  onOpenShortcutsSettings: () => {},
  onEnableHistory: () => {},
  onAdvance: () => {},
  onFinish: () => {},
};

/** Mantine components need the shared providers tree. */
const renderView = (props: Partial<OnboardingViewProps> = {}) =>
  render(<OnboardingView {...base} {...props} />, { wrapper: AppProviders });

afterEach(cleanup);

describe("OnboardingView", () => {
  it("welcome renders the heading and a formal start button", () => {
    renderView();
    expect(screen.getByTestId("onboarding-title")).toHaveTextContent("Welcome to LinguaRay");
    const start = screen.getByRole("button", { name: "Get started" });
    expect(start.tagName).toBe("BUTTON");
    expect(start).toBeEnabled();
  });

  it("step change focuses the new step heading", () => {
    renderView();
    // The effect focuses the heading on mount AND on every step change —
    // keyboard + SR users land on the new step's title.
    expect(screen.getByTestId("onboarding-title")).toHaveFocus();
    expect(screen.getByTestId("onboarding-title")).toHaveTextContent("Welcome to LinguaRay");
    cleanup();
    renderView({ step: "accessibility" });
    const nextTitle = screen.getByTestId("onboarding-title");
    expect(nextTitle).toHaveTextContent("Grant permissions");
    expect(nextTitle).toHaveFocus();
  });

  it("accessibility step shows both permission cards with honest badges", () => {
    renderView({ step: "accessibility", a11y: "denied", screenCapture: "checking" });
    expect(screen.getByText("Not granted")).toBeInTheDocument();
    expect(screen.getByText("Checking…")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Open Accessibility Settings" })).toBeEnabled();
  });

  it("unsupported permissions disable their open-settings button", () => {
    renderView({ step: "accessibility", screenCapture: "unsupported" });
    expect(
      screen.getByRole("button", { name: "Open Screen Recording Settings" }),
    ).toBeDisabled();
  });

  it("continue stays disabled while a11y is still checking", () => {
    renderView({ step: "accessibility", a11y: "checking" });
    expect(screen.getByRole("button", { name: "Continue" })).toBeDisabled();
  });

  it("provider hint covers checking / empty / populated", () => {
    renderView({ step: "provider", providerCount: null });
    expect(screen.getByText("Checking…")).toBeInTheDocument();
    cleanup();
    renderView({ step: "provider", providerCount: 0 });
    expect(screen.getByText(/No provider yet/)).toBeInTheDocument();
    cleanup();
    renderView({ step: "provider", providerCount: 3 });
    expect(screen.getByText("3 providers configured")).toBeInTheDocument();
  });

  it("history busy disables both actions and shows the enabling label", () => {
    renderView({ step: "history", historyBusy: true });
    expect(screen.getByRole("button", { name: "Enabling…" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Skip for now" })).toBeDisabled();
  });

  it("shortcuts list maps combos to labels + Kbd", () => {
    renderView({
      step: "shortcuts",
      shortcuts: [{ action: "translate_selection", combo: "Alt+D" }],
    });
    expect(screen.getByText("Translate selection")).toBeInTheDocument();
    expect(screen.getByText("Alt+D").tagName).toBe("KBD");
  });

  it("error renders as a live alert with the localized prefix", () => {
    renderView({ error: "db locked" });
    const alert = screen.getByTestId("onboarding-error");
    expect(alert).toHaveAttribute("role", "alert");
    expect(alert).toHaveTextContent("Something went wrong");
    expect(alert).toHaveTextContent("db locked");
  });

  it("advancing disables the step's primary action", () => {
    renderView({ step: "provider", advancing: true });
    expect(screen.getByRole("button", { name: "Continue" })).toBeDisabled();
  });

  it("zh locale renders Chinese copy end-to-end", () => {
    renderView({ locale: "zh", step: "accessibility", a11y: "denied" });
    expect(screen.getByTestId("onboarding-title")).toHaveTextContent("授予权限");
    expect(screen.getByText("未授权")).toBeInTheDocument();
  });

  it("buttons call their callbacks (advance/finish/open settings)", () => {
    const onAdvance = vi.fn();
    renderView({ onAdvance });
    fireEvent.click(screen.getByRole("button", { name: "Get started" }));
    expect(onAdvance).toHaveBeenCalledWith("start");
    cleanup();

    const onOpenA11ySettings = vi.fn();
    renderView({
      step: "accessibility",
      a11y: "denied",
      onOpenA11ySettings,
    });
    fireEvent.click(screen.getByRole("button", { name: "Open Accessibility Settings" }));
    expect(onOpenA11ySettings).toHaveBeenCalledTimes(1);
    cleanup();

    const onFinish = vi.fn();
    renderView({ step: "done", onFinish });
    fireEvent.click(screen.getByRole("button", { name: "Start using LinguaRay" }));
    expect(onFinish).toHaveBeenCalledWith(false);
  });

  it("has no axe violations on the accessibility step (light)", async () => {
    const { container } = renderView({ step: "accessibility", a11y: "denied", screenCapture: "granted" });
    const results = await axe.run(container);
    expect(results.violations).toEqual([]);
  });
});
