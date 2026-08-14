import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@solidjs/testing-library";

const { invokeMock, localeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(async (_command: string, _args?: unknown): Promise<unknown> => undefined),
  localeMock: { current: "en" as "en" | "zh" },
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));
vi.mock("../src/i18n", () => ({ detectLocale: () => localeMock.current }));

import Shortcuts, { canonicalCombo } from "../src/features/settings/Shortcuts";
import type { ShortcutSnapshot } from "../src/features/settings/shortcut-types";

const snapshot = (over: Partial<ShortcutSnapshot> = {}): ShortcutSnapshot => ({
  revision: 3,
  entries: [
    { action: "translate_selection", combo: "Alt+Space", available: true, registration_state: "registered", registration_error: null },
    { action: "translate_input", combo: "Ctrl+Space", available: true, registration_state: "registered", registration_error: null },
    { action: "translate_clipboard", combo: "Ctrl+Alt+Space", available: true, registration_state: "registered", registration_error: null },
    { action: "ocr_translate", combo: "Alt+Shift+Space", available: true, registration_state: "registered", registration_error: null },
  ],
  ...over,
});

function route(commands: Record<string, (args?: unknown) => unknown>): void {
  invokeMock.mockImplementation(async (command: string, args?: unknown) => {
    const handler = commands[command];
    if (!handler) throw new Error(`unexpected invoke ${command}`);
    return handler(args);
  });
}

beforeEach(() => {
  localeMock.current = "en";
  invokeMock.mockReset();
  route({ shortcut_list: () => snapshot() });
});
afterEach(() => cleanup());

describe("Shortcuts", () => {
  it("canonicalizes modifiers in the frozen order and ignores modifier-only/repeat", () => {
    const keyboard = (over: Partial<KeyboardEvent>) => ({
      key: "k", code: "KeyK", ctrlKey: false, altKey: false, shiftKey: false,
      metaKey: false, repeat: false, isComposing: false, ...over,
    }) as KeyboardEvent;
    expect(canonicalCombo(keyboard({ ctrlKey: true, altKey: true, shiftKey: true, metaKey: true })))
      .toBe("Ctrl+Alt+Shift+Super+K");
    expect(canonicalCombo(keyboard({ key: "Shift", code: "ShiftLeft", shiftKey: true }))).toBeNull();
    expect(canonicalCombo(keyboard({ repeat: true }))).toBeNull();
    expect(canonicalCombo(keyboard({}))).toBeNull();
  });

  it("renders the four frozen actions and OCR is changeable", async () => {
    render(() => <Shortcuts />);
    await waitFor(() => expect(screen.getByText("Translate Selection")).toBeTruthy());
    expect(screen.getByText("Translate Input")).toBeTruthy();
    expect(screen.getByText("Translate Clipboard")).toBeTruthy();
    expect(screen.getByText("OCR Translate")).toBeTruthy();
    expect(screen.queryByText("Available in R5")).toBeNull();
    expect(screen.getByRole("button", { name: "Change OCR Translate" })).not.toBeDisabled();
  });

  it("records a canonical combo, checks conflict, and saves with expected revision", async () => {
    const saved = snapshot({
      revision: 4,
      entries: snapshot().entries.map((entry) =>
        entry.action === "translate_selection" ? { ...entry, combo: "Ctrl+Shift+K" } : entry,
      ),
    });
    route({
      shortcut_list: () => snapshot(),
      shortcut_recording_begin: () => undefined,
      shortcut_check_conflict: () => null,
      shortcut_save: () => saved,
      shortcut_recording_end: () => undefined,
    });
    render(() => <Shortcuts />);
    await waitFor(() => expect(screen.getByText("Translate Selection")).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { name: "Change Translate Selection" }));
    const recorder = await screen.findByRole("button", { name: "Press a key combo…" });
    fireEvent.keyDown(recorder, { key: "k", code: "KeyK", ctrlKey: true, shiftKey: true });
    await waitFor(() => expect(screen.getByText("Ctrl+Shift+K")).toBeTruthy());
    expect(invokeMock).toHaveBeenCalledWith("shortcut_check_conflict", {
      action: "translate_selection",
      combo: "Ctrl+Shift+K",
      revision: 3,
    });
    expect(invokeMock).toHaveBeenCalledWith("shortcut_save", {
      action: "translate_selection",
      combo: "Ctrl+Shift+K",
      expectedRevision: 3,
      overrideAction: null,
    });
  });

  it("shows conflict and Override swaps the two mappings", async () => {
    route({
      shortcut_list: () => snapshot(),
      shortcut_recording_begin: () => undefined,
      shortcut_check_conflict: () => "translate_input",
      shortcut_save: (args) => {
        expect(args).toEqual({
          action: "translate_selection",
          combo: "Ctrl+Space",
          expectedRevision: 3,
          overrideAction: "translate_input",
        });
        return snapshot({ revision: 4 });
      },
      shortcut_recording_end: () => undefined,
    });
    render(() => <Shortcuts />);
    await waitFor(() => expect(screen.getByText("Translate Selection")).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { name: "Change Translate Selection" }));
    fireEvent.keyDown(await screen.findByRole("button", { name: "Press a key combo…" }), {
      key: " ", code: "Space", ctrlKey: true,
    });
    await screen.findByText("Conflicts with Translate Input");
    fireEvent.click(screen.getByRole("button", { name: "Override" }));
    await waitFor(() => expect(invokeMock).toHaveBeenCalledWith("shortcut_save", expect.anything()));
  });

  it("Escape cancels recording and ends native recording mode", async () => {
    route({
      shortcut_list: () => snapshot(),
      shortcut_recording_begin: () => undefined,
      shortcut_recording_end: () => undefined,
    });
    render(() => <Shortcuts />);
    await waitFor(() => expect(screen.getByText("Translate Selection")).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { name: "Change Translate Selection" }));
    fireEvent.keyDown(await screen.findByRole("button", { name: "Press a key combo…" }), { key: "Escape" });
    await waitFor(() => expect(screen.queryByRole("button", { name: "Press a key combo…" })).toBeNull());
    expect(invokeMock).toHaveBeenCalledWith("shortcut_recording_end");
    await waitFor(() =>
      expect(document.activeElement).toBe(
        screen.getByRole("button", { name: "Change Translate Selection" }),
      ),
    );
  });

  it("Cancel invalidates a late conflict response", async () => {
    let resolveConflict!: (value: "translate_input") => void;
    const deferred = new Promise<"translate_input">((resolve) => { resolveConflict = resolve; });
    route({
      shortcut_list: () => snapshot(),
      shortcut_recording_begin: () => undefined,
      shortcut_check_conflict: () => deferred,
      shortcut_recording_end: () => undefined,
    });
    render(() => <Shortcuts />);
    await waitFor(() => expect(screen.getByText("Translate Selection")).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { name: "Change Translate Selection" }));
    const recorder = await screen.findByRole("button", { name: "Press a key combo…" });
    fireEvent.keyDown(recorder, { key: "k", code: "KeyK", ctrlKey: true });
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    resolveConflict("translate_input");
    await Promise.resolve();
    expect(screen.queryByText("Conflicts with Translate Input")).toBeNull();
    expect(invokeMock.mock.calls.some((call) => call[0] === "shortcut_save")).toBe(false);
  });

  it("resets a customized map only after confirmation using the current revision", async () => {
    const custom = snapshot({
      revision: 9,
      entries: snapshot().entries.map((entry) =>
        entry.action === "translate_selection" ? { ...entry, combo: "Ctrl+Shift+K" } : entry,
      ),
    });
    route({
      shortcut_list: () => custom,
      shortcut_reset_defaults: () => snapshot({ revision: 10 }),
    });
    render(() => <Shortcuts />);
    await waitFor(() => expect(screen.getByText("Ctrl+Shift+K")).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { name: "Reset to Defaults" }));
    await screen.findByText("Reset keyboard shortcuts?");
    fireEvent.click(screen.getByRole("button", { name: "Use Defaults" }));
    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith("shortcut_reset_defaults", { expectedRevision: 9 }),
    );
  });

  it("shows a load error and Retry reloads the snapshot", async () => {
    let attempts = 0;
    route({
      shortcut_list: () => {
        attempts += 1;
        if (attempts === 1) throw new Error("offline");
        return snapshot();
      },
    });
    render(() => <Shortcuts />);
    await screen.findByText("Shortcuts couldn't be loaded.");
    fireEvent.click(screen.getByRole("button", { name: "Retry" }));
    await waitFor(() => expect(screen.getByText("Translate Selection")).toBeTruthy());
    expect(attempts).toBe(2);
  });

  it("surfaces a registration failure and keeps the old mapping", async () => {
    route({
      shortcut_list: () => snapshot(),
      shortcut_recording_begin: () => undefined,
      shortcut_check_conflict: () => null,
      shortcut_save: () => Promise.reject({ error: "registration_failed", message: "system reserved" }),
      shortcut_recording_end: () => undefined,
    });
    render(() => <Shortcuts />);
    await waitFor(() => expect(screen.getByText("Translate Selection")).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { name: "Change Translate Selection" }));
    fireEvent.keyDown(await screen.findByRole("button", { name: "Press a key combo…" }), {
      key: "q", code: "KeyQ", metaKey: true,
    });
    await screen.findByText("This combo couldn't be registered (system reserved)");
    expect(screen.getByText("Alt+Space")).toBeTruthy();
  });

  it("reloads after a stale-revision save so the next edit cannot loop on old CAS", async () => {
    let loads = 0;
    route({
      shortcut_list: () => {
        loads += 1;
        return snapshot({ revision: loads === 1 ? 3 : 4 });
      },
      shortcut_recording_begin: () => undefined,
      shortcut_check_conflict: () => null,
      shortcut_save: () => Promise.reject({ error: "stale_revision", expected: 3, actual: 4 }),
      shortcut_recording_end: () => undefined,
    });
    render(() => <Shortcuts />);
    await waitFor(() => expect(screen.getByText("Translate Selection")).toBeTruthy());
    fireEvent.click(screen.getByRole("button", { name: "Change Translate Selection" }));
    fireEvent.keyDown(await screen.findByRole("button", { name: "Press a key combo…" }), {
      key: "k", code: "KeyK", ctrlKey: true,
    });
    await waitFor(() => expect(loads).toBe(2));
    expect(screen.queryByText("The shortcut couldn't be saved. Try again.")).toBeNull();
  });

  it("renders Chinese copy and has no axe violations", async () => {
    localeMock.current = "zh";
    render(() => <Shortcuts />);
    await waitFor(() => expect(screen.getByText("键盘快捷键")).toBeTruthy());
    const { assertNoAxeViolations } = await import("./axe");
    await assertNoAxeViolations({ disableRules: ["color-contrast"] });
  });
});
