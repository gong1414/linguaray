import type { Meta, StoryObj } from "@storybook/react-vite";
import { FluentProvider, webDarkTheme } from "@fluentui/react-components";
import { SettingsShellView } from "./view";
import { PrivacyView } from "../privacy/view";
import { HistoryView } from "../history/view";

/**
 * Production settings shell composing REAL migrated sections (spec §八:
 * Storybook renders production compositions, not simplified mocks).
 */
const meta: Meta<typeof SettingsShellView> = {
  title: "Settings/Shell",
  component: SettingsShellView,
};
export default meta;

const shellProps = {
  locale: "en" as const,
  active: "privacy" as const,
  a11yGranted: true,
  onNavigate: () => {},
  onRecheckA11y: () => {},
  onOpenA11ySettings: () => {},
};

export const WithPrivacy: StoryObj<typeof SettingsShellView> = {
  render: (args) => (
    <SettingsShellView {...args}>
      <PrivacyView
        locale="en"
        status={{ enabled: true, retention_days: 30, record_count: 12 }}
        loading={false}
        error={null}
        busy={null}
        clearOpen={false}
        toasts={[]}
        external={{ state: "disabled" }}
        externalBusy={false}
        tokenOnce={null}
        tokenCopied={false}
        onRetry={() => {}}
        onEnabledChange={() => {}}
        onRetentionChange={() => {}}
        onOpenClear={() => {}}
        onCloseClear={() => {}}
        onConfirmClear={() => {}}
        onEnableExternal={() => {}}
        onDisableExternal={() => {}}
        onRegenToken={() => {}}
        onCopyToken={() => {}}
        onDismissToast={() => {}}
      />
    </SettingsShellView>
  ),
  args: shellProps,
};

export const WithHistory: StoryObj<typeof SettingsShellView> = {
  render: (args) => (
    <SettingsShellView {...args} active="history">
      <HistoryView
        locale="en"
        state="populated"
        items={[
          {
            session_uuid: "s1",
            timestamp: 1,
            trigger_source: "selection",
            detected_language: "en",
            target_language: "zh",
            is_favorite: true,
            source_text: "The quick brown fox",
            results: [],
            corrupt: false,
          },
        ]}
        query=""
        favoritesOnly={false}
        hasMore
        notice=""
        busy={false}
        onQueryChange={() => {}}
        onSearch={() => {}}
        onFavoritesOnlyChange={() => {}}
        onLoadMore={() => {}}
        onToggleFavorite={() => {}}
        onRemove={() => {}}
        onExport={() => {}}
      />
    </SettingsShellView>
  ),
  args: shellProps,
};

export const A11yBanner: StoryObj<typeof SettingsShellView> = {
  render: (args) => (
    <SettingsShellView {...args} a11yGranted={false}>
      <div>section content</div>
    </SettingsShellView>
  ),
  args: shellProps,
};

export const LongChineseNav: StoryObj<typeof SettingsShellView> = {
  render: (args) => (
    <SettingsShellView {...args} locale="zh" a11yGranted={false}>
      <div>中文分区内容</div>
    </SettingsShellView>
  ),
  args: shellProps,
};

export const Dark: StoryObj<typeof SettingsShellView> = {
  parameters: { backgrounds: { default: "dark" } },
  decorators: [
    (Story) => (
      <FluentProvider theme={webDarkTheme}>
        <Story />
      </FluentProvider>
    ),
  ],
  render: (args) => (
    <SettingsShellView {...args} locale="zh">
      <div>深色模式内容</div>
    </SettingsShellView>
  ),
  args: shellProps,
};

export const NarrowRail: StoryObj<typeof SettingsShellView> = {
  parameters: { viewport: { defaultViewport: "mobile1" } },
  render: (args) => (
    <SettingsShellView {...args} locale="zh">
      <div>窄窗口内容</div>
    </SettingsShellView>
  ),
  args: shellProps,
};
