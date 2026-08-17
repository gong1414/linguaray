import type { Meta, StoryObj } from "@storybook/react-vite";
import UpdaterPanelView from "./view";
import type { UpdaterController } from "./controller";
import type { AvailableUpdate } from "./model";

const noop = () => {};
const update: AvailableUpdate = { state: "available", current: "0.1.0", next: "0.2.0", notes: "• Faster popup\n• OCR fixes" };
const fake = (phase: UpdaterController["phase"], over: Record<string, unknown> = {}): UpdaterController =>
  ({
    phase,
    autoCheck: true,
    autoCheckError: null,
    check: noop,
    install: noop,
    relaunch: noop,
    toggleAutoCheck: noop,
    ...over,
  }) as unknown as UpdaterController;

const meta: Meta<typeof UpdaterPanelView> = { title: "Settings/Updater", component: UpdaterPanelView };
export default meta;

export const Checking: StoryObj<typeof UpdaterPanelView> = { args: { c: fake({ kind: "checking" }) } };
export const UpToDate: StoryObj<typeof UpdaterPanelView> = {
  args: { c: fake({ kind: "upToDate", version: "0.1.0" }) },
};
export const Available: StoryObj<typeof UpdaterPanelView> = {
  args: { c: fake({ kind: "available", update }) },
};
export const Downloading: StoryObj<typeof UpdaterPanelView> = {
  args: { c: fake({ kind: "downloading", update, percent: 45, downloaded: 45000 }) },
};
export const DownloadingUnknownSize: StoryObj<typeof UpdaterPanelView> = {
  args: { c: fake({ kind: "downloading", update, percent: null, downloaded: 0 }) },
};
export const Installing: StoryObj<typeof UpdaterPanelView> = {
  args: { c: fake({ kind: "installing", update }) },
};
export const ReadyToRelaunch: StoryObj<typeof UpdaterPanelView> = {
  args: { c: fake({ kind: "readyToRelaunch", update }) },
};
export const ErrorState: StoryObj<typeof UpdaterPanelView> = {
  args: { c: fake({ kind: "error", message: "github unreachable" }) },
};
export const AutoCheckError: StoryObj<typeof UpdaterPanelView> = {
  args: { c: fake({ kind: "upToDate", version: "0.1.0" }, { autoCheckError: "db locked" }) },
};
export const Dark: StoryObj<typeof UpdaterPanelView> = {
  parameters: { colorScheme: "dark", backgrounds: { default: "dark" } },
  args: { c: fake({ kind: "available", update }) },
};
