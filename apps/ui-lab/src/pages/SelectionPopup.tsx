import { For, Show, type Component, createMemo } from "solid-js";
import { Copy, Volume2, Pin, PinOff, Star, AlertTriangle } from "lucide-solid";
import {
  ResultCard,
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
 * S0 §4.1 Selection Popup — all states.
 *
 * The popup is rendered inside a sized frame (400×300 single / 600×400 expanded)
 * controlled by the shell. This component owns the *content*: loading, single
 * success, side-by-side multi-engine, partial success, and all error states.
 *
 * Per MASTER §8.2: multi-engine cards are side-by-side in provider sort order,
 * do not jump as results arrive, and overflow horizontally if total width
 * exceeds the popup max.
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
  const isMulti = createMemo(
    () =>
      props.state === "success-dual" ||
      props.state === "success-multi" ||
      props.state === "partial",
  );

  const pinned = createMemo(() => props.state === "pinned");

  const results = createMemo<MockResult[]>(() => {
    const t = props.t;
    switch (props.state) {
      case "loading":
        return [];
      case "success-single":
        return [
          {
            engineId: "deepseek",
            engineLabel: t.engineA,
            text: props.locale === "zh" ? enText : zhText,
            elapsedMs: 420,
            outcome: "success" as ResultOutcome,
          },
        ];
      case "pinned":
        return [
          {
            engineId: "deepseek",
            engineLabel: t.engineA,
            text: props.locale === "zh" ? enText : zhText,
            elapsedMs: 380,
            outcome: "success" as ResultOutcome,
          },
        ];
      case "success-dual":
        return [
          {
            engineId: "deepseek",
            engineLabel: t.engineA,
            text: props.locale === "zh" ? enText : zhText,
            elapsedMs: 410,
            outcome: "success" as ResultOutcome,
          },
          {
            engineId: "openai",
            engineLabel: t.engineB,
            text: props.locale === "zh" ? "A quick brown fox leaps over a lazy dog." : "敏捷的棕色狐狸跃过了懒狗。",
            elapsedMs: 680,
            outcome: "success" as ResultOutcome,
          },
        ];
      case "success-multi":
        return [
          { engineId: "deepseek", engineLabel: t.engineA, text: props.locale === "zh" ? enText : zhText, elapsedMs: 410, outcome: "success" as ResultOutcome },
          { engineId: "openai", engineLabel: t.engineB, text: props.locale === "zh" ? "A quick brown fox leaps over a lazy dog." : "敏捷的棕色狐狸跃过了懒狗。", elapsedMs: 680, outcome: "success" as ResultOutcome },
          { engineId: "google", engineLabel: t.engineC, text: props.locale === "zh" ? "The fast brown fox jumps over the lazy dog." : "敏捷的棕狐越过那只懒狗。", elapsedMs: 290, outcome: "success" as ResultOutcome },
        ];
      case "partial":
        return [
          { engineId: "deepseek", engineLabel: t.engineA, text: props.locale === "zh" ? enText : zhText, elapsedMs: 410, outcome: "success" as ResultOutcome },
          { engineId: "openai", engineLabel: t.engineB, outcome: "failure" as ResultOutcome, errorText: t.networkError },
          { engineId: "google", engineLabel: t.engineC, text: props.locale === "zh" ? "The fast brown fox jumps over the lazy dog." : "敏捷的棕狐越过那只懒狗。", elapsedMs: 290, outcome: "success" as ResultOutcome },
        ];
      default:
        return [];
    }
  });

  const toActions = (m: MockResult): ResultAction[] => {
    if (m.outcome === "failure") return [];
    return [
      { label: props.t.copy, icon: <Copy size={14} /> },
      { label: props.t.speak, icon: <Volume2 size={14} /> },
      {
        label: pinned() ? props.t.unpin : props.t.pin,
        icon: pinned() ? <PinOff size={14} /> : <Pin size={14} />,
        active: pinned(),
      },
      { label: props.t.favorite, icon: <Star size={14} /> },
    ];
  };

  // Single-card error states (no ResultCards).
  const singleError = createMemo<{ title: string; retry?: boolean; settings?: boolean } | null>(() => {
    const t = props.t;
    switch (props.state) {
      case "error-no-selection":
        return { title: t.noSelection };
      case "error-network":
        return { title: t.networkError, retry: true };
      case "error-config":
        return { title: t.configError, settings: true };
      case "error-no-provider":
        return { title: t.noProvider, settings: true };
      case "error-no-permission":
        return { title: t.noPermission, settings: true };
      case "keystore-corrupt":
        return { title: t.keystoreCorrupt, settings: true };
      case "offline":
        return { title: t.offline };
      default:
        return null;
    }
  });

  return (
    <div
      class="sel-popup__body"
      data-multi={isMulti() ? "true" : undefined}
      role="region"
      aria-label={
        isMulti() ? props.t.multiTitle : props.t.states[props.state]
      }
    >
      {/* Loading */}
      <Show when={props.state === "loading"}>
        <div class="sel-popup__loading">
          <Spinner size={16} label={props.t.loading} />
        </div>
      </Show>

      {/* Single-card error states */}
      <Show when={singleError()}>
        {(err) => (
          <div class="sel-popup__error-card" role="alert">
            <AlertTriangle size={20} class="sel-popup__error-icon" />
            <p class="sel-popup__error-text">{err().title}</p>
            <div class="sel-popup__error-actions">
              <Show when={err().retry}>
                <button
                  type="button"
                  class="lr-btn lr-focusable lr-btn--secondary lr-btn--sm"
                >
                  {props.t.retry}
                </button>
              </Show>
              <Show when={err().settings}>
                <button
                  type="button"
                  class="lr-btn lr-focusable lr-btn--ghost lr-btn--sm"
                >
                  {props.t.goSettings}
                </button>
              </Show>
            </div>
          </div>
        )}
      </Show>

      {/* Success / multi: side-by-side ResultCards */}
      <Show when={results().length > 0}>
        <div
          class="sel-popup__results"
          classList={{ "sel-popup__results--scroll": isMulti() }}
        >
          <For each={results()}>
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
