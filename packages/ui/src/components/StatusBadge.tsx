import { Show, type Component, type JSX } from "solid-js";
import "./StatusBadge.css";

export type StatusBadgeVariant = "success" | "warning" | "danger" | "info" | "neutral";
export type StatusBadgeProps = {
  variant: StatusBadgeVariant;
  children: JSX.Element;
  icon?: JSX.Element;
  dot?: boolean;
};

const StatusBadge: Component<StatusBadgeProps> = (props) => {
  return (
    <span
      class={`status-badge status-badge--${props.variant}`}
      role="img"
      aria-label={typeof props.children === "string" ? props.children : undefined}
    >
      <Show when={props.dot}>
        <span class="status-badge__dot" aria-hidden="true" />
      </Show>
      <Show when={props.icon}>
        <span class="status-badge__icon" aria-hidden="true">{props.icon}</span>
      </Show>
      <span class="status-badge__label">{props.children}</span>
    </span>
  );
};
export default StatusBadge;
