/*
 * Adapted from Ueli's Settings, Navigation, SettingGroupList, SettingGroup,
 * and Setting renderer components at commit
 * f04ebdd82df71949d6b685ca7f2e5dd7e9b1bf90.
 * Copyright (c) 2023 Oliver Schwendener. Licensed under MIT.
 */
import {
  Body1,
  Caption1,
  Card,
  NavDrawer,
  NavDrawerBody,
  NavItem,
  NavSectionHeader,
  Text,
  tokens,
} from "@fluentui/react-components";
import type { ReactElement, ReactNode } from "react";

export type UeliNavigationItem<T extends string> = {
  value: T;
  label: string;
  icon: ReactElement;
};

export function SettingsNavigation<T extends string>({
  label,
  active,
  items,
  onNavigate,
}: {
  label: string;
  active: T;
  items: UeliNavigationItem<T>[];
  onNavigate: (value: T) => void;
}) {
  return (
    <NavDrawer
      density="small"
      open
      type="inline"
      selectedValue={active}
      onNavItemSelect={(_, data) => onNavigate(data.value as T)}
      aria-label={label}
      style={{ height: "100%", minWidth: 220 }}
    >
      <NavDrawerBody>
        <NavSectionHeader>{label}</NavSectionHeader>
        {items.map((item) => (
          <NavItem
            key={item.value}
            value={item.value}
            icon={item.icon}
            onFocus={() => onNavigate(item.value)}
          >
            {item.label}
          </NavItem>
        ))}
      </NavDrawerBody>
    </NavDrawer>
  );
}

export function SettingsLayout({ navigation, children }: { navigation: ReactNode; children: ReactNode }) {
  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
      <div
        style={{
          flexGrow: 1,
          display: "flex",
          flexDirection: "row",
          boxSizing: "border-box",
          height: "100%",
          width: "100%",
          overflow: "hidden",
        }}
      >
        <div style={{ display: "flex", flexShrink: 0 }}>
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              gap: 20,
              boxSizing: "border-box",
              height: "100vh",
              overflowX: "auto",
              overflowY: "auto",
              borderRight: `${tokens.strokeWidthThin} solid ${tokens.colorNeutralStroke2}`,
            }}
          >
            {navigation}
          </div>
        </div>
        <div
          style={{
            height: "100vh",
            flexGrow: 1,
            minWidth: 0,
            overflowY: "auto",
            padding: 20,
            boxSizing: "border-box",
          }}
        >
          {children}
        </div>
      </div>
    </div>
  );
}

export function SettingGroupList({ children }: { children?: ReactNode }) {
  return <div style={{ display: "flex", flexDirection: "column", gap: 40 }}>{children}</div>;
}

export function SettingGroup({ title, children }: { title?: string; children?: ReactNode }) {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 5 }}>
      {title ? <Text weight="semibold" size={300}>{title}</Text> : null}
      {children}
    </div>
  );
}

export function Setting({
  label,
  description,
  control,
}: {
  label: string;
  description?: string;
  control: ReactElement;
}) {
  return (
    <Card appearance="filled-alternative">
      <div
        style={{
          width: "100%",
          display: "flex",
          flexDirection: "row",
          alignItems: "center",
          justifyContent: "space-between",
          gap: 16,
        }}
      >
        <div style={{ flex: "1 1 0", minWidth: 0 }}>
          <Body1>{label}</Body1>
          {description ? <Caption1 style={{ textWrap: "wrap" }}>{description}</Caption1> : null}
        </div>
        <div style={{ flex: "0 0 auto" }}>{control}</div>
      </div>
    </Card>
  );
}
