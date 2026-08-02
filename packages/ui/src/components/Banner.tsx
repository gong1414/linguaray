import { X } from "lucide-solid";
import { Show, type Component, type JSX } from "solid-js";
import "./Banner.css";

export type BannerVariant = "info" | "success" | "warning" | "destructive";

export type BannerProps = {
  variant: BannerVariant;
  title: string;
  description?: string;
  action?: JSX.Element;
  onDismiss?: () => void;
  /** Accessible label for the dismiss button (i18n). Default: "Dismiss". */
  dismissLabel?: string;
  class?: string;
};

/**
 * MASTER §7 Banner. Full-width, top of content area. *-fill bg + on-*-fill text.
 */
const Banner: Component<BannerProps> = (props) => {
  return (
    <div
      class={`lr-banner lr-banner--${props.variant}${
        props.class ? ` ${props.class}` : ""
      }`}
      role={props.variant === "destructive" || props.variant === "warning" ? "alert" : "status"}
    >
      <div class="lr-banner__content">
        <span class="lr-banner__title">{props.title}</span>
        <Show when={props.description}>
          <span class="lr-banner__description">{props.description}</span>
        </Show>
      </div>
      <Show when={props.action}>
        <div class="lr-banner__action">{props.action}</div>
      </Show>
      <Show when={props.onDismiss}>
        <button
          type="button"
          class="lr-banner__dismiss lr-focusable"
          aria-label={props.dismissLabel ?? "Dismiss"}
          onClick={() => props.onDismiss?.()}
        >
          <X size={16} aria-hidden="true" />
        </button>
      </Show>
    </div>
  );
};

export default Banner;
