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
  /** Ref to a fallback element if the trigger is disabled/removed (e.g. the
   *  next provider card or the provider list container). */
  fallbackFocusRef?: { current?: HTMLElement };
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
  };

  const handleCancel = () => {
    props.onCancel();
    props.onOpenChange(false);
  };

  // Unified focus restore: called by onCloseAutoFocus for ALL close paths
  // (Cancel click, Esc, overlay dismiss, controlled open=false).
  // Trigger may be disabled (e.g. after destructive Confirm deletes the
  // provider) — fall back to the next focusable element in the document.
  const restoreFocus = () => {
    const trigger = props.triggerRef?.current;
    // Trigger is invalid if: not in DOM, disabled, aria-disabled, OR inside
    // a deleting row (pointer-events:none, not operable)
    const isTriggerValid = (el: HTMLElement | undefined): el is HTMLElement => {
      if (!el || !document.contains(el)) return false;
      if (el.hasAttribute("disabled")) return false;
      if (el.getAttribute("aria-disabled") === "true") return false;
      const deletingAncestor = el.closest('[data-status="deleting"]');
      return !deletingAncestor;
    };
    if (isTriggerValid(trigger)) {
      trigger.focus();
      return;
    }
    // Fallback to explicit fallbackFocusRef (e.g. provider list container)
    const fallback = props.fallbackFocusRef?.current;
    if (fallback && document.contains(fallback)) {
      fallback.focus();
      return;
    }
    // No last-resort selector — caller MUST provide fallbackFocusRef for
    // disappearing-trigger dialogs. This avoids coupling the shared component
    // to consumer-specific CSS classes.
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
          onCloseAutoFocus={(e: Event) => {
            e.preventDefault();
            restoreFocus();
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
