import { Show, type Component, type JSX, splitProps } from "solid-js";
import Spinner from "./Spinner";
import "./Button.css";

export type ButtonVariant = "primary" | "secondary" | "ghost" | "destructive";
export type ButtonSize = "sm" | "md" | "lg";

export type ButtonProps = JSX.ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: ButtonVariant;
  size?: ButtonSize;
  /** Show spinner + set disabled + aria-busy. Keeps button width (no shift). */
  loading?: boolean;
  /** Accessible label for the loading spinner. */
  loadingLabel?: string;
  leftIcon?: JSX.Element;
  rightIcon?: JSX.Element;
  fullWidth?: boolean;
};

/**
 * MASTER §7 Button.
 * Implements §6 states: default/hover/pressed/focus/disabled/loading.
 * Loading = native disabled + aria-busy + spinner + sr-only label.
 */
const Button: Component<ButtonProps> = (props) => {
  const [local, rest] = splitProps(props, [
    "variant",
    "size",
    "loading",
    "loadingLabel",
    "leftIcon",
    "rightIcon",
    "fullWidth",
    "class",
    "classList",
    "children",
    "disabled",
  ]);

  const variant = () => local.variant ?? "primary";
  const size = () => local.size ?? "md";
  const disabled = () => local.disabled || local.loading;

  return (
    <button
      {...rest}
      disabled={disabled()}
      aria-busy={local.loading ? "true" : undefined}
      class={`lr-btn lr-focusable lr-btn--${variant()} lr-btn--${size()}${
        local.fullWidth ? " lr-btn--block" : ""
      }${local.class ? ` ${local.class}` : ""}`}
      classList={local.classList}
    >
      <span class="lr-btn__content">
        <Show when={local.loading}>
          <Spinner size={12} label={local.loadingLabel ?? "Loading…"} />
        </Show>
        <Show when={!local.loading && local.leftIcon}>{local.leftIcon}</Show>
        <Show when={local.children}>{local.children}</Show>
        <Show when={!local.loading && local.rightIcon}>{local.rightIcon}</Show>
      </span>
    </button>
  );
};

export default Button;
