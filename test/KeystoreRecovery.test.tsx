import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, fireEvent, cleanup, waitFor } from "@solidjs/testing-library";

// Default mock: keystore_health returns "" (healthy). Each test overrides
// invoke as needed via vi.mocked(invoke).mockImplementation.
const invoke = vi.fn(async (cmd: string) => {
  if (cmd === "keystore_health") return "";
  throw new Error(`unexpected invoke ${cmd}`);
});

vi.mock("@tauri-apps/api/core", () => ({ invoke }));

// detectLocale is mocked per-test via vi.doMock-style override; default "en".
let locale = "en";
vi.mock("../src/i18n", () => ({
  detectLocale: () => locale,
}));

import KeystoreRecovery from "../src/features/settings/KeystoreRecovery";

const flush = () => new Promise((r) => setTimeout(r, 0));

beforeEach(() => {
  locale = "en";
  invoke.mockImplementation(async (cmd: string) => {
    if (cmd === "keystore_health") return "";
    throw new Error(`unexpected invoke ${cmd}`);
  });
});

afterEach(() => cleanup());

describe("KeystoreRecovery (Surface 06)", () => {
  it("on mount, calls keystore_health once", async () => {
    render(() => <KeystoreRecovery />);
    await waitFor(() => expect(invoke).toHaveBeenCalledWith("keystore_health"));
  });

  it("healthy: renders no alert banner", async () => {
    const { container } = render(() => <KeystoreRecovery />);
    await waitFor(() => expect(invoke).toHaveBeenCalledWith("keystore_health"));
    await flush();
    expect(container.querySelector('[role="alert"]')).toBeNull();
    expect(container.querySelector('[role="status"]')).toBeNull();
  });

  it("corrupt: renders destructive banner with the reason and two buttons", async () => {
    invoke.mockResolvedValueOnce("corrupt: header mismatch");
    const { getByText, findByText } = render(() => <KeystoreRecovery />);
    // title
    expect(await findByText("Keystore unreadable")).toBeTruthy();
    // description interpolates {reason}
    expect(getByText(/corrupt: header mismatch/)).toBeTruthy();
    // two action buttons
    expect(getByText("Archive & re-enter")).toBeTruthy();
    expect(getByText("Reset")).toBeTruthy();
  });

  it("corrupt to Archive: calls archive_keystore, transitions to archived state", async () => {
    invoke.mockResolvedValueOnce("corrupt: header mismatch");
    invoke.mockResolvedValueOnce("/path/to/archive"); // archive_keystore
    const { findByText, getByText } = render(() => <KeystoreRecovery />);
    expect(await findByText("Keystore unreadable")).toBeTruthy();
    fireEvent.click(getByText("Archive & re-enter"));
    expect(await findByText("Keys archived")).toBeTruthy();
    expect(getByText("Enter your keys again")).toBeTruthy();
    expect(invoke).toHaveBeenCalledWith("archive_keystore");
  });

  it("corrupt to Reset: opens destructive Confirm; initial focus on Cancel", async () => {
    invoke.mockResolvedValueOnce("corrupt: header mismatch");
    const { findByText, getByText } = render(() => <KeystoreRecovery />);
    expect(await findByText("Keystore unreadable")).toBeTruthy();
    fireEvent.click(getByText("Reset"));
    // dialog title appears
    expect(await findByText("Reset keystore?")).toBeTruthy();
    await flush(); // let Kobalte's onOpenAutoFocus settle
    const cancelBtn = getByText("Cancel").closest("button") as HTMLButtonElement;
    expect(document.activeElement).toBe(cancelBtn);
  });

  it("reset Confirm to Cancel: closes dialog, stays corrupt", async () => {
    invoke.mockResolvedValueOnce("corrupt: header mismatch");
    const { findByText, getByText, queryByText } = render(() => <KeystoreRecovery />);
    expect(await findByText("Keystore unreadable")).toBeTruthy();
    fireEvent.click(getByText("Reset"));
    expect(await findByText("Reset keystore?")).toBeTruthy();
    fireEvent.click(getByText("Cancel"));
    await flush();
    expect(queryByText("Reset keystore?")).toBeNull();
    // still corrupt
    expect(getByText("Keystore unreadable")).toBeTruthy();
  });

  it("reset Confirm to Confirm: calls reset_keystore, transitions to archived", async () => {
    invoke.mockResolvedValueOnce("corrupt: header mismatch");
    invoke.mockResolvedValueOnce(null); // reset_keystore returns Option<string>
    const { findByText, getByText, queryByText } = render(() => <KeystoreRecovery />);
    expect(await findByText("Keystore unreadable")).toBeTruthy();
    fireEvent.click(getByText("Reset"));
    expect(await findByText("Reset keystore?")).toBeTruthy();
    // There are two "Reset" labels: the banner action and the dialog confirm.
    // Click the confirm one inside the dialog footer.
    const dialog = getByText("Reset keystore?").closest(".lr-dialog__content");
    const confirmBtn = dialog!.querySelectorAll("button");
    const resetConfirm = Array.from(confirmBtn).find(
      (b) => b.textContent === "Reset",
    ) as HTMLButtonElement;
    fireEvent.click(resetConfirm);
    expect(await findByText("Keys archived")).toBeTruthy();
    expect(queryByText("Reset keystore?")).toBeNull();
    expect(invoke).toHaveBeenCalledWith("reset_keystore");
  });

  it("archive_keystore rejects: shows destructive toast", async () => {
    invoke.mockResolvedValueOnce("corrupt: header mismatch");
    invoke.mockRejectedValueOnce(new Error("io error")); // archive fails
    const { findByText, getByText } = render(() => <KeystoreRecovery />);
    expect(await findByText("Keystore unreadable")).toBeTruthy();
    fireEvent.click(getByText("Archive & re-enter"));
    // destructive toast (role=alert) carries the failure message
    expect(await findByText(/Archive failed|io error/)).toBeTruthy();
  });

  it("uses zh copy when locale is zh", async () => {
    locale = "zh";
    invoke.mockResolvedValueOnce("corrupt: header mismatch");
    const { findByText } = render(() => <KeystoreRecovery />);
    // Chinese title
    expect(await findByText("密钥库不可读")).toBeTruthy();
  });
});
