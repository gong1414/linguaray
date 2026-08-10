import { For, Show, createMemo, createSignal, onCleanup, type Component } from "solid-js";
import { Copy, Volume2, Pin, PinOff, Star, AlertTriangle } from "lucide-solid";
import { invoke } from "@tauri-apps/api/core";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import {
  Button,
  EmptyState,
  ResultCard,
  Spinner,
  type ResultAction,
  type ResultOutcome,
} from "@linguaray/ui";
import { createPopupController } from "./features/translation/popupController";
import { detectLocale, t } from "./i18n";
import type { TranslationState } from "./features/translation/types";
import "./Popup.css";
import "./App.css";

/** Map a TranslationState kind onto the headline copy key for aria-label. */
function headlineKey(s: TranslationState): string {
  switch (s.kind) {
    case "loading": return t("selection.loading");
    case "single-success":
    case "multi-success": return t("selection.multi.title");
    case "partial": return t("selection.multi.title");
    case "error":
      switch (s.sub) {
        case "network": return t("selection.error.network");
        case "config-key": return t("selection.error.config.key");
        case "config-401": return t("selection.error.config.auth");
        default: return s.message;
      }
    case "offline": return t("selection.error.offline");
    case "no-selection": return t("selection.error.noSelection");
    case "no-permission": return t("selection.error.noPermission");
    case "keystore-corrupt": return t("selection.error.keystore");
  }
}

/** How long the Copy button shows its "Copied" feedback (ms). */
const COPIED_FEEDBACK_MS = 1200;

const Popup: Component = () => {
  detectLocale(); // resolve locale once on mount (t() reads it lazily)
  const ctrl = createPopupController();
  const state = ctrl.state;

  const isCompact = createMemo(() => state().kind === "loading");

  // P1-3 + B4: Copy feedback. copiedUuid is "__single__" for the single card,
  // or the engine uuid for a multi card. While set, that card's Copy button
  // shows the "Copied" label.
  const [copiedUuid, setCopiedUuid] = createSignal<string | null>(null);
  let copiedTimer: ReturnType<typeof setTimeout> | undefined;
  onCleanup(() => {
    if (copiedTimer) clearTimeout(copiedTimer);
  });

  /** B4/P1-5: open the settings window to a named section. */
  const openSettings = (section?: string) =>
    invoke("open_settings_window", section ? { section } : {});

  // Narrowed snapshots for the single-success card. Solid re-runs these memos
  // reactively; capturing `s` locally lets TS narrow within each branch (two
  // separate `state()` calls would not narrow).
  const single = createMemo(() => {
    const s = state();
    return s.kind === "single-success"
      ? { engine: s.engine, text: s.text }
      : null;
  });
  const multi = createMemo(() => {
    const s = state();
    return s.kind === "multi-success" || s.kind === "partial" ? s.results : null;
  });
  const errorState = createMemo(() => {
    const s = state();
    return s.kind === "error" ? s : null;
  });
  // B4: keystore-corrupt gets its OWN dedicated Show with a recovery CTA, so it
  // is excluded from the generic error shell.
  const isErrorShell = createMemo(() => {
    const k = state().kind;
    return k === "error" || k === "offline" || k === "no-selection" ||
      k === "no-permission";
  });

  function textFor(uuid: string): string | undefined {
    const s = state();
    if (s.kind === "multi-success" || s.kind === "partial") {
      return s.results.find((r) => r.uuid === uuid)?.text;
    }
    if (s.kind === "single-success") return s.text;
    return undefined;
  }

  // B4: Copy via the Tauri clipboard plugin (no navigator.clipboard fallback —
  // the webview may not expose it, and the plugin is the supported path).
  // Writes the TRANSLATION text, never the source. TTS/Favorite are
  // aria-disabled (focusable for discovery, not natively disabled) because they
  // are not yet implemented.
  const buildActions = (uuid: string): ResultAction[] => {
    const isPinned = ctrl.pinned();
    const isCopied = copiedUuid() === uuid;
    return [
      {
        label: isCopied ? t("selection.action.copied") : t("selection.action.copy"),
        // When copied, surface the label as visible text (the IconButton is
        // icon-only, so the aria-label alone is not queryable by findByText).
        icon: isCopied
          ? <span class="popup-copy-copied">{t("selection.action.copied")}</span>
          : <Copy size={14} />,
        onClick: () => {
          const translationText = textFor(uuid) ?? "";
          void writeText(translationText).then(() => {
            setCopiedUuid(uuid);
            if (copiedTimer) clearTimeout(copiedTimer);
            copiedTimer = setTimeout(() => setCopiedUuid(null), COPIED_FEEDBACK_MS);
          });
        },
      },
      {
        label: t("selection.action.comingTts"),
        icon: <Volume2 size={14} />,
        ariaDisabled: true,
        onClick: () => { /* TTS: not yet shipped */ },
      },
      {
        label: isPinned ? t("selection.action.unpin") : t("selection.action.pin"),
        icon: isPinned ? <PinOff size={14} /> : <Pin size={14} />,
        active: isPinned,
        onClick: () => (isPinned ? ctrl.unpin() : ctrl.pin()),
      },
      {
        label: t("selection.action.comingFavorite"),
        icon: <Star size={14} />,
        ariaDisabled: true,
        onClick: () => { /* vocabulary IPC: not yet shipped */ },
      },
    ];
  };

  const onKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Escape") { e.preventDefault(); void ctrl.dismiss(); }
  };

  return (
    <main
      class="popup-shell"
      classList={{ "popup-shell--compact": isCompact() }}
      role="region"
      aria-label={headlineKey(state())}
      aria-busy={state().kind === "loading" ? "true" : undefined}
      onKeyDown={onKeyDown}
      tabIndex={-1}
    >
      <Show when={state().kind === "loading"}>
        <div class="popup-loading">
          <Spinner size={12} label={t("selection.loading")} />
          {/* B3/P1-8: Retry available whenever a SOURCE text is saved, including
              the loading state (re-translate the saved source via
              translate_selection_ipc — never the clipboard or the result). */}
          <Show when={ctrl.hasSource()}>
            <Button
              variant="ghost"
              size="sm"
              aria-label={t("selection.action.retry")}
              onClick={() => void ctrl.retrySelection()}
            >
              {t("selection.action.retry")}
            </Button>
          </Show>
        </div>
      </Show>

      <Show when={single()} keyed>
        {(s) => (
          <ResultCard
            engineId={s.engine}
            engineLabel={ctrl.engineLabel(s.engine)}
            text={s.text}
            outcome={"success" as ResultOutcome}
            actions={buildActions("__single__")}
          />
        )}
      </Show>

      <Show when={multi()} keyed>
        {(results) => (
          <div class="popup-results" data-multi="true">
            <For each={results}>
              {(r) => (
                <ResultCard
                  engineId={r.uuid}
                  engineLabel={ctrl.engineLabel(r.engine)}
                  text={r.text}
                  outcome={(r.ok ? "success" : "failure") as ResultOutcome}
                  errorText={r.errorText}
                  actions={r.ok ? buildActions(r.uuid) : undefined}
                />
              )}
            </For>
          </div>
        )}
      </Show>

      {/* P1-3: Retry re-translates the saved SOURCE text via
          translate_selection_ipc (never translate_clipboard, never the result).
          Shown whenever the controller has a source: in the success shell and
          (for network errors) in the error shell below. */}
      <Show when={
        (single() || multi()) && ctrl.hasSource()
      }>
        <div class="popup-retry">
          <Button
            variant="ghost"
            size="sm"
            aria-label={t("selection.action.retry")}
            onClick={() => void ctrl.retrySelection()}
          >
            {t("selection.action.retry")}
          </Button>
        </div>
      </Show>

      {/* Single-card error / special states (no ResultCard grid). */}
      <Show when={isErrorShell()}>
        <div class="popup-error" role="alert">
          <EmptyState
            icon={<AlertTriangle size={32} />}
            title={headlineKey(state())}
            action={
              <Show when={errorState()?.sub === "network"} fallback={
                <Show when={
                  errorState()?.sub === "config-key" || errorState()?.sub === "config-401"
                }>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => void openSettings("provider-center")}
                  >
                    {t("selection.action.openSettings")}
                  </Button>
                </Show>
              }>
                <Show when={ctrl.hasSource()} fallback={<span />}>
                  <Button
                    variant="secondary"
                    size="sm"
                    aria-label={t("selection.action.retry")}
                    onClick={() => void ctrl.retrySelection()}
                  >
                    {t("selection.action.retry")}
                  </Button>
                </Show>
              </Show>
            }
          />
        </div>
      </Show>

      {/* B4: keystore-corrupt gets its OWN dedicated recovery CTA (distinct from
          the generic error shell so the wording targets keystore recovery). */}
      <Show when={state().kind === "keystore-corrupt"}>
        <div class="popup-error" role="alert">
          <EmptyState
            icon={<AlertTriangle size={32} />}
            title={t("selection.error.keystore")}
            action={
              <Button
                variant="ghost"
                size="sm"
                onClick={() => void openSettings("keystore-recovery")}
              >
                {t("selection.action.recovery")}
              </Button>
            }
          />
        </div>
      </Show>
    </main>
  );
};

export default Popup;
