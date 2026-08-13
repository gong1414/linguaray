import { afterEach, describe, expect, it, vi } from "vitest";
import { cleanup, render } from "@solidjs/testing-library";
import { assertNoAxeViolations } from "./axe";

const { localeMock } = vi.hoisted(() => ({
  localeMock: { current: "en" as "en" | "zh" },
}));
vi.mock("../src/i18n", () => ({ detectLocale: () => localeMock.current }));

import {
  ShortcutsView,
  type ShortcutsViewProps,
} from "../src/features/settings/Shortcuts";
import type { ShortcutSnapshot } from "../src/features/settings/shortcut-types";

const snapshot = (registrationFailed = false): ShortcutSnapshot => ({
  revision: 1,
  entries: [
    {
      action: "translate_selection",
      combo: "Alt+Space",
      available: true,
      registration_state: registrationFailed ? "registration_failed" : "registered",
      registration_error: registrationFailed ? "system reserved" : null,
    },
    { action: "translate_input", combo: "Ctrl+Space", available: true, registration_state: "registered", registration_error: null },
    { action: "translate_clipboard", combo: "Ctrl+Alt+Space", available: true, registration_state: "registered", registration_error: null },
    { action: "ocr_translate", combo: "Alt+Shift+Space", available: false, registration_state: "unavailable", registration_error: null },
  ],
});

const noop = () => {};
const props = (over: Partial<ShortcutsViewProps> = {}): ShortcutsViewProps => ({
  snapshot: snapshot(),
  loadError: false,
  recordingAction: null,
  recordedCombo: "",
  conflict: null,
  busy: null,
  resetOpen: false,
  localRegistrationFailures: {},
  operationError: null,
  onRetryLoad: noop,
  onChange: noop,
  onCancelRecording: noop,
  onRecorderKeyDown: noop,
  onOverride: noop,
  onOpenReset: noop,
  onCloseReset: noop,
  onReset: noop,
  ...over,
});

afterEach(() => {
  cleanup();
  localeMock.current = "en";
  document.documentElement.dataset.theme = "light";
});

describe("ShortcutsView accessibility", () => {
  it("has no axe violations in Default", async () => {
    render(() => <ShortcutsView {...props()} />);
    await assertNoAxeViolations({ disableRules: ["color-contrast"] });
  });

  it("has no axe violations in Recording", async () => {
    render(() => (
      <ShortcutsView
        {...props({ recordingAction: "translate_selection", recordedCombo: "" })}
      />
    ));
    await assertNoAxeViolations({ disableRules: ["color-contrast"] });
  });

  it("has no axe violations in Conflict", async () => {
    render(() => (
      <ShortcutsView
        {...props({
          recordingAction: "translate_selection",
          recordedCombo: "Ctrl+Space",
          conflict: {
            action: "translate_selection",
            otherAction: "translate_input",
            combo: "Ctrl+Space",
          },
        })}
      />
    ));
    await assertNoAxeViolations({ disableRules: ["color-contrast"] });
  });

  it("has no axe violations in Registration failed, dark Chinese", async () => {
    localeMock.current = "zh";
    document.documentElement.dataset.theme = "dark";
    render(() => <ShortcutsView {...props({ snapshot: snapshot(true) })} />);
    expect(document.querySelector(".inline-error--warning")).not.toBeNull();
    await assertNoAxeViolations({ disableRules: ["color-contrast"] });
  });
});
