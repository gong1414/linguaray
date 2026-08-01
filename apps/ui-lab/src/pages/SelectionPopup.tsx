import {
  For,
  Show,
  createSignal,
  createMemo,
  createEffect,
  onCleanup,
  type Component,
} from "solid-js";
import { Copy, Check, Volume2, Square, Pin, PinOff, Star, AlertTriangle } from "lucide-solid";
import {
  ResultCard,
  Button,
  Spinner,
  type ResultAction,
  type ResultOutcome,
} from "@linguaray/ui";
import type { Locale, LabStrings, SelectionState } from "../i18n";
import "./SelectionPopup.css";

export type SelectionPopupProps = {
  state: SelectionState;
  locale: Locale;
  t: LabStrings["selection"];
};

/**
 * S0 §4.1 Selection Popup — complete state matrix.
 *
 * Async-safety: every delayed mock callback (copy revert, retry→success,
 * settings-open) is guarded by a generation token that increments on state
 * change, AND its timer ID is tracked and cleared on state change + onCleanup.
 * An old callback whose generation no longer matches exits without touching
 * state, so a retry fired on "network error" can never overwrite a later
 * switch to "401".
 *
 * Pinned: the initial pinned value derives from state==="pinned" but is stored
 * in an override map the user can flip, so Unpin actually works.
 */

type MockResult = {
  engineId: string;
  engineLabel: string;
  text?: string;
  elapsedMs?: number;
  outcome: ResultOutcome;
  errorText?: string;
};

const enText = "The quick brown fox jumps over the lazy dog.";
const zhText = "敏捷的棕色狐狸跳过了懒狗。";

const SelectionPopup: Component<SelectionPopupProps> = (props) => {
  // --- interactive mock state ---
  const [copiedEngine, setCopiedEngine] = createSignal<string | null>(null);
  // pin/favorite overrides: undefined = use state default; true/false = user choice
  const [pinOverride, setPinOverride] = createSignal<Record<string, boolean>>({});
  const [favoritedEngines, setFavoritedEngines] = createSignal<Set<string>>(
    new Set(),
  );
  const [speakingEngine, setSpeakingEngine] = createSignal<string | null>(null);
  const [retrying, setRetrying] = createSignal(false);
  const [retriedDone, setRetriedDone] = createSignal(false);
  const [settingsStatus, setSettingsStatus] = createSignal<
    "idle" | "opening" | "opened"
  >("idle");

  // --- async-safety: generation token + tracked timer IDs ---
  let generation = 0;
  const timers = new Set<number>();

  const schedule = (fn: () => void, ms: number): void => {
    const myGen = generation;
    const id = window.setTimeout(() => {
      timers.delete(id);
      // Stale guard: if the state changed since this was scheduled, do nothing.
      if (myGen !== generation) return;
      fn();
    }, ms);
    timers.add(id);
  };

  const clearAllTimers = (): void => {
    for (const id of timers) window.clearTimeout(id);
    timers.clear();
  };

  // On state change: bump generation (pending callbacks self-invalidate),
  // cancel tracked timers, and reset transient mock signals. This runs before
  // render so a stale retry callback can never overwrite the new state.
  createEffect(() => {
    void props.state; // track
    generation += 1;
    clearAllTimers();
    setCopiedEngine(null);
    setSpeakingEngine(null);
    setRetrying(false);
    setRetriedDone(false);
    setSettingsStatus("idle");
    setPinOverride({});
    setFavoritedEngines(new Set<string>());
  });

  // Also cancel timers if the component is destroyed.
  onCleanup(() => clearAllTimers());

  const isPinnedFor = (engineId: string): boolean => {
    const ov = pinOverride();
    if (engineId in ov) return ov[engineId];
    return props.state === "pinned";
  };

  const isMulti = createMemo(
    () =>
      props.state === "success-dual" ||
      props.state === "success-multi" ||
      props.state === "partial",
  );

  // true when the body should fill a compact ~200×40 frame (loading only;
  // initial-hidden is not rendered by App at all).
  const isCompact = createMemo(() => props.state === "loading");

  const baseResults = createMemo<MockResult[]>(() => {
    const t = props.t;
    const toEn = () => (props.locale === "zh" ? enText : zhText);
    switch (props.state) {
      case "success-single":
      case "pinned":
        return [
          {
            engineId: "deepseek",
            engineLabel: t.engineA,
            text: toEn(),
            elapsedMs: 420,
            outcome: "success" as ResultOutcome,
          },
        ];
      case "success-dual":
        return [
          { engineId: "deepseek", engineLabel: t.engineA, text: toEn(), elapsedMs: 410, outcome: "success" as ResultOutcome },
          { engineId: "openai", engineLabel: t.engineB, text: props.locale === "zh" ? "A quick brown fox leaps over a lazy dog." : "敏捷的棕色狐狸跃过了懒狗。", elapsedMs: 680, outcome: "success" as ResultOutcome },
        ];
      case "success-multi":
        return [
          { engineId: "deepseek", engineLabel: t.engineA, text: toEn(), elapsedMs: 410, outcome: "success" as ResultOutcome },
          { engineId: "openai", engineLabel: t.engineB, text: props.locale === "zh" ? "A quick brown fox leaps over a lazy dog." : "敏捷的棕色狐狸跃过了懒狗。", elapsedMs: 680, outcome: "success" as ResultOutcome },
          { engineId: "google", engineLabel: t.engineC, text: props.locale === "zh" ? "The fast brown fox jumps over the lazy dog." : "敏捷的棕狐越过那只懒狗。", elapsedMs: 290, outcome: "success" as ResultOutcome },
        ];
      case "partial":
        return [
          { engineId: "deepseek", engineLabel: t.engineA, text: toEn(), elapsedMs: 410, outcome: "success" as ResultOutcome },
          { engineId: "openai", engineLabel: t.engineB, outcome: "failure" as ResultOutcome, errorText: t.networkError },
          { engineId: "google", engineLabel: t.engineC, text: props.locale === "zh" ? "The fast brown fox jumps over the lazy dog." : "敏捷的棕狐越过那只懒狗。", elapsedMs: 290, outcome: "success" as ResultOutcome },
        ];
      // §4.1 Offline: if a traditional engine is available → fallback result.
      // This is a selected traditional MT engine (Google), NOT a dictionary
      // lookup (dictionary is a separate capability per S0 §4.13).
      case "offline-fallback":
        return [
          {
            engineId: "google",
            engineLabel: `${t.engineC} · ${t.fallbackSuffix}`,
            text: props.locale === "zh" ? "The quick brown fox jumps over the lazy dog." : "敏捷的棕色狐狸跳过了懒狗。",
            elapsedMs: 120,
            outcome: "success" as ResultOutcome,
          },
        ];
      default:
        return [];
    }
  });

  // Retry behavior differs by state:
  //  - single-error states (network/config-key/config-401): retry clears the
  //    error content while loading, then swaps to a success card.
  //  - success states including Pinned: retry KEEPS the existing result cards
  //    visible (the pinned bar + cards must remain, only the Retry button
  //    shows loading). Returning [] here would blank the whole popup.
  const isSingleErrorState = () =>
    props.state === "error-network" ||
    props.state === "error-config-key" ||
    props.state === "error-config-401";

  const effectiveResults = createMemo<MockResult[]>(() => {
    // Only single-error retry blanks content; success/pinned retry preserves it.
    if (retrying() && isSingleErrorState()) return [];
    if (isSingleErrorState() && retriedDone()) {
      return [
        {
          engineId: "deepseek",
          engineLabel: props.t.engineA,
          text: props.locale === "zh" ? enText : zhText,
          elapsedMs: 380,
          outcome: "success" as ResultOutcome,
        },
      ];
    }
    return baseResults();
  });

  const togglePin = (engineId: string) => {
    setPinOverride((prev) => ({
      ...prev,
      [engineId]: !isPinnedFor(engineId),
    }));
  };

  const toggleFavorite = (engineId: string) => {
    setFavoritedEngines((prev) => {
      const next = new Set(prev);
      if (next.has(engineId)) next.delete(engineId);
      else next.add(engineId);
      return next;
    });
  };

  const doCopy = (engineId: string) => {
    setCopiedEngine(engineId);
    schedule(() => setCopiedEngine(null), 1500);
  };

  const toggleSpeak = (engineId: string) => {
    setSpeakingEngine((prev) => (prev === engineId ? null : engineId));
  };

  const doRetry = () => {
    setRetriedDone(false);
    setRetrying(true);
    schedule(() => {
      setRetrying(false);
      setRetriedDone(true);
    }, 1200);
  };

  const doOpenSettings = () => {
    setSettingsStatus("opening");
    schedule(() => setSettingsStatus("opened"), 1000);
  };

  const toActions = (m: MockResult): ResultAction[] => {
    if (m.outcome === "failure") return [];
    const pinned = isPinnedFor(m.engineId);
    const isFav = favoritedEngines().has(m.engineId);
    const isCopied = copiedEngine() === m.engineId;
    const isSpeaking = speakingEngine() === m.engineId;
    return [
      {
        label: isCopied ? props.t.copied : props.t.copy,
        icon: isCopied ? <Check size={14} /> : <Copy size={14} />,
        active: isCopied,
        onClick: () => doCopy(m.engineId),
      },
      {
        label: isSpeaking ? props.t.stop : props.t.speak,
        icon: isSpeaking ? <Square size={14} /> : <Volume2 size={14} />,
        active: isSpeaking,
        onClick: () => toggleSpeak(m.engineId),
      },
      {
        label: pinned ? props.t.unpin : props.t.pin,
        icon: pinned ? <PinOff size={14} /> : <Pin size={14} />,
        active: pinned,
        onClick: () => togglePin(m.engineId),
      },
      {
        label: isFav ? props.t.favorited : props.t.favorite,
        icon: <Star size={14} />,
        active: isFav,
        onClick: () => toggleFavorite(m.engineId),
      },
    ];
  };

  // Single-card error states (no ResultCards).
  const singleError = createMemo<
    | { title: string; retry?: boolean; settings?: boolean }
    | null
  >(() => {
    if (retriedDone()) return null;
    const t = props.t;
    switch (props.state) {
      case "error-no-selection":
        return { title: t.noSelection };
      case "error-network":
        return { title: t.networkError, retry: true };
      case "error-config-key":
        return { title: t.configErrorKey, settings: true };
      case "error-config-401":
        return { title: t.configError401, settings: true };
      case "error-no-provider":
        return { title: t.noProvider, settings: true };
      case "error-no-permission":
        return { title: t.noPermission, settings: true };
      case "keystore-corrupt":
        return { title: t.keystoreCorrupt, settings: true };
      case "offline-error":
        return { title: t.offlineError };
      default:
        return null;
    }
  });

  return (
    <div
      class="sel-popup__body"
      classList={{ "sel-popup__body--compact": isCompact() }}
      data-multi={isMulti() ? "true" : undefined}
      role="region"
      aria-label={
        isMulti() ? props.t.multiTitle : props.t.states[props.state]
      }
      aria-busy={retrying() ? "true" : undefined}
    >
      {/* Loading — fills the compact ~200×40 frame. Uses the frozen Spinner
          (MASTER §7) so reduced-motion text fallback is consistent with the
          component package, not a second loading implementation here. */}
      <Show when={props.state === "loading"}>
        <div class="sel-popup__loading">
          <Spinner size={12} label={props.t.loading} />
        </div>
      </Show>

      {/* Settings-opened toast */}
      <Show when={settingsStatus() === "opened"}>
        <p class="sel-popup__toast" role="status">
          {props.t.settingsOpened}
        </p>
      </Show>

      {/* Single-card error states */}
      <Show when={singleError()}>
        {(err) => (
          <div class="sel-popup__error-card" role="alert">
            <AlertTriangle size={20} class="sel-popup__error-icon" />
            <p class="sel-popup__error-text">{err().title}</p>
            <div class="sel-popup__error-actions">
              <Show when={err().retry}>
                <Button
                  variant="secondary"
                  size="sm"
                  loading={retrying()}
                  loadingLabel={props.t.retrying}
                  onClick={doRetry}
                >
                  {props.t.retry}
                </Button>
              </Show>
              <Show when={err().settings}>
                <Button
                  variant="ghost"
                  size="sm"
                  loading={settingsStatus() === "opening"}
                  loadingLabel={props.t.openingSettings}
                  onClick={doOpenSettings}
                >
                  {props.t.goSettings}
                </Button>
              </Show>
            </div>
          </div>
        )}
      </Show>

      {/* Pinned retry affordance — rendered independently of the results Show
          so it stays visible even if results are momentarily empty. S0 §4.1
          requires Pinned to support copy/retry/TTS/favorite. */}
      <Show when={props.state === "pinned"}>
        <div class="sel-popup__pinned-bar">
          <Button
            variant="ghost"
            size="sm"
            loading={retrying()}
            loadingLabel={props.t.retrying}
            onClick={doRetry}
          >
            {props.t.retry}
          </Button>
        </div>
      </Show>

      {/* Success / multi: side-by-side ResultCards. */}
      <Show when={effectiveResults().length > 0}>
        <div
          class="sel-popup__results"
          classList={{
            "sel-popup__results--dual": effectiveResults().length === 2,
            "sel-popup__results--scroll": effectiveResults().length >= 3,
          }}
        >
          <For each={effectiveResults()}>
            {(m) => (
              <ResultCard
                engineId={m.engineId}
                engineLabel={m.engineLabel}
                text={m.text}
                elapsedMs={m.elapsedMs}
                outcome={m.outcome}
                errorText={m.errorText}
                actions={toActions(m)}
              />
            )}
          </For>
        </div>
      </Show>
    </div>
  );
};

export default SelectionPopup;
