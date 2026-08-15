import type { Meta, StoryObj } from "@storybook/react-vite";
import { MantineProvider } from "@mantine/core";
import { linguaTheme } from "../../ui/theme";
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

const meta: Meta<typeof PrivacyView> = {
  title: "Settings/Privacy",
  component: PrivacyView,
};
export default meta;

export const Default: StoryObj<typeof PrivacyView> = { args: base };

export const Loading: StoryObj<typeof PrivacyView> = {
  args: { ...base, status: null, loading: true },
};

export const EmptyHistory: StoryObj<typeof PrivacyView> = {
  args: { ...base, status: { enabled: false, retention_days: 30, record_count: 0 } },
};

export const ErrorState: StoryObj<typeof PrivacyView> = {
  args: { ...base, status: null, error: "database: preferences table missing" },
};

export const BusyClearing: StoryObj<typeof PrivacyView> = {
  args: { ...base, busy: "clear" },
};

export const ConfirmOpen: StoryObj<typeof PrivacyView> = {
  args: { ...base, clearOpen: true },
};

export const ExternalEnabledTokenShown: StoryObj<typeof PrivacyView> = {
  args: {
    ...base,
    external: { state: "enabled", port: 8787 },
    tokenOnce: "lray_9f2b7c8e51a4",
  },
};

export const LongChinese: StoryObj<typeof PrivacyView> = {
  args: { ...base, locale: "zh", status: { enabled: false, retention_days: 90, record_count: 12345 } },
};

export const Dark: StoryObj<typeof PrivacyView> = {
  parameters: { backgrounds: { default: "dark" } },
  decorators: [
    (Story) => (
      <MantineProvider theme={linguaTheme} forceColorScheme="dark">
        <Story />
      </MantineProvider>
    ),
  ],
  args: { ...base, locale: "zh", external: { state: "enabled", port: 8787 }, tokenOnce: "lray_zh" },
};

export const Narrow: StoryObj<typeof PrivacyView> = {
  parameters: { viewport: { defaultViewport: "mobile1" } },
  args: { ...base, locale: "zh" },
};
