import { Show, type Component, type JSX, splitProps } from "solid-js";
import Spinner from "./Spinner";
import "./IconButton.css";

export type IconButtonVariant = "ghost" | "primary" | "destructive";
export type IconButtonSize = "sm" | "md" | "lg";

export type IconButtonProps = Omit<
  JSX.ButtonHTMLAttributes<HTMLButtonElement>,
  "children"
> & {
  /** REQUIRED: icon-only control needs an accessible name. */
  "aria-label": string;
  variant?: IconButtonVariant;
  size?: IconButtonSize;
  loading?: boolean;
  loadingLabel?: string;
  children?: JSX.Element;
};

/**
 * MASTER §7 IconButton. Always square, icon centered, aria-label mandatory.
 * Same state model as Button.
 */
const IconButton: Component<IconButtonProps> = (props) => {
  const [local, rest] = splitProps(props, [
    "variant",
    "size",
    "loading",
    "loadingLabel",
    "class",
    "classList",
    "children",
    "disabled",
    "aria-label",
  ]);

  const variant = () => local.variant ?? "ghost";
  const size = () => local.size ?? "md";
  const disabled = () => local.disabled || local.loading;
  // Spinner icon diameter when loading. Sm button is 28px, so 12px spinner
  // fits the inner target (MASTER Spinner allows 12|16|20).
  const iconSize = () => (size() === "sm" ? 12 : size() === "lg" ? 20 : 16);

  return (
    <button
      {...rest}
      disabled={disabled()}
      aria-busy={local.loading ? "true" : undefined}
      aria-label={local["aria-label"]}
      class={`lr-icon-btn lr-focusable lr-icon-btn--${variant()} lr-icon-btn--${size()}${
        local.class ? ` ${local.class}` : ""
      }`}
      classList={local.classList}
    >
      <Show when={local.loading} fallback={local.children}>
        <Spinner size={iconSize()} label={local.loadingLabel ?? "Loading…"} />
      </Show>
    </button>
  );
};

export default IconButton;
