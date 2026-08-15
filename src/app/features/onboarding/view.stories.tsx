import type { Meta, StoryObj } from "@storybook/react-vite";
import { MantineProvider } from "@mantine/core";
import { linguaTheme } from "../../ui/theme";
import { OnboardingView, type OnboardingViewProps } from "./view";
import { STEP_ORDER } from "./model";
/**
 * Production OnboardingView (pure props) across the required state matrix
 * (default / loading / empty / populated / error / disabled / long CJK /
 * narrow / dark). The real controller+ipc composition is covered by the
 * vitest integration test (controller.test.tsx).
 */

const base: OnboardingViewProps = {
  step: "welcome",
  locale: "en",
  a11y: "granted",
  screenCapture: "granted",
  providerCount: 1,
  historyBusy: false,
  shortcuts: [],
  advancing: false,
  error: null,
  onOpenA11ySettings: () => {},
  onOpenScreenCaptureSettings: () => {},
  onRecheckPermissions: () => {},
  onOpenProviderSettings: () => {},
  onOpenShortcutsSettings: () => {},
  onEnableHistory: () => {},
  onAdvance: () => {},
  onFinish: () => {},
};

const SHORTCUTS = [
  { action: "translate_selection", combo: "Alt+D" },
  { action: "translate_input", combo: "Alt+I" },
  { action: "translate_clipboard", combo: "Alt+C" },
  { action: "ocr_translate", combo: "Alt+O" },
];

const meta: Meta<typeof OnboardingView> = {
  title: "Windows/Onboarding",
  component: OnboardingView,
  parameters: { viewport: { defaultViewport: "onboarding600" }, viewMode: "story" },
};
export default meta;

export const Welcome: StoryObj<typeof OnboardingView> = { args: base };

export const PermissionsLoading: StoryObj<typeof OnboardingView> = {
  args: { ...base, step: "accessibility", a11y: "checking", screenCapture: "checking" },
};

export const PermissionsMissing: StoryObj<typeof OnboardingView> = {
  args: { ...base, step: "accessibility", a11y: "denied", screenCapture: "denied" },
};

export const PermissionsGranted: StoryObj<typeof OnboardingView> = {
  args: { ...base, step: "accessibility" },
};

export const ProviderChecking: StoryObj<typeof OnboardingView> = {
  args: { ...base, step: "provider", providerCount: null },
};

export const ProviderEmpty: StoryObj<typeof OnboardingView> = {
  args: { ...base, step: "provider", providerCount: 0 },
};

export const ProviderPopulated: StoryObj<typeof OnboardingView> = {
  args: { ...base, step: "provider", providerCount: 3 },
};

export const HistoryBusy: StoryObj<typeof OnboardingView> = {
  args: { ...base, step: "history", historyBusy: true },
};

export const ShortcutsPopulated: StoryObj<typeof OnboardingView> = {
  args: { ...base, step: "shortcuts", shortcuts: SHORTCUTS },
};

export const Done: StoryObj<typeof OnboardingView> = {
  args: { ...base, step: "done" },
};

export const ErrorState: StoryObj<typeof OnboardingView> = {
  args: { ...base, step: "accessibility", error: "onboarding_next: database locked" },
};

export const DisabledAdvancing: StoryObj<typeof OnboardingView> = {
  args: { ...base, step: "provider", advancing: true },
};

export const LongChinese: StoryObj<typeof OnboardingView> = {
  args: {
    ...base,
    locale: "zh",
    step: "shortcuts",
    shortcuts: SHORTCUTS.map((s, i) => ({
      ...s,
      combo: `${s.combo}+${"⌘".repeat(i + 1)}`,
    })),
  },
};

export const Dark: StoryObj<typeof OnboardingView> = {
  parameters: { backgrounds: { default: "dark" } },
  decorators: [
    (Story) => (
      <MantineProvider theme={linguaTheme} forceColorScheme="dark">
        <Story />
      </MantineProvider>
    ),
  ],
  args: { ...base, step: "accessibility", a11y: "denied" },
};

// Narrow-window: onboarding's real stage is a fixed 600×400 window; the
// narrow variant renders at 360px to prove no horizontal squeeze/overflow.
export const Narrow: StoryObj<typeof OnboardingView> = {
  parameters: { viewport: { defaultViewport: "mobile1" } },
  args: { ...base, step: "accessibility", a11y: "denied" },
};

// Step-machine sweep: one story per step keeps visual baselines granular.
export const EveryStep: StoryObj<typeof OnboardingView> = {
  render: (args) => (
    <div style={{ display: "flex", flexDirection: "column", gap: 24 }}>
      {STEP_ORDER.map((step) => (
        <div key={step}>
          <div style={{ fontSize: 12, opacity: 0.6, marginBottom: 4 }}>{step}</div>
          <OnboardingView {...args} step={step} />
        </div>
      ))}
    </div>
  ),
  args: { ...base, locale: "zh" },
};
