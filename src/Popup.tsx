import { For, Show, createMemo, createSignal, onCleanup, type Component, type JSX } from "solid-js";
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

/**
 * rev-7-3: pure presentational View for Surface 01 (selection popup). Shared by
 * the production Popup mount (src/Popup.tsx default export) + the ui-lab visual
 * fixture (apps/ui-lab/src/pages/SelectionPopup.tsx). No controller, no invoke,
 * no clipboard plugin — all side effects are delegated via callbacks. The only
 * signal-owned state is the Copy button's transient "Copied" feedback, which is
 * purely presentational UI feedback (never touches IPC/the controller).
 */
export type PopupViewProps = {
  state: TranslationState;
  pinned: boolean;
  hasSource: boolean;
  /** Resolve a raw engine id to a friendly label. */
  engineLabel: (raw: string) => string;
  /** Copy handler. The View passes the TRANSLATION text (never the source) and
   *  manages its own "Copied" feedback on resolution. */
  onCopy: (text: string) => void | Promise<void>;
  onPin: () => void;
  onUnpin: () => void;
  onDismiss: () => void;
  onRetry: () => void;
  /** Open the settings window to a named section (e.g. provider-center). */
  onOpenSettings: (section?: string) => void;
  /** Save the translation (and its source) to vocabulary. */
  onFavorite?: (translation: string) => void | Promise<void>;
  onSpeak?: (text: string) => void | Promise<void>;
  onStopSpeak?: () => void | Promise<void>;
};

export function PopupView(props: PopupViewProps): JSX.Element {
  const isCompact = createMemo(() => props.state.kind === "loading");

  // Copy feedback. copiedUuid is "__single__" for the single card, or the
  // engine uuid for a multi card. While set, that card's Copy button shows the
  // "Copied" label. This is presentational feedback only — the actual clipboard
  // write is delegated to props.onCopy.
  const [copiedUuid, setCopiedUuid] = createSignal<string | null>(null);
  const [favoritedUuid, setFavoritedUuid] = createSignal<string | null>(null);
  const [speakingUuid, setSpeakingUuid] = createSignal<string | null>(null);
  let copiedTimer: ReturnType<typeof setTimeout> | undefined;
  onCleanup(() => {
    if (copiedTimer) clearTimeout(copiedTimer);
  });

  // Narrowed snapshots for the single-success card. Derived from props.state.
  const single = createMemo(() => {
    const s = props.state;
    return s.kind === "single-success"
      ? { engine: s.engine, text: s.text }
      : null;
  });
  const multi = createMemo(() => {
    const s = props.state;
    return s.kind === "multi-success" || s.kind === "partial" ? s.results : null;
  });
  const errorState = createMemo(() => {
    const s = props.state;
    return s.kind === "error" ? s : null;
  });
  // keystore-corrupt gets its OWN dedicated Show with a recovery CTA, so it
  // is excluded from the generic error shell.
  const isErrorShell = createMemo(() => {
    const k = props.state.kind;
    return k === "error" || k === "offline" || k === "no-selection" ||
      k === "no-permission";
  });

  function textFor(uuid: string): string | undefined {
    const s = props.state;
    if (s.kind === "multi-success" || s.kind === "partial") {
      return s.results.find((r) => r.uuid === uuid)?.text;
    }
    if (s.kind === "single-success") return s.text;
    return undefined;
  }

  // Copy delegates to props.onCopy (the controller writes via the Tauri
  // clipboard plugin; the lab fixture passes a no-op). TTS stays
  // aria-disabled (focusable for discovery) because it is not yet shipped.
  const buildActions = (uuid: string): ResultAction[] => {
    const isPinned = props.pinned;
    const isCopied = copiedUuid() === uuid;
    const isFavorited = favoritedUuid() === uuid;
    return [
      {
        label: isCopied ? t("selection.action.copied") : t("selection.action.copy"),
        // When copied, surface the label as visible text (the IconButton is
        // icon-only, so the aria-label alone is not queryable by findByText).
        icon: isCopied
          ? <span class="popup-copy-copied">{t("selection.action.copied")}</span>
          : <Copy size={14} />,
        onClick: async () => {
          const translationText = textFor(uuid) ?? "";
          try {
            // Await the copy so a rejection (clipboard unavailable / denied) is
            // observed here rather than becoming an unhandled promise rejection.
            await props.onCopy(translationText);
            // Copy succeeded — show the Copied feedback.
            setCopiedUuid(uuid);
            if (copiedTimer) clearTimeout(copiedTimer);
            copiedTimer = setTimeout(() => setCopiedUuid(null), COPIED_FEEDBACK_MS);
          } catch {
            // Copy failed — do NOT show Copied (no false-positive feedback).
            // Best-effort: no error UI (clipboard may simply be unavailable).
          }
        },
      },
      {
        label: speakingUuid() === uuid ? t("selection.action.stop") : t("selection.action.speak"),
        icon: <Volume2 size={14} />,
        active: speakingUuid() === uuid,
        onClick: async () => {
          if (speakingUuid() === uuid) {
            await props.onStopSpeak?.();
            setSpeakingUuid(null);
            return;
          }
          const translation = textFor(uuid) ?? "";
          try {
            await props.onSpeak?.(translation);
            setSpeakingUuid(uuid);
          } catch {
            setSpeakingUuid(null);
          }
        },
      },
      {
        label: isPinned ? t("selection.action.unpin") : t("selection.action.pin"),
        icon: isPinned ? <PinOff size={14} /> : <Pin size={14} />,
        active: isPinned,
        onClick: () => (isPinned ? props.onUnpin() : props.onPin()),
      },
      {
        label: isFavorited ? t("selection.action.favorited") : t("selection.action.favorite"),
        icon: <Star size={14} fill={isFavorited ? "currentColor" : "none"} />,
        active: isFavorited,
        onClick: async () => {
          const translation = textFor(uuid) ?? "";
          try {
            await props.onFavorite?.(translation);
            setFavoritedUuid(uuid);
          } catch {
            // Keep the button unmarked when the save fails.
          }
        },
      },
    ];
  };

  const onKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Escape") { e.preventDefault(); props.onDismiss(); }
  };

  return (
    <section
      class="popup-shell"
      classList={{ "popup-shell--compact": isCompact() }}
      aria-label={headlineKey(props.state)}
      aria-busy={props.state.kind === "loading" ? "true" : undefined}
      onKeyDown={onKeyDown}
      tabIndex={-1}
    >
      <Show when={props.state.kind === "loading"}>
        <div class="popup-loading">
          <Spinner size={12} label={t("selection.loading")} />
          {/* B3/P1-8: Retry available whenever a SOURCE text is saved, including
              the loading state (re-translate the saved source via
              translate_selection_ipc — never the clipboard or the result). */}
          <Show when={props.hasSource}>
            <Button
              variant="ghost"
              size="sm"
              aria-label={t("selection.action.retry")}
              onClick={() => props.onRetry()}
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
            engineLabel={props.engineLabel(s.engine)}
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
                  engineLabel={props.engineLabel(r.engine)}
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
        (single() || multi()) && props.hasSource
      }>
        <div class="popup-retry">
          <Button
            variant="ghost"
            size="sm"
            aria-label={t("selection.action.retry")}
            onClick={() => props.onRetry()}
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
            title={headlineKey(props.state)}
            action={
              <Show when={errorState()?.sub === "network"} fallback={
                <Show when={
                  errorState()?.sub === "config-key" || errorState()?.sub === "config-401"
                }>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => props.onOpenSettings("provider-center")}
                  >
                    {t("selection.action.openSettings")}
                  </Button>
                </Show>
              }>
                <Show when={props.hasSource} fallback={<span />}>
                  <Button
                    variant="secondary"
                    size="sm"
                    aria-label={t("selection.action.retry")}
                    onClick={() => props.onRetry()}
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
      <Show when={props.state.kind === "keystore-corrupt"}>
        <div class="popup-error" role="alert">
          <EmptyState
            icon={<AlertTriangle size={32} />}
            title={t("selection.error.keystore")}
            action={
              <Button
                variant="ghost"
                size="sm"
                onClick={() => props.onOpenSettings("keystore-recovery")}
              >
                {t("selection.action.recovery")}
              </Button>
            }
          />
        </div>
      </Show>
    </section>
  );
}

/**
 * Production Popup controller. Owns the popup controller (state/pinned signals +
 * Tauri event subscriptions) and binds it to the presentational PopupView.
 */
const Popup: Component = () => {
  detectLocale(); // resolve locale once on mount (t() reads it lazily)
  const ctrl = createPopupController();

  return (
    <PopupView
      state={ctrl.state()}
      pinned={ctrl.pinned()}
      hasSource={ctrl.hasSource()}
      engineLabel={ctrl.engineLabel}
      onCopy={(text) => writeText(text)}
      onPin={() => ctrl.pin()}
      onUnpin={() => ctrl.unpin()}
      onDismiss={() => { void ctrl.dismiss(); }}
      onRetry={() => { void ctrl.retrySelection(); }}
      onOpenSettings={(section) =>
        void invoke("open_settings_window", section ? { section } : {})
      }
      onSpeak={async (text) => {
        await invoke("tts_speak", { text, voiceId: null });
      }}
      onStopSpeak={() => invoke("tts_stop")}
      onFavorite={async (translation) => {
        const source = ctrl.lastSource();
        await invoke("vocabulary_add", {
          word: source || translation,
          definition: translation,
          sourceLanguage: "auto",
          targetLanguage: detectLocale() === "zh" ? "zh" : "en",
        });
      }}
    />
  );
};

export default Popup;
