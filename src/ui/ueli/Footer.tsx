/*
 * Adapted from Ueli's MIT-licensed renderer at commit
 * f04ebdd82df71949d6b685ca7f2e5dd7e9b1bf90.
 * Copyright (c) 2023 Oliver Schwendener.
 */
import type { ReactNode } from "react";

export type FooterProps = {
  children?: ReactNode;
  draggable?: boolean;
};

export function Footer({ children, draggable }: FooterProps) {
  return (
    <div
      className={draggable ? "draggable-area" : undefined}
      style={{
        flexShrink: 0,
        padding: 8,
        gap: 8,
        boxSizing: "border-box",
        display: "flex",
        flexDirection: "row",
        justifyContent: "space-between",
        alignItems: "center",
      }}
    >
      {children}
    </div>
  );
}
