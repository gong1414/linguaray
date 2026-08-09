import { Show, type Component, type JSX } from "solid-js";
import "./SidebarItem.css";

export type SidebarItemProps = {
  label: string;
  icon: JSX.Element;
  active?: boolean;
  badge?: string;
  onClick?: () => void;
  disabled?: boolean;
  /** Accessible label. Defaults to `label`; pass a combined string for disabled
   *  items so a screen reader announces BOTH the label and the placeholder hint
   *  (e.g. "Shortcuts — Coming in R3b"). */
  ariaLabel?: string;
};

const SidebarItem: Component<SidebarItemProps> = (props) => {
  return (
    <button
      type="button"
      class="sidebar-item"
      classList={{ "sidebar-item--active": !!props.active, "sidebar-item--disabled": !!props.disabled }}
      aria-current={props.active ? "page" : undefined}
      aria-label={props.ariaLabel ?? props.label}
      aria-disabled={props.disabled || undefined}
      onClick={() => {
        // Disabled placeholders announce via aria-disabled but stay focusable
        // (NOT native disabled) so keyboard + SR users can discover them. Gate
        // the click here so an Enter/Space on a disabled item is a no-op.
        if (props.disabled) return;
        props.onClick?.();
      }}
    >
      <span class="sidebar-item__icon" aria-hidden="true">{props.icon}</span>
      <span class="sidebar-item__label">{props.label}</span>
      <Show when={props.badge}>
        <span class="sidebar-item__badge">{props.badge}</span>
      </Show>
    </button>
  );
};
// 注：原生 <button type="button"> 天然支持 Enter/Space 触发 click，无需人工 onKeyDown。
// tabindex 默认为 0，无需显式设置。
export default SidebarItem;
