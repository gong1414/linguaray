import type { ReactNode } from "react";
import { Card, Layout, Menu, Typography } from "antd";
import type { MenuProps } from "antd";

export type XNavigationItem<T extends string> = { value: T; label: string; icon: ReactNode };

export function SettingsNavigation<T extends string>({ label, active, items, onNavigate }: {
  label: string;
  active: T;
  items: XNavigationItem<T>[];
  onNavigate: (value: T) => void;
}) {
  const menuItems: MenuProps["items"] = items.map((item) => ({ key: item.value, icon: item.icon, label: item.label }));
  return (
    <nav className="lr-x-navigation" aria-label={label}>
      <Typography.Text className="lr-x-navigation-label" type="secondary">{label}</Typography.Text>
      <Menu mode="inline" selectedKeys={[active]} items={menuItems} onClick={({ key }) => onNavigate(key as T)} />
    </nav>
  );
}

export function SettingsLayout({ navigation, children }: { navigation: ReactNode; children: ReactNode }) {
  return (
    <Layout className="lr-x-settings">
      <Layout.Sider width={224} theme="light" className="lr-x-sider">{navigation}</Layout.Sider>
      <Layout.Content className="lr-x-settings-content">{children}</Layout.Content>
    </Layout>
  );
}

export function SettingGroupList({ children }: { children?: ReactNode }) {
  return <div className="lr-x-setting-groups">{children}</div>;
}

export function SettingGroup({ title, children }: { title?: string; children?: ReactNode }) {
  return (
    <section className="lr-x-setting-group">
      {title ? <Typography.Title level={5}>{title}</Typography.Title> : null}
      <div className="lr-x-setting-group-content">{children}</div>
    </section>
  );
}

export function Setting({ label, description, control }: { label: string; description?: string; control: ReactNode }) {
  return (
    <Card size="small" className="lr-x-setting-card">
      <div className="lr-x-setting-row">
        <div className="lr-x-setting-copy">
          <Typography.Text>{label}</Typography.Text>
          {description ? <Typography.Text type="secondary">{description}</Typography.Text> : null}
        </div>
        <div className="lr-x-setting-control">{control}</div>
      </div>
    </Card>
  );
}
