import { Loader2 } from "lucide-solid";
import type { Component } from "solid-js";
import "./Spinner.css";

export type SpinnerProps = {
  /** Icon diameter in px. MASTER §7 Spinner: 12 | 16 | 20. */
  size?: 12 | 16 | 20;
  /** Accessible + (reduced-motion) visible label. */
  label?: string;
  class?: string;
};

/**
 * MASTER §7 Spinner.
 *
 * The icon is decorative (`aria-hidden`); the text element carries the
 * accessible name. In full motion the text is `.lr-visually-hidden` (screen
 * readers announce it, nothing on screen). Under reduced-motion CSS flips the
 * text to be the visible "Loading…" fallback (Spinner.css), so the same single
 * text node serves both the a11y name and the reduced-motion visual — never
 * duplicated in the a11y tree.
 */
const Spinner: Component<SpinnerProps> = (props) => {
  const size = () => props.size ?? 16;
  const label = () => props.label ?? "Loading…";
  return (
    <span
      class={`lr-spinner${props.class ? ` ${props.class}` : ""}`}
      role="status"
      aria-live="polite"
    >
      <Loader2 class="lr-spinner__icon" size={size()} aria-hidden="true" />
      <span class="lr-spinner__label lr-visually-hidden">{label()}</span>
    </span>
  );
};

export default Spinner;
