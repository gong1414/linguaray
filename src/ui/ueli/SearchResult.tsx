/*
 * Adapted from Ueli's DetailedSearchResultItem, SearchResultListItem, and
 * SearchResultListItemSelectedIndicator at commit
 * f04ebdd82df71949d6b685ca7f2e5dd7e9b1bf90.
 * Copyright (c) 2023 Oliver Schwendener. Licensed under MIT.
 */
import { Text, tokens } from "@fluentui/react-components";
import type { ReactElement, ReactNode } from "react";

export type SearchResultItemProps = {
  selected?: boolean;
  icon?: ReactElement;
  name: string;
  details?: ReactNode;
  badge?: ReactNode;
  actions?: ReactNode;
  onClick?: () => void;
  children?: ReactNode;
};

export function SearchResultItem({
  selected = false,
  icon,
  name,
  details,
  badge,
  actions,
  onClick,
  children,
}: SearchResultItemProps) {
  return (
    <div
      onClick={onClick}
      style={{
        position: "relative",
        backgroundColor: selected ? tokens.colorNeutralBackground1Selected : undefined,
        userSelect: "none",
        borderRadius: tokens.borderRadiusMedium,
        cursor: onClick ? "pointer" : "default",
      }}
    >
      {selected ? (
        <div
          style={{
            position: "absolute",
            left: 0,
            top: "50%",
            backgroundColor: tokens.colorBrandForeground1,
            height: "45%",
            width: 3,
            transform: "translateY(-50%)",
            borderRadius: tokens.borderRadiusLarge,
          }}
        />
      ) : null}
      <div
        style={{
          display: "flex",
          flexDirection: "row",
          alignItems: "center",
          justifyContent: "space-between",
          padding: "8px 10px",
          boxSizing: "border-box",
          gap: 10,
          width: "100%",
        }}
      >
        {icon ? <div style={{ flexShrink: 0, display: "flex" }}>{icon}</div> : null}
        <div style={{ display: "flex", flexDirection: "column", flexGrow: 1, overflow: "hidden" }}>
          <Text weight="semibold" style={{ whiteSpace: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }}>
            {name}
          </Text>
          {details ? <Text size={200}>{details}</Text> : null}
          {children}
        </div>
        {badge ? <div style={{ flexShrink: 0, display: "flex" }}>{badge}</div> : null}
        {actions ? <div style={{ flexShrink: 0, display: "flex" }}>{actions}</div> : null}
      </div>
    </div>
  );
}

export function SearchResultList({ children }: { children: ReactNode }) {
  return <div style={{ display: "flex", flexDirection: "column", gap: 5 }}>{children}</div>;
}
