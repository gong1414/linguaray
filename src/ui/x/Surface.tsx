import type { KeyboardEvent, ReactNode, RefObject } from "react";
import { Divider } from "antd";

export function SurfaceLayout({ header, content, contentRef, footer, onKeyDown, transparent = false }: {
  header?: ReactNode;
  content: ReactNode;
  contentRef?: RefObject<HTMLDivElement | null>;
  footer?: ReactNode;
  onKeyDown?: (event: KeyboardEvent<HTMLDivElement>) => void;
  transparent?: boolean;
}) {
  return (
    <div className={transparent ? "lr-x-surface lr-x-surface-transparent" : "lr-x-surface"} onKeyDown={onKeyDown} tabIndex={-1}>
      {header}
      {header ? <Divider /> : null}
      <div ref={contentRef} className="lr-x-surface-content">{content}</div>
      {footer ? <Divider /> : null}
      {footer}
    </div>
  );
}

export function SurfaceHeader({ children, draggable }: { children: ReactNode; draggable?: boolean }) {
  return <header className={draggable ? "lr-x-surface-header draggable-area" : "lr-x-surface-header"}>{children}</header>;
}

export function SurfaceFooter({ children, draggable }: { children?: ReactNode; draggable?: boolean }) {
  return <footer className={draggable ? "lr-x-surface-footer draggable-area" : "lr-x-surface-footer"}>{children}</footer>;
}
