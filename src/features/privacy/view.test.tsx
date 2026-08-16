import { render, screen, fireEvent, cleanup, within } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { runAxe } from "../../../test/axe";
import { AppProviders } from "../../app/providers";
import { PrivacyView, type PrivacyViewProps } from "./view";

const base: PrivacyViewProps = {
  locale: "en",
  status: { enabled: true, retention_days: 30, record_count: 42 },
  loading: false,
  error: null,
  busy: null,
  clearOpen: false,
  toasts: [],
  external: { state: "disabled" },
  externalBusy: false,
  tokenOnce: null,
  tokenCopied: false,
  onRetry: () => {},
  onEnabledChange: () => {},
  onRetentionChange: () => {},
  onOpenClear: () => {},
  onCloseClear: () => {},
  onConfirmClear: () => {},
  onEnableExternal: () => {},
  onDisableExternal: () => {},
  onRegenToken: () => {},
  onCopyToken: () => {},
  onDismissToast: () => {},
};

const renderView = (props: Partial<PrivacyViewProps> = {}) =>
  render(<PrivacyView {...base} {...props} />, { wrapper: AppProviders });

afterEach(cleanup);

describe("PrivacyView", () => {
  it("renders the history panel with record count", () => {
    renderView();
    expect(screen.getByTestId("history-panel")).toBeInTheDocument();
    expect(screen.getByText("42 encrypted records")).toBeInTheDocument();
  });

  it("loading state replaces the panels", () => {
    renderView({ status: null, loading: true });
    expect(screen.getByTestId("privacy-loading")).toBeInTheDocument();
    expect(screen.queryByTestId("history-panel")).toBeNull();
  });

  it("error shows a retryable alert", () => {
    renderView({ status: null, error: "db locked" });
    expect(screen.getByTestId("privacy-error")).toHaveTextContent("db locked");
  });

  it("clear is disabled with zero records", () => {
    renderView({ status: { enabled: false, retention_days: 30, record_count: 0 } });
    expect(screen.getByRole("button", { name: "Clear All" })).toBeDisabled();
  });

  it("destructive confirm modal shows the warning and confirm action", async () => {
    const onConfirmClear = vi.fn();
    renderView({ clearOpen: true, onConfirmClear });
    const dialog = await screen.findByRole("dialog");
    expect(await within(dialog).findByText(/permanently removes all encrypted history/)).toBeInTheDocument();
    fireEvent.click(within(dialog).getByRole("button", { name: "Clear All" }));
    expect(onConfirmClear).toHaveBeenCalledTimes(1);
  });

  it("retention select is disabled while history is off", () => {
    renderView({ status: { enabled: false, retention_days: 30, record_count: 3 } });
    const select = screen.getByRole("combobox", { name: "Retention period" });
    expect(select).toBeDisabled();
  });

  it("external API section is a formal panel with status badge", () => {
    renderView({ external: { state: "enabled", port: 8787 } });
    expect(screen.getByTestId("external-panel")).toBeInTheDocument();
    expect(screen.getByText("External API: On (port 8787)")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Enable" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Disable" })).toBeEnabled();
  });

  it("token is shown once with a copy action and a security hint", () => {
    renderView({ external: { state: "enabled", port: 1 }, tokenOnce: "lray_abc" });
    const panel = screen.getByTestId("external-token");
    expect(panel).toHaveTextContent("lray_abc");
    expect(panel).toHaveTextContent("shown ONCE");
    fireEvent.click(screen.getByRole("button", { name: "Copy token" }));
    // tokenCopied flips label via props (controller owns state).
    cleanup();
    renderView({ external: { state: "enabled", port: 1 }, tokenOnce: "lray_abc", tokenCopied: true });
    expect(screen.getByRole("button", { name: "Copied" })).toBeInTheDocument();
  });

  it("toasts render live and are dismissible", () => {
    const onDismissToast = vi.fn();
    renderView({ toasts: [{ id: 1, variant: "success", message: "History cleared" }], onDismissToast });
    expect(screen.getByText("History cleared")).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Dismiss" }));
    expect(onDismissToast).toHaveBeenCalledWith(1);
  });

  it("zh locale renders Chinese copy", () => {
    renderView({ locale: "zh" });
    expect(screen.getByText("隐私与数据")).toBeInTheDocument();
    expect(screen.getByText("42 条加密记录")).toBeInTheDocument();
  });

  it("has no axe violations (populated, zh)", async () => {
    const { container } = renderView({ locale: "zh", external: { state: "enabled", port: 8787 } });
    const results = await runAxe(container);
    expect(results.violations).toEqual([]);
  });
});
