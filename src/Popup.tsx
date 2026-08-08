import { For, Show, createMemo, type Component } from "solid-js";
import { Copy, Volume2, Pin, PinOff, Star, AlertTriangle } from "lucide-solid";
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

const Popup: Component = () => {
  detectLocale(); // resolve locale once on mount (t() reads it lazily)
  const ctrl = createPopupController();
  const state = ctrl.state;

  const isCompact = createMemo(() => state().kind === "loading");

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
  const isErrorShell = createMemo(() => {
    const k = state().kind;
    return k === "error" || k === "offline" || k === "no-selection" ||
      k === "no-permission" || k === "keystore-corrupt";
  });

  function textFor(uuid: string): string | undefined {
    const s = state();
    if (s.kind === "multi-success" || s.kind === "partial") {
      return s.results.find((r) => r.uuid === uuid)?.text;
    }
    if (s.kind === "single-success") return s.text;
    return undefined;
  }

  // Per-card action builders (copy/speak/pin/favorite). Stale-safe via the
  // controller's reaction: state changes re-run this component, so the
  // actions captured here always reflect the current state/pin.
  const buildActions = (uuid: string): ResultAction[] => {
    const isPinned = ctrl.pinned();
    return [
      {
        label: t("selection.action.copy"),
        icon: <Copy size={14} />,
        onClick: () => { void navigator.clipboard?.writeText(textFor(uuid) ?? ""); },
      },
      {
        label: t("selection.action.speak"),
        icon: <Volume2 size={14} />,
        onClick: () => { /* TTS hook: window.speechSynthesis if available */ },
      },
      {
        label: isPinned ? t("selection.action.unpin") : t("selection.action.pin"),
        icon: isPinned ? <PinOff size={14} /> : <Pin size={14} />,
        active: isPinned,
        onClick: () => (isPinned ? ctrl.unpin() : ctrl.pin()),
      },
      {
        label: t("selection.action.favorite"),
        icon: <Star size={14} />,
        onClick: () => { /* vocabulary IPC hook */ },
      },
    ];
  };

  const onKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Escape") { e.preventDefault(); void ctrl.dismiss(); }
  };

  return (
    <main
      class="container"
      classList={{ "container--compact": isCompact() }}
      role="region"
      aria-label={headlineKey(state())}
      aria-busy={state().kind === "loading" ? "true" : undefined}
      onKeyDown={onKeyDown}
      tabIndex={-1}
    >
      <Show when={state().kind === "loading"}>
        <div class="popup-loading">
          <Spinner size={12} label={t("selection.loading")} />
        </div>
      </Show>

      <Show when={single()} keyed>
        {(s) => (
          <ResultCard
            engineId={s.engine}
            engineLabel={s.engine}
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
                  engineLabel={r.engine}
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
                  <Button variant="ghost" size="sm" onClick={() => { /* open settings window */ }}>
                    {t("selection.action.retry")}
                  </Button>
                </Show>
              }>
                <Button variant="secondary" size="sm" onClick={() => void ctrl.retry()}>
                  {t("selection.action.retry")}
                </Button>
              </Show>
            }
          />
        </div>
      </Show>
    </main>
  );
};

export default Popup;
