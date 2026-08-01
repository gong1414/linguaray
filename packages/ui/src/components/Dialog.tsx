import { Dialog as KobalteDialog } from "@kobalte/core/dialog";
import { Show, type Component, type JSX } from "solid-js";
import "./Dialog.css";

export type DialogProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  description?: string;
  children?: JSX.Element;
  footer?: JSX.Element;
  /** Ref to the trigger element. On close, focus is restored to it. */
  triggerRef?: { current?: HTMLElement };
  class?: string;
};

/**
 * MASTER §7 Dialog. Esc to close, focus trap, focus restore to trigger —
 * all provided by Kobante. Overlay = bg-overlay; content = bg-elevated +
 * radius-lg + shadow-lg.
 */
const Dialog: Component<DialogProps> = (props) => {
  return (
    <KobalteDialog open={props.open} onOpenChange={props.onOpenChange}>
      <KobalteDialog.Portal>
        <KobalteDialog.Overlay class="lr-dialog__overlay" />
        <KobalteDialog.Content
          class={`lr-dialog__content${props.class ? ` ${props.class}` : ""}`}
          onCloseAutoFocus={() => {
            if (props.triggerRef?.current) {
              props.triggerRef.current.focus();
            }
          }}
        >
          <KobalteDialog.Title class="lr-dialog__title">
            {props.title}
          </KobalteDialog.Title>
          <Show when={props.description}>
            <KobalteDialog.Description class="lr-dialog__description">
              {props.description}
            </KobalteDialog.Description>
          </Show>
          <Show when={props.children}>
            <div class="lr-dialog__body">{props.children}</div>
          </Show>
          <Show when={props.footer}>
            <div class="lr-dialog__footer">{props.footer}</div>
          </Show>
        </KobalteDialog.Content>
      </KobalteDialog.Portal>
    </KobalteDialog>
  );
};

export default Dialog;
