import { beforeEach, describe, expect, it, vi } from "vitest";

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(async (_command: string, _args?: unknown): Promise<unknown> => undefined),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));

import {
  shortcutCheckConflict,
  shortcutList,
  shortcutRecordingBegin,
  shortcutRecordingEnd,
  shortcutResetDefaults,
  shortcutSave,
} from "./shortcut-ipc";

describe("shortcut IPC wrappers", () => {
  beforeEach(() => invokeMock.mockReset());

  it("uses the frozen command names and camelCase revision argument", async () => {
    await shortcutList();
    await shortcutCheckConflict("translate_selection", "Ctrl+Space", 6);
    await shortcutSave("translate_selection", "Ctrl+Space", 7, "translate_input");
    await shortcutResetDefaults(8);
    await shortcutRecordingBegin("translate_selection");
    await shortcutRecordingEnd();

    expect(invokeMock.mock.calls).toEqual([
      ["shortcut_list"],
      ["shortcut_check_conflict", { action: "translate_selection", combo: "Ctrl+Space", revision: 6 }],
      ["shortcut_save", {
        action: "translate_selection",
        combo: "Ctrl+Space",
        expectedRevision: 7,
        overrideAction: "translate_input",
      }],
      ["shortcut_reset_defaults", { expectedRevision: 8 }],
      ["shortcut_recording_begin", { action: "translate_selection" }],
      ["shortcut_recording_end"],
    ]);
  });
});
