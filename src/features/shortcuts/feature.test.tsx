import { render, screen, fireEvent, cleanup } from "@testing-library/react";
import { waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { AppProviders } from "../../app/providers";
import { ShortcutsView } from "./view";
import { useShortcutsController } from "./controller";
import { canonicalCombo } from "./model";

const { ipc } = vi.hoisted(() => ({
  ipc: {
    shortcutList: vi.fn(),
    shortcutCheckConflict: vi.fn(),
    shortcutSave: vi.fn(),
    shortcutResetDefaults: vi.fn(),
    shortcutRecordingBegin: vi.fn(),
    shortcutRecordingEnd: vi.fn(),
  },
}));
vi.mock("./ipc", () => ipc);

const snap = (combo?: string, rev = 5) => ({
  revision: rev,
  entries: [
    { action: "translate_selection", combo: combo ?? "Alt+Space", available: true, registration_state: "registered", registration_error: null },
    { action: "translate_input", combo: "Ctrl+Space", available: true, registration_state: "registered", registration_error: null },
    { action: "translate_clipboard", combo: "Ctrl+Alt+Space", available: true, registration_state: "registered", registration_error: null },
    { action: "ocr_translate", combo: "Alt+Shift+Space", available: true, registration_state: "registered", registration_error: null },
  ],
});

beforeEach(() => {
  vi.clearAllMocks();
  ipc.shortcutList.mockResolvedValue(snap());
  ipc.shortcutRecordingBegin.mockResolvedValue(undefined);
  ipc.shortcutRecordingEnd.mockResolvedValue(undefined);
  ipc.shortcutCheckConflict.mockResolvedValue(null);
  ipc.shortcutSave.mockResolvedValue(snap("Ctrl+Alt+K"));
});

afterEach(cleanup);

function Live() {
  const c = useShortcutsController();
  return <ShortcutsView c={c} />;
}

describe("canonicalCombo", () => {
  const key = (over: Partial<KeyboardEvent> = {}) =>
    canonicalCombo({
      key: "D",
      code: "KeyD",
      ctrlKey: false,
      altKey: true,
      shiftKey: false,
      metaKey: false,
      ...over,
    } as KeyboardEvent);

  it("builds the frozen modifier order", () => {
    expect(key({ ctrlKey: true, altKey: true, shiftKey: true, metaKey: true })).toBe(
      "Ctrl+Alt+Shift+Super+D",
    );
    expect(key()).toBe("Alt+D");
    expect(key({ key: "3", code: "Digit3", altKey: true })).toBe("Alt+3");
  });

  it("rejects bare keys and modifier-only holds and repeats", () => {
    expect(key({ altKey: false })).toBeNull();
    expect(key({ key: "Alt", code: "AltLeft" })).toBeNull();
    expect(key({ repeat: true })).toBeNull();
  });
});

describe("useShortcutsController (integration with view)", () => {
  it("loads the snapshot and renders one row per action", async () => {
    render(<Live />, { wrapper: AppProviders });
    expect(await screen.findByText("Translate Selection")).toBeInTheDocument();
    expect(screen.getByTestId("shortcut-chip-translate_selection")).toHaveTextContent("Alt+Space");
  });

  it("record → conflict → override saves with overrideAction", async () => {
    ipc.shortcutSave.mockResolvedValue(snap("Ctrl+Alt+K"));
    ipc.shortcutCheckConflict.mockResolvedValueOnce("translate_input");
    render(<Live />, { wrapper: AppProviders });
    await screen.findByText("Translate Selection");
    fireEvent.click(screen.getByTestId("shortcuts-change-translate_selection"));
    const recorder = await screen.findByRole("button", { name: "Press a key combo…" });
    fireEvent.keyDown(recorder, {
      key: "K",
      code: "KeyK",
      ctrlKey: true,
      altKey: true,
    });
    expect(await screen.findByTestId("shortcut-conflict-translate_selection")).toHaveTextContent(
      "Conflicts with Translate Input",
    );
    fireEvent.click(screen.getByRole("button", { name: "Override" }));
    await waitFor(() =>
      expect(ipc.shortcutSave).toHaveBeenCalledWith(
        "translate_selection",
        "Ctrl+Alt+K",
        5,
        "translate_input",
      ),
    );
  });

  it("same combo cancels; Escape cancels; native end is paired", async () => {
    render(<Live />, { wrapper: AppProviders });
    await screen.findByText("Translate Selection");
    const changeBtn = screen.getAllByRole("button", { name: /Change Translate Selection/ })[0];
    fireEvent.click(changeBtn);
    const recorder = await screen.findByRole("button", { name: "Press a key combo…" });
    // Same combo → cancel (no conflict check).
    fireEvent.keyDown(recorder, { key: "Space", code: "Space", altKey: true });
    await waitFor(() =>
      expect(screen.queryByRole("button", { name: "Press a key combo…" })).toBeNull(),
    );
    expect(ipc.shortcutCheckConflict).not.toHaveBeenCalled();

    // Escape path also ends native recording.
    fireEvent.click(screen.getAllByRole("button", { name: /Change Translate Selection/ })[0]);
    const r2 = await screen.findByRole("button", { name: "Press a key combo…" });
    fireEvent.keyDown(r2, { key: "Escape", code: "Escape" });
    await waitFor(() => expect(ipc.shortcutRecordingEnd).toHaveBeenCalled());
  });

  it("registration failure marks the row without touching the snapshot", async () => {
    ipc.shortcutSave.mockRejectedValue({ error: "registration_failed" });
    render(<Live />, { wrapper: AppProviders });
    await screen.findByText("Translate Selection");
    fireEvent.click(screen.getAllByRole("button", { name: /Change Translate Selection/ })[0]);
    const recorder = await screen.findByRole("button", { name: "Press a key combo…" });
    fireEvent.keyDown(recorder, { key: "K", code: "KeyK", ctrlKey: true, altKey: true });
    expect(await screen.findByTestId("shortcut-regfail-translate_selection")).toBeInTheDocument();
  });

  it("stale_revision triggers a reload", async () => {
    ipc.shortcutSave.mockRejectedValue({ error: "stale_revision" });
    render(<Live />, { wrapper: AppProviders });
    await screen.findByText("Translate Selection");
    fireEvent.click(screen.getAllByRole("button", { name: /Change Translate Selection/ })[0]);
    const recorder = await screen.findByRole("button", { name: "Press a key combo…" });
    fireEvent.keyDown(recorder, { key: "K", code: "KeyK", ctrlKey: true, altKey: true });
    await waitFor(() => expect(ipc.shortcutList).toHaveBeenCalledTimes(2));
  });

  it("reset defaults requires a diff and flows through the modal", async () => {
    ipc.shortcutList.mockResolvedValue(snap("Ctrl+Alt+K"));
    ipc.shortcutResetDefaults.mockResolvedValue(snap());
    render(<Live />, { wrapper: AppProviders });
    const trigger = await screen.findByTestId("shortcuts-reset-trigger");
    expect(trigger).toBeEnabled();
    fireEvent.click(trigger);
    fireEvent.click(await screen.findByTestId("shortcuts-reset-confirm"));
    await waitFor(() =>
      expect(ipc.shortcutResetDefaults).toHaveBeenCalledWith(5),
    );
  });

  it("unmount pairs recording_end (no leak)", async () => {
    const { unmount } = render(<Live />, { wrapper: AppProviders });
    await screen.findByText("Translate Selection");
    fireEvent.click(screen.getAllByRole("button", { name: /Change Translate Selection/ })[0]);
    await screen.findByRole("button", { name: "Press a key combo…" });
    unmount();
    await waitFor(() => expect(ipc.shortcutRecordingEnd).toHaveBeenCalledTimes(1));
  });
});
