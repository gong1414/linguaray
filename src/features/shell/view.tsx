import type { ReactNode } from "react";
import { Alert, Button, Flex } from "antd";
import {
  BookOutlined,
  CloudServerOutlined,
  HistoryOutlined,
  KeyOutlined,
  ThunderboltOutlined,
  SafetyCertificateOutlined,
  SyncOutlined,
  TranslationOutlined,
} from "@ant-design/icons";
import { SHELL_COPY } from "./copy";
import { SETTINGS_SECTIONS, type SettingsSection } from "./model";
import { SettingsLayout, SettingsNavigation } from "../../ui/x";

const SECTION_ICONS: Record<SettingsSection, ReactNode> = {
  "provider-center": <CloudServerOutlined aria-hidden />,
  "keystore-recovery": <KeyOutlined aria-hidden />,
  shortcuts: <ThunderboltOutlined aria-hidden />,
  privacy: <SafetyCertificateOutlined aria-hidden />,
  history: <HistoryOutlined aria-hidden />,
  vocabulary: <TranslationOutlined aria-hidden />,
  dictionary: <BookOutlined aria-hidden />,
  updater: <SyncOutlined aria-hidden />,
};

export type SettingsShellViewProps = {
  locale: "zh" | "en";
  active: SettingsSection;
  a11yGranted: boolean | null;
  children: ReactNode;
  onNavigate: (section: SettingsSection) => void;
  onRecheckA11y: () => void;
  onOpenA11ySettings: () => void;
};

/** Ant Design settings shell; all capabilities remain outside this view. */
export function SettingsShellView(props: SettingsShellViewProps) {
  const t = SHELL_COPY[props.locale];
  const items = SETTINGS_SECTIONS.map((section) => ({
    value: section,
    label: t.nav[section],
    icon: SECTION_ICONS[section],
  }));

  return (
    <div data-testid="shell" data-page={props.active} data-layout="ant-design-x">
      <SettingsLayout
        navigation={
          <SettingsNavigation label={t.navLabel} active={props.active} items={items} onNavigate={props.onNavigate} />
        }
      >
        <div>
          {props.a11yGranted === false ? (
            <Alert
              className="lr-x-permission-alert"
              type="warning"
              showIcon
              title={t.a11y.title}
              description={t.a11y.hint}
              data-testid="a11y-banner"
              action={
                <Flex gap="small" wrap>
                  <Button type="text" size="small" onClick={props.onRecheckA11y} data-testid="a11y-recheck">{t.a11y.recheck}</Button>
                  <Button size="small" onClick={props.onOpenA11ySettings} data-testid="a11y-open-settings">{t.a11y.openSettings}</Button>
                </Flex>
              }
            />
          ) : null}
          {props.children}
        </div>
      </SettingsLayout>
    </div>
  );
}

export default SettingsShellView;
