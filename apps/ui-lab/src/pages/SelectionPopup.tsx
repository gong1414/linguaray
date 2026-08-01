import {
  For,
  Show,
  createSignal,
  createMemo,
  createEffect,
  type Component,
} from "solid-js";
import { Copy, Volume2, Pin, PinOff, Star, AlertTriangle } from "lucide-solid";
import {
  ResultCard,
  Button,
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
 * All actions are interactive mock state with visible feedback:
 *  - Copy → "Copied" state (label swap) for 1.5s
 *  - Pin/Unpin → toggles aria-pressed + visual selected style
 *  - Favorite → toggles aria-pressed + filled star
 *  - Retry → loading 1.2s then jumps to success-single (observable transition)
 *  - Open settings → "Opening…" → "Settings opened (mock)" toast
 *
 * The popup is rendered inside a sized frame (400×300 / 600×400) controlled by
 * the shell. Loading and initial-hidden use a COMPACT mode (~200×40 / empty)
 * per S0 §4.1: "Small card at cursor: … spinner".
 */

// --- Mock data (lab only) -------------------------------------------------

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
  const [pinnedEngines, setPinnedEngines] = createSignal<Set<string>>(new Set());
  const [favoritedEngines, setFavoritedEngines] = createSignal<Set<string>>(
    new Set(),
  );
  const [retrying, setRetrying] = createSignal(false);
  const [retriedDone, setRetriedDone] = createSignal(false);
  const [settingsStatus, setSettingsStatus] = createSignal<
    "idle" | "opening" | "opened"
  >("idle");

  // Reset transient mock interaction state whenever the selected state changes,
  // so feedback from one state (e.g. a successful retry) does not leak into a
  // different state and mask its real appearance.
  createEffect(() => {
    props.state; // track
    setCopiedEngine(null);
    setRetrying(false);
    setRetriedDone(false);
    setSettingsStatus("idle");
    setPinnedEngines(new Set<string>());
    setFavoritedEngines(new Set<string>());
  });

  const isMulti = createMemo(
    () =>
      props.state === "success-dual" ||
      props.state === "success-multi" ||
      props.state === "partial",
  );

  // true when the state wants the full-size window frame; loading/hidden use
  // a compact card at the cursor instead.
  const isCompact = createMemo(
    () => props.state === "loading" || props.state === "initial-hidden",
  );

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
      case "offline-fallback":
        return [
          {
            engineId: "dict",
            engineLabel: t.engineFallback,
            text: props.locale === "zh" ? "fox n. 狐狸" : "狐狸 n. fox",
            outcome: "success" as ResultOutcome,
          },
        ];
      default:
        return [];
    }
  });

  // Retry transitions the single error → success after a delay.
  const effectiveState = createMemo<SelectionState>(() => {
    if (retrying()) return "loading";
    return props.state;
  });
  const effectiveResults = createMemo<MockResult[]>(() => {
    if (retrying()) return [];
    // If we just retried a single-result error, show success.
    const isSingleError =
      props.state === "error-network" ||
      props.state === "error-config-key" ||
      props.state === "error-config-401";
    if (isSingleError && settingsStatus() === "idle") {
      // After a successful retry we show a success result. We detect "retried"
      // via a dedicated signal to avoid conflating with settings.
      if (retriedDone()) {
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
    }
    return baseResults();
  });

  const togglePin = (engineId: string) => {
    setPinnedEngines((prev) => {
      const next = new Set(prev);
      if (next.has(engineId)) next.delete(engineId);
      else next.add(engineId);
      return next;
    });
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
    window.setTimeout(() => setCopiedEngine(null), 1500);
  };

  const doRetry = () => {
    setRetriedDone(false);
    setRetrying(true);
    window.setTimeout(() => {
      setRetrying(false);
      setRetriedDone(true);
    }, 1200);
  };

  const doOpenSettings = () => {
    setSettingsStatus("opening");
    window.setTimeout(() => setSettingsStatus("opened"), 1000);
  };

  const toActions = (m: MockResult): ResultAction[] => {
    if (m.outcome === "failure") return [];
    const isPinned = pinnedEngines().has(m.engineId) || props.state === "pinned";
    const isFav = favoritedEngines().has(m.engineId);
    const isCopied = copiedEngine() === m.engineId;
    return [
      {
        label: isCopied ? props.t.copied : props.t.copy,
        icon: <Copy size={14} />,
        onClick: () => doCopy(m.engineId),
      },
      {
        label: props.t.speak,
        icon: <Volume2 size={14} />,
      },
      {
        label: isPinned ? props.t.unpin : props.t.pin,
        icon: isPinned ? <PinOff size={14} /> : <Pin size={14} />,
        active: isPinned,
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
    | {
        title: string;
        retry?: boolean;
        settings?: boolean;
      }
    | null
  >(() => {
    if (retriedDone()) return null;
    const t = props.t;
    switch (effectiveState()) {
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
    >
      {/* Initial hidden */}
      <Show when={props.state === "initial-hidden"}>
        <div class="sel-popup__hidden">{props.t.initialHidden}</div>
      </Show>

      {/* Loading — compact card */}
      <Show when={effectiveState() === "loading"}>
        <div class="sel-popup__loading">
          <span class="sel-popup__loading-dot" aria-hidden="true" />
          <span>{props.t.loading}</span>
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

      {/* Success / multi: side-by-side ResultCards */}
      <Show when={effectiveResults().length > 0}>
        <div
          class="sel-popup__results"
          classList={{
            "sel-popup__results--dual":
              effectiveResults().length === 2,
            "sel-popup__results--scroll":
              effectiveResults().length >= 3,
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
