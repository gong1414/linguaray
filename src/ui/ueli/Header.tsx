/*
 * Adapted from Ueli's MIT-licensed renderer at commit
 * f04ebdd82df71949d6b685ca7f2e5dd7e9b1bf90.
 * Copyright (c) 2023 Oliver Schwendener.
 */
import type { ReactElement, ReactNode } from "react";

export type HeaderProps = {
  children: ReactNode;
  contentBefore?: ReactElement;
  draggable?: boolean;
};

export function Header({ draggable, contentBefore, children }: HeaderProps) {
  return (
    <div
      className={draggable ? "draggable-area" : undefined}
      style={{
        display: "flex",
        flexDirection: "row",
        alignItems: "center",
        flexShrink: 0,
        padding: 10,
        boxSizing: "border-box",
        gap: 10,
      }}
    >
      {contentBefore}
      {children}
    </div>
  );
}
