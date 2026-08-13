import { Show, type Component, type JSX } from "solid-js";
import "./ListRow.css";

export type ListRowProps = {
  leading?: JSX.Element;
  title: string;
  subtitle?: string;
  trailing?: JSX.Element;
  onClick?: () => void;
  /** Required when `onClick` is present and the visible title is insufficient. */
  ariaLabel?: string;
  class?: string;
};

/**
 * MASTER ListRow. A row with a trailing action never nests that action inside
 * the primary button: content and trailing controls are siblings.
 */
const ListRow: Component<ListRowProps> = (props) => {
  const content = (
    <>
      <Show when={props.leading}>
        <span class="list-row__leading">{props.leading}</span>
      </Show>
      <span class="list-row__text">
        <span class="list-row__title">{props.title}</span>
        <Show when={props.subtitle}>
          <span class="list-row__subtitle">{props.subtitle}</span>
        </Show>
      </span>
    </>
  );

  return (
    <div
      class={`list-row ${props.subtitle ? "list-row--two-line" : "list-row--single"}${
        props.class ? ` ${props.class}` : ""
      }`}
    >
      <Show
        when={props.onClick}
        fallback={<div class="list-row__content">{content}</div>}
      >
        <button
          type="button"
          class="list-row__content list-row__content--button lr-focusable"
          aria-label={props.ariaLabel}
          onClick={() => props.onClick?.()}
        >
          {content}
        </button>
      </Show>
      <Show when={props.trailing}>
        <div class="list-row__trailing">{props.trailing}</div>
      </Show>
    </div>
  );
};

export default ListRow;
