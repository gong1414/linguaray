import { Show, type Component, type JSX } from "solid-js";
import "./EmptyState.css";

export type EmptyStateProps = {
  icon: JSX.Element;
  title: string;
  description?: string;
  action?: JSX.Element;
  class?: string;
};

/**
 * MASTER §7 EmptyState. Centered 32px icon + title + description + optional action.
 */
const EmptyState: Component<EmptyStateProps> = (props) => {
  return (
    <div
      class={`lr-empty-state${props.class ? ` ${props.class}` : ""}`}
    >
      <span class="lr-empty-state__icon">{props.icon}</span>
      <span class="lr-empty-state__title">{props.title}</span>
      <Show when={props.description}>
        <span class="lr-empty-state__description">{props.description}</span>
      </Show>
      <Show when={props.action}>
        <div class="lr-empty-state__action">{props.action}</div>
      </Show>
    </div>
  );
};

export default EmptyState;
