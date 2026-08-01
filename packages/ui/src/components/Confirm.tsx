import { Dialog as KobalteDialog } from "@kobalte/core/dialog";
import { Show, type Component, type JSX } from "solid-js";
import Button from "./Button";
import "./Dialog.css";

export type ConfirmVariant = "primary" | "destructive";

export type ConfirmProps = {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  message: string;
  confirmLabel: string;
  cancelLabel: string;
  variant?: ConfirmVariant;
  onConfirm: () => void;
  onCancel: () => void;
  /** Optional extra content rendered between message and footer (e.g. consent
   *  recipient list). */
  children?: JSX.Element;
  /** Ref to the trigger element that opened this dialog. On close, focus is
   *  restored to this element via onCloseAutoFocus. */
  triggerRef?: { current?: HTMLElement };
};

/**
 * MASTER §7 Confirm. Destructive variant: initial focus lands on Cancel
 * (NOT Confirm) to prevent accidental destructive action via Enter. Achieved
 * by intercepting onOpenAutoFocus → preventDefault + ref-focus Cancel.
 */
const Confirm: Component<ConfirmProps> = (props) => {
  const variant = () => props.variant ?? "primary";
  let cancelRef: HTMLButtonElement | undefined;

  const handleConfirm = () => {
    props.onConfirm();
    props.onOpenChange(false);
    restoreFocus();
  };

  const handleCancel = () => {
    props.onCancel();
    props.onOpenChange(false);
    restoreFocus();
  };

  const restoreFocus = () => {
    if (props.triggerRef?.current) {
      const el = props.triggerRef.current;
      setTimeout(() => el.focus(), 0);
    }
  };

  return (
    <KobalteDialog open={props.open} onOpenChange={props.onOpenChange}>
      <KobalteDialog.Portal>
        <KobalteDialog.Overlay class="lr-dialog__overlay" />
        <KobalteDialog.Content
          class="lr-dialog__content"
          onOpenAutoFocus={(e) => {
            if (variant() === "destructive") {
              e.preventDefault();
              cancelRef?.focus();
            }
          }}
        >
          {/* Destructive: override auto-focus to land on Cancel, not the
              default first-focusable (which would be Cancel here anyway since
              it's first in DOM, but we make it explicit + robust). */}
          <KobalteDialog.Title class="lr-dialog__title">
            {props.title}
          </KobalteDialog.Title>
          <KobalteDialog.Description class="lr-dialog__description">
            {props.message}
          </KobalteDialog.Description>
          <Show when={props.children}>
            <div class="lr-dialog__body">{props.children}</div>
          </Show>
          <div class="lr-dialog__footer">
            <Button
              variant="secondary"
              size="md"
              ref={cancelRef}
              onClick={handleCancel}
            >
              {props.cancelLabel}
            </Button>
            <Button
              variant={variant() === "destructive" ? "destructive" : "primary"}
              size="md"
              onClick={handleConfirm}
            >
              {props.confirmLabel}
            </Button>
          </div>
        </KobalteDialog.Content>
      </KobalteDialog.Portal>
    </KobalteDialog>
  );
};

export default Confirm;
