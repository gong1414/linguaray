/*
 * Adapted from Ueli's MIT-licensed renderer at commit
 * f04ebdd82df71949d6b685ca7f2e5dd7e9b1bf90.
 * Copyright (c) 2023 Oliver Schwendener.
 */
import { Divider } from "@fluentui/react-components";
import type { KeyboardEvent, ReactNode, RefObject } from "react";

export type BaseLayoutProps = {
  header?: ReactNode;
  contentRef?: RefObject<HTMLDivElement | null>;
  content: ReactNode;
  footer?: ReactNode;
  onKeyDown?: (event: KeyboardEvent<HTMLDivElement>) => void;
  transparent?: boolean;
};

export function BaseLayout({
  header,
  content,
  contentRef,
  footer,
  onKeyDown,
  transparent = false,
}: BaseLayoutProps) {
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100%",
        minHeight: 0,
        background: transparent ? "transparent" : undefined,
      }}
      onKeyDown={onKeyDown}
      tabIndex={-1}
    >
      {header}
      {header ? <Divider appearance="subtle" /> : null}
      <div
        ref={contentRef}
        style={{ minHeight: 0, flexGrow: 1, overflowX: "auto", overflowY: "auto" }}
      >
        {content}
      </div>
      {footer ? <Divider appearance="subtle" /> : null}
      {footer}
    </div>
  );
}

