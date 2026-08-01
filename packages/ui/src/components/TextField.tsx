import {
  Show,
  createMemo,
  type Component,
  type JSX,
  splitProps,
} from "solid-js";
import "./TextField.css";

export type TextFieldSize = "md" | "lg";

export type TextFieldProps = Omit<
  JSX.InputHTMLAttributes<HTMLInputElement>,
  "size"
> & {
  label: string;
  size?: TextFieldSize;
  helperText?: string;
  errorText?: string;
  monospace?: boolean;
};

let idCounter = 0;
function useId(prefix: string): string {
  return `${prefix}-${idCounter++}`;
}

/**
 * MASTER §7 TextField.
 * - label always visible, associated via <label for=id>.
 * - errorText present → aria-invalid + border = destructive-fg + helper hidden
 *   + aria-describedby → error element.
 * - helperText present (no error) → aria-describedby → helper element.
 */
const TextField: Component<TextFieldProps> = (props) => {
  const [local, rest] = splitProps(props, [
    "label",
    "size",
    "helperText",
    "errorText",
    "monospace",
    "class",
    "classList",
    "id",
    "disabled",
  ]);

  // Compute once so <label for> and <input id> share the exact same value.
  const inputId = local.id ?? useId("tf");
  const helperId = () => `${inputId}-helper`;
  const errorId = () => `${inputId}-error`;

  const hasError = createMemo(() => Boolean(local.errorText));
  const describedBy = createMemo(() => {
    if (hasError()) return errorId();
    if (local.helperText) return helperId();
    return undefined;
  });

  const size = () => local.size ?? "md";

  return (
    <div
      class={`lr-text-field lr-text-field--${size()}${
        local.monospace ? " lr-text-field--mono" : ""
      }${hasError() ? " lr-text-field--error" : ""}${
        local.disabled ? " lr-text-field--disabled" : ""
      }${local.class ? ` ${local.class}` : ""}`}
      classList={local.classList}
    >
      <label class="lr-text-field__label" for={inputId}>
        {local.label}
      </label>
      <span class="lr-text-field__control">
        <input
          {...rest}
          id={inputId}
          class="lr-text-field__input lr-focusable"
          disabled={local.disabled}
          aria-invalid={hasError() ? "true" : undefined}
          aria-describedby={describedBy()}
        />
      </span>
      <Show when={hasError()}>
        <p class="lr-text-field__error" id={errorId()}>
          {local.errorText}
        </p>
      </Show>
      <Show when={!hasError() && local.helperText}>
        <p class="lr-text-field__helper" id={helperId()}>
          {local.helperText}
        </p>
      </Show>
    </div>
  );
};

export default TextField;
