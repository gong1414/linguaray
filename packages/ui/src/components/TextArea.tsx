import {
  Show,
  createMemo,
  type Component,
  type JSX,
  splitProps,
} from "solid-js";
import "./TextArea.css";

export type TextAreaProps = Omit<
  JSX.TextareaHTMLAttributes<HTMLTextAreaElement>,
  "aria-label"
> & {
  /** Optional visible label; when present it is associated via <label for=id>. */
  label?: string;
  /** Optional helper text shown beneath the control when no error is present. */
  helperText?: string;
  /** Error text — when present, marks the control invalid and links via aria-describedby. */
  errorText?: string;
  /** Maps to the underlying aria-label; use when no visible label is shown. */
  ariaLabel?: string;
  /** Forwarded to the underlying <textarea> so callers can focus/inspect it. */
  ref?: (el: HTMLTextAreaElement) => void;
};

let idCounter = 0;
function useId(prefix: string): string {
  return `${prefix}-${idCounter++}`;
}

/**
 * MASTER §7 TextArea — the multi-line sibling of TextField.
 *
 * - label (optional) is associated via <label for=id>.
 * - errorText present → aria-invalid + border = status-danger + helper hidden
 *   + aria-describedby → error element.
 * - helperText present (no error) → aria-describedby → helper element.
 * - ref callback is forwarded to the underlying <textarea>.
 */
const TextArea: Component<TextAreaProps> = (props) => {
  const [local, rest] = splitProps(props, [
    "label",
    "helperText",
    "errorText",
    "ariaLabel",
    "ref",
    "class",
    "classList",
    "id",
    "disabled",
  ]);

  // Compute once so <label for> and <textarea id> share the exact same value.
  const inputId = local.id ?? useId("ta");
  const helperId = () => `${inputId}-helper`;
  const errorId = () => `${inputId}-error`;

  const hasError = createMemo(() => Boolean(local.errorText));
  const describedBy = createMemo(() => {
    if (hasError()) return errorId();
    if (local.helperText) return helperId();
    return undefined;
  });

  return (
    <div
      class={`lr-text-area${hasError() ? " lr-text-area--error" : ""}${
        local.disabled ? " lr-text-area--disabled" : ""
      }${local.class ? ` ${local.class}` : ""}`}
      classList={local.classList}
    >
      <Show when={local.label}>
        <label class="lr-text-area__label" for={inputId}>
          {local.label}
        </label>
      </Show>
      <textarea
        {...rest}
        ref={local.ref}
        id={inputId}
        class="lr-text-area__input lr-focusable"
        disabled={local.disabled}
        aria-label={local.ariaLabel}
        aria-invalid={hasError() ? "true" : undefined}
        aria-describedby={describedBy()}
      />
      <Show when={hasError()}>
        <p class="lr-text-area__error" id={errorId()}>
          {local.errorText}
        </p>
      </Show>
      <Show when={!hasError() && local.helperText}>
        <p class="lr-text-area__helper" id={helperId()}>
          {local.helperText}
        </p>
      </Show>
    </div>
  );
};

export default TextArea;
