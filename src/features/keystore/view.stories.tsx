import type { Meta, StoryObj } from "@storybook/react-vite";
import KeystoreRecoveryView from "./view";
import type { KeystoreController } from "./controller";

const noop = () => {};
const fake = (over: Record<string, unknown>): KeystoreController =>
  ({
    state: "healthy",
    reason: "",
    resetOpen: false,
    busy: null,
    toasts: [],
    archive: noop,
    reset: noop,
    openReset: noop,
    closeReset: noop,
    dismissToast: noop,
    ...over,
  }) as unknown as KeystoreController;

const meta: Meta<typeof KeystoreRecoveryView> = {
  title: "Settings/Keystore Recovery",
  component: KeystoreRecoveryView,
};
export default meta;

export const Healthy: StoryObj<typeof KeystoreRecoveryView> = { args: { c: fake({}) } };
export const Corrupt: StoryObj<typeof KeystoreRecoveryView> = {
  args: { c: fake({ state: "corrupt", reason: "bad header magic" }) },
};
export const Archiving: StoryObj<typeof KeystoreRecoveryView> = {
  args: { c: fake({ state: "corrupt", reason: "bad header magic", busy: "archive" }) },
};
export const Archived: StoryObj<typeof KeystoreRecoveryView> = {
  args: { c: fake({ state: "archived" }) },
};
export const ResetConfirm: StoryObj<typeof KeystoreRecoveryView> = {
  args: { c: fake({ state: "corrupt", reason: "x", resetOpen: true }) },
};
export const ResetFailed: StoryObj<typeof KeystoreRecoveryView> = {
  args: {
    c: fake({
      state: "corrupt",
      reason: "x",
      toasts: [{ id: 1, variant: "destructive", message: "Reset failed: io error" }],
    }),
  },
};
export const Dark: StoryObj<typeof KeystoreRecoveryView> = {
  parameters: { colorScheme: "dark", backgrounds: { default: "dark" } },
  args: { c: fake({ state: "corrupt", reason: "bad header" }) },
};
