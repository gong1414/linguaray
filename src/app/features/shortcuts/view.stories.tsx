import type { Meta, StoryObj } from "@storybook/react-vite";
import { MantineProvider } from "@mantine/core";
import { linguaTheme } from "../../ui/theme";
import ShortcutsView from "./view";
import type { ShortcutsController } from "./controller";
import type { ShortcutSnapshot } from "./model";

const noop = () => {};
const snap = (combo?: string): ShortcutSnapshot => ({
  revision: 5,
  entries: [
    { action: "translate_selection", combo: combo ?? "Alt+Space", available: true, registration_state: "registered", registration_error: null },
    { action: "translate_input", combo: "Ctrl+Space", available: true, registration_state: "registered", registration_error: null },
    { action: "translate_clipboard", combo: "Ctrl+Alt+Space", available: true, registration_state: "registered", registration_error: null },
    { action: "ocr_translate", combo: "Alt+Shift+Space", available: true, registration_state: "registered", registration_error: null },
  ],
});

const fake = (over: Record<string, unknown>): ShortcutsController =>
  ({
    snapshot: snap(),
    loadError: false,
    recordingAction: null,
    recordedCombo: "",
    conflict: null,
    busy: null,
    resetOpen: false,
    localFailures: {},
    operationError: null,
    differsFromDefaults: false,
    retryLoad: noop,
    change: noop,
    cancelRecording: noop,
    onRecorderKeyDown: noop,
    overrideConflict: noop,
    openReset: noop,
    closeReset: noop,
    reset: noop,
    ...over,
  }) as unknown as ShortcutsController;

const meta: Meta<typeof ShortcutsView> = { title: "Settings/Shortcuts", component: ShortcutsView };
export default meta;

export const Default: StoryObj<typeof ShortcutsView> = { args: { c: fake({}) } };
export const Loading: StoryObj<typeof ShortcutsView> = { args: { c: fake({ snapshot: null }) } };
export const LoadError: StoryObj<typeof ShortcutsView> = { args: { c: fake({ loadError: true, snapshot: null }) } };
export const Recording: StoryObj<typeof ShortcutsView> = {
  args: { c: fake({ recordingAction: "translate_selection", recordedCombo: "Ctrl+Alt+K" }) },
};
export const Conflict: StoryObj<typeof ShortcutsView> = {
  args: {
    c: fake({
      recordingAction: "translate_selection",
      recordedCombo: "Ctrl+Space",
      conflict: { action: "translate_selection", otherAction: "translate_input", combo: "Ctrl+Space" },
    }),
  },
};
export const RegistrationFailed: StoryObj<typeof ShortcutsView> = {
  args: { c: fake({ localFailures: { translate_selection: true } }) },
};
export const Customized: StoryObj<typeof ShortcutsView> = {
  args: { c: fake({ snapshot: snap("Ctrl+Alt+K"), differsFromDefaults: true }) },
};
export const ResetOpen: StoryObj<typeof ShortcutsView> = {
  args: { c: fake({ snapshot: snap("Ctrl+Alt+K"), differsFromDefaults: true, resetOpen: true }) },
};
export const SaveFailed: StoryObj<typeof ShortcutsView> = {
  args: { c: fake({ operationError: "save" }) },
};
export const Dark: StoryObj<typeof ShortcutsView> = {
  parameters: { backgrounds: { default: "dark" } },
  decorators: [
    (Story) => (
      <MantineProvider theme={linguaTheme} forceColorScheme="dark">
        <Story />
      </MantineProvider>
    ),
  ],
  args: { c: fake({ recordingAction: "ocr_translate", recordedCombo: "⌥⇧" }) },
};
