import {
  createSignal,
  onMount,
  onCleanup,
  Show,
  For,
  type Component,
} from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { Banner, Confirm, Button, Toast } from "@linguaray/ui";
import { SETTINGS_COPY } from "./copy";
import { detectLocale } from "../../i18n";
import "./KeystoreRecovery.css";

/**
 * Keystore Recovery (Surface 06).
 *
 * Four visual states:
 *   - healthy: keystore_health returned "" -> no banner (settings normal).
 *   - corrupt: keystore_health returned a non-empty fail-closed reason (or the
 *     call itself threw). A destructive Banner offers "Archive & re-enter" and
 *     "Reset". The Reset action opens a destructive Confirm whose initial
 *     focus lands on Cancel (provided by the Confirm component contract).
 *   - archived: archive_keystore (or reset_keystore) succeeded. An info Banner
 *     prompts the user to re-enter their keys.
 *   - reset-confirm is a transient dialog-open state, NOT a top-level state.
 *
 * Destructive Confirm never auto-focuses Confirm — `variant="destructive"` in
 * the Confirm component intercepts onOpenAutoFocus and focuses Cancel. Do NOT
 * override that behaviour here.
 */
export type KsState = "healthy" | "corrupt" | "archived";

export type KsToastEntry = {
  id: number;
  variant: "info" | "success" | "warning" | "destructive";
  message: string;
};

/** rev-7-4: pure presentational View. Shared by the production mount + the
 *  ui-lab visual fixture. Renders the FULL surface: Banner (corrupt/archived)
 *  + Confirm (destructive, Cancel-focused) + Toast stack + busy. No IPC, no
 *  effects. */
export type KeystoreRecoveryViewProps = {
  state: KsState;
  reason: string;
  resetOpen: boolean;
  busy: "archive" | "reset" | null;
  toasts: KsToastEntry[];
  onArchive: () => void;
  onReset: () => void; // Confirm "Confirm" -> reset_keystore
  onOpenReset: () => void; // Reset trigger button -> open the Confirm
  onCloseReset: () => void; // Confirm "Cancel" + backdrop -> close
  onDismissToast: (id: number) => void;
  /** Optional override ref for the Reset trigger (focus restore on Confirm
   *  close). The production mount supplies one; the lab fixture omits it. */
  resetTriggerRef?: { current?: HTMLElement };
};

export const KeystoreRecoveryView: Component<KeystoreRecoveryViewProps> = (
  props,
) => {
  const locale = detectLocale();
  const t = SETTINGS_COPY[locale].keystore;
  const description = () => t.description.replace("{reason}", props.reason);

  return (
    <section class="keystore-recovery" aria-label={t.title}>
      <Show when={props.state === "corrupt"}>
        <Banner
          variant="destructive"
          title={t.title}
          description={description()}
          action={
            <span class="keystore-recovery__banner-actions">
              <Button
                variant="primary"
                size="sm"
                loading={props.busy === "archive"}
                onClick={props.onArchive}
              >
                {t.archive}
              </Button>
              <Button
                variant="destructive"
                size="sm"
                ref={(el: HTMLButtonElement) => {
                  if (props.resetTriggerRef) props.resetTriggerRef.current = el;
                }}
                onClick={props.onOpenReset}
              >
                {t.reset}
              </Button>
            </span>
          }
        />
      </Show>

      <Show when={props.state === "archived"}>
        <Banner variant="info" title={t.archivedTitle} description={t.archivedPrompt} />
      </Show>

      {/* Healthy: no banner; settings normal (the shell content target is the
          section itself). */}

      <Confirm
        open={props.resetOpen}
        // rev-8-7 (load-bearing): Confirm's onOpenChange passes (open: boolean).
        // Route open=true -> onOpenReset (opens the Confirm), open=false ->
        // onCloseReset (Cancel/backdrop). The previous `() => props.onCloseReset()`
        // form ignored the boolean and would close on every change.
        onOpenChange={(open) => (open ? props.onOpenReset() : props.onCloseReset())}
        variant="destructive"
        title={t.resetConfirmTitle}
        message={t.resetConfirmMessage}
        confirmLabel={t.resetConfirmConfirmLabel}
        cancelLabel={t.resetConfirmCancelLabel}
        onConfirm={props.onReset}
        onCancel={() => props.onCloseReset()}
        triggerRef={props.resetTriggerRef ?? {}}
      />

      <Show when={props.toasts.length > 0}>
        <div class="keystore-recovery__toasts" aria-live="polite">
          <For each={props.toasts}>
            {(entry) => (
              <Toast
                variant={entry.variant}
                message={entry.message}
                onDismiss={() => props.onDismissToast(entry.id)}
              />
            )}
          </For>
        </div>
      </Show>
    </section>
  );
};

// --- PRODUCTION MOUNT (controller) ---------------------------------------
// Owns the IPC + state signals + the resetOpen signal; renders
// <KeystoreRecoveryView .../>. The View stays presentational (no invoke).
const KeystoreRecovery: Component = () => {
  const locale = detectLocale();
  const t = SETTINGS_COPY[locale].keystore;

  const [state, setState] = createSignal<KsState>("healthy");
  const [reason, setReason] = createSignal("");
  const [resetOpen, setResetOpen] = createSignal(false);
  const [busy, setBusy] = createSignal<"archive" | "reset" | null>(null);
  const [toasts, setToasts] = createSignal<KsToastEntry[]>([]);

  // Ref to the Reset trigger button for Confirm focus restore on close.
  const resetTriggerRef: { current?: HTMLElement } = {};

  let toastSeq = 0;
  const pushToast = (variant: KsToastEntry["variant"], message: string) => {
    const id = ++toastSeq;
    setToasts((prev) => [...prev, { id, variant, message }]);
  };
  const dismissToast = (id: number) =>
    setToasts((prev) => prev.filter((tEntry) => tEntry.id !== id));

  onMount(() => {
    invoke<string>("keystore_health")
      .then((h) => {
        if (h === "" || h == null) {
          setState("healthy");
        } else {
          setState("corrupt");
          setReason(h);
        }
      })
      .catch((e: unknown) => {
        // A thrown keystore_health is itself a corrupt signal.
        setState("corrupt");
        setReason(String(e));
      });
  });

  const onArchive = async () => {
    setBusy("archive");
    try {
      await invoke<string>("archive_keystore");
      setState("archived");
    } catch (e: unknown) {
      pushToast("destructive", `${t.archiveFailed}: ${String(e)}`);
    } finally {
      setBusy(null);
    }
  };

  const onReset = async () => {
    setBusy("reset");
    try {
      await invoke<string | null>("reset_keystore");
      setState("archived");
      setResetOpen(false);
    } catch (e: unknown) {
      pushToast("destructive", `${t.resetFailed}: ${String(e)}`);
    } finally {
      setBusy(null);
    }
  };

  onCleanup(() => setToasts([]));

  return (
    <KeystoreRecoveryView
      state={state()}
      reason={reason()}
      resetOpen={resetOpen()}
      busy={busy()}
      toasts={toasts()}
      onArchive={onArchive}
      onReset={onReset}
      onOpenReset={() => setResetOpen(true)}
      onCloseReset={() => setResetOpen(false)}
      onDismissToast={dismissToast}
      resetTriggerRef={resetTriggerRef}
    />
  );
};

export default KeystoreRecovery;
