import { X } from "lucide-solid";
import { onCleanup, onMount, type Component } from "solid-js";
import "./Banner.css";

export type ToastVariant = "info" | "success" | "warning" | "destructive";

export type ToastProps = {
  variant: ToastVariant;
  message: string;
  /** Auto-dismiss after N ms. Default 3000. Destructive NEVER auto-dismisses. */
  duration?: number;
  onDismiss: () => void;
  /** Accessible label for the dismiss button (i18n). Default: "Dismiss". */
  dismissLabel?: string;
  /** Accessible name for the toast region (i18n). Defaults to `message` so a
   *  screen reader announces the toast content as a single labeled region
   *  (the inner message text is kept for sighted users). */
  ariaLabel?: string;
  class?: string;
};

/**
 * MASTER §7 Toast.
 * - info/success/warning → role="status"; warning/destructive → role="alert"
 *   (per §7: warning joins destructive as alert for urgency).
 * - Auto-dismiss 3s default for info/success/warning.
 * - Destructive NEVER auto-dismisses — explicit dismissal required.
 *
 * Timer lifecycle: the Toast OWNS its timer. onMount starts it (if
 * auto-dismiss applies), onCleanup clears it. This prevents leaks even if the
 * host page doesn't use a generation-token pattern for toasts.
 */
const Toast: Component<ToastProps> = (props) => {
  const shouldAutoDismiss = () =>
    props.variant !== "destructive" && (props.duration ?? 3000) > 0;

  let timerId: ReturnType<typeof setTimeout> | undefined;

  onMount(() => {
    if (shouldAutoDismiss()) {
      timerId = setTimeout(() => props.onDismiss(), props.duration ?? 3000);
    }
  });

  onCleanup(() => {
    if (timerId !== undefined) clearTimeout(timerId);
  });

  return (
    <div
      class={`lr-toast lr-toast--${props.variant}${
        props.class ? ` ${props.class}` : ""
      }`}
      role={props.variant === "warning" || props.variant === "destructive" ? "alert" : "status"}
      aria-label={props.ariaLabel ?? props.message}
    >
      <span class="lr-toast__message">{props.message}</span>
      <button
        type="button"
        class="lr-toast__dismiss lr-focusable"
        aria-label={props.dismissLabel ?? "Dismiss"}
        onClick={() => props.onDismiss()}
      >
        <X size={16} aria-hidden="true" />
      </button>
    </div>
  );
};

export default Toast;
