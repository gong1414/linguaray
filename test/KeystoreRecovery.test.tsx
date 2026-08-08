import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, fireEvent, cleanup, waitFor, screen } from "@solidjs/testing-library";

// vi.hoisted lets us reference the mock inside vi.mock factories (which are
// hoisted above top-level declarations).
const { invokeMock, localeMock } = vi.hoisted(() => ({
  // The mock is typed loosely (unknown return) so per-test
  // mockResolvedValueOnce can return strings, null, etc. without friction.
  invokeMock: vi.fn(async (_cmd: string, _args?: unknown): Promise<unknown> => {
    throw new Error(`unexpected invoke ${_cmd}`);
  }),
  localeMock: { current: "en" as "en" | "zh" },
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));
vi.mock("../src/i18n", () => ({ detectLocale: () => localeMock.current }));

import KeystoreRecovery from "../src/features/settings/KeystoreRecovery";

const flush = () => new Promise((r) => setTimeout(r, 0));

beforeEach(() => {
  localeMock.current = "en";
  // mockReset clears the once-stack AND implementations, so per-test
  // mockResolvedValueOnce ordering is deterministic.
  invokeMock.mockReset();
  invokeMock.mockImplementation(async (cmd: string) => {
    if (cmd === "keystore_health") return "";
    throw new Error(`unexpected invoke ${cmd}`);
  });
});

afterEach(() => cleanup());

describe("KeystoreRecovery (Surface 06)", () => {
  it("on mount, calls keystore_health once", async () => {
    render(() => <KeystoreRecovery />);
    await waitFor(() => expect(invokeMock).toHaveBeenCalledWith("keystore_health"));
  });

  it("healthy: renders no alert banner", async () => {
    const { container } = render(() => <KeystoreRecovery />);
    await waitFor(() => expect(invokeMock).toHaveBeenCalledWith("keystore_health"));
    await flush();
    expect(container.querySelector('[role="alert"]')).toBeNull();
    expect(container.querySelector('[role="status"]')).toBeNull();
  });

  it("corrupt: renders destructive banner with the reason and two buttons", async () => {
    invokeMock.mockResolvedValueOnce("corrupt: header mismatch");
    render(() => <KeystoreRecovery />);
    // title
    expect(await screen.findByText("Keystore unreadable")).toBeTruthy();
    // description interpolates {reason}
    expect(screen.getByText(/corrupt: header mismatch/)).toBeTruthy();
    // two action buttons
    expect(screen.getByText("Archive & re-enter")).toBeTruthy();
    expect(screen.getByText("Reset")).toBeTruthy();
  });

  it("corrupt to Archive: calls archive_keystore, transitions to archived state", async () => {
    invokeMock.mockResolvedValueOnce("corrupt: header mismatch");
    invokeMock.mockResolvedValueOnce("/path/to/archive"); // archive_keystore
    render(() => <KeystoreRecovery />);
    expect(await screen.findByText("Keystore unreadable")).toBeTruthy();
    fireEvent.click(screen.getByText("Archive & re-enter"));
    expect(await screen.findByText("Keys archived")).toBeTruthy();
    expect(screen.getByText("Enter your keys again")).toBeTruthy();
    expect(invokeMock).toHaveBeenCalledWith("archive_keystore");
  });

  it("corrupt to Reset: opens destructive Confirm; initial focus on Cancel", async () => {
    invokeMock.mockResolvedValueOnce("corrupt: header mismatch");
    render(() => <KeystoreRecovery />);
    expect(await screen.findByText("Keystore unreadable")).toBeTruthy();
    fireEvent.click(screen.getByText("Reset"));
    // The Confirm content is rendered into a portal (document.body); use screen.
    expect(await screen.findByText("Reset keystore?")).toBeTruthy();
    await flush(); // let Kobalte's onOpenAutoFocus settle
    const cancelBtn = screen.getByText("Cancel").closest("button") as HTMLButtonElement;
    expect(document.activeElement).toBe(cancelBtn);
  });

  it("reset Confirm to Cancel: stays corrupt (no reset_keystore call)", async () => {
    invokeMock.mockResolvedValueOnce("corrupt: header mismatch");
    render(() => <KeystoreRecovery />);
    expect(await screen.findByText("Keystore unreadable")).toBeTruthy();
    fireEvent.click(screen.getByText("Reset"));
    expect(await screen.findByText("Reset keystore?")).toBeTruthy();
    fireEvent.click(screen.getByText("Cancel"));
    await flush();
    // Cancel must not invoke reset_keystore, and the corrupt banner persists.
    expect(invokeMock).not.toHaveBeenCalledWith("reset_keystore");
    expect(screen.getByText("Keystore unreadable")).toBeTruthy();
  });

  it("reset Confirm to Confirm: calls reset_keystore, transitions to archived", async () => {
    invokeMock.mockResolvedValueOnce("corrupt: header mismatch");
    invokeMock.mockResolvedValueOnce(null); // reset_keystore returns Option<string>
    render(() => <KeystoreRecovery />);
    expect(await screen.findByText("Keystore unreadable")).toBeTruthy();
    fireEvent.click(screen.getByText("Reset"));
    expect(await screen.findByText("Reset keystore?")).toBeTruthy();
    // There are two "Reset" labels: the banner action and the dialog confirm.
    // Click the confirm one inside the dialog footer.
    const dialog = screen.getByText("Reset keystore?").closest(".lr-dialog__content");
    const confirmBtn = Array.from(dialog!.querySelectorAll("button")).find(
      (b) => b.textContent === "Reset",
    ) as HTMLButtonElement;
    fireEvent.click(confirmBtn);
    // Confirm calls reset_keystore and transitions to the archived banner.
    expect(await screen.findByText("Keys archived")).toBeTruthy();
    expect(invokeMock).toHaveBeenCalledWith("reset_keystore");
  });

  it("archive_keystore rejects: shows destructive toast", async () => {
    invokeMock.mockResolvedValueOnce("corrupt: header mismatch");
    invokeMock.mockRejectedValueOnce(new Error("io error")); // archive fails
    render(() => <KeystoreRecovery />);
    expect(await screen.findByText("Keystore unreadable")).toBeTruthy();
    fireEvent.click(screen.getByText("Archive & re-enter"));
    // destructive toast (role=alert) carries the failure message
    expect(await screen.findByText(/Archive failed|io error/)).toBeTruthy();
  });

  it("uses zh copy when locale is zh", async () => {
    localeMock.current = "zh";
    invokeMock.mockResolvedValueOnce("corrupt: header mismatch");
    render(() => <KeystoreRecovery />);
    // Chinese title
    expect(await screen.findByText("密钥库不可读")).toBeTruthy();
  });
});
