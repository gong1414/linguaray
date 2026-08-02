import { Show, For, type Component, type JSX } from "solid-js";
import IconButton from "./IconButton";
import "./ResultCard.css";

export type ResultOutcome = "success" | "failure";

export type ResultAction = {
  /** Accessible name for the icon button. */
  label: string;
  icon: JSX.Element;
  onClick?: () => void;
  /** Mark active (e.g. pinned/favorited). Visual only. */
  active?: boolean;
  disabled?: boolean;
};

export type ResultCardProps = {
  engineId: string;
  engineLabel: string;
  text?: string;
  elapsedMs?: number;
  outcome: ResultOutcome;
  errorText?: string;
  /** Actions rendered as ghost IconButtons at the card bottom. */
  actions?: ResultAction[];
  class?: string;
};

/**
 * MASTER §7 ResultCard.
 * Success: text in --text-base + engine label in --text-xs fg-muted.
 * Failure: error text in destructive-fg.
 */
const ResultCard: Component<ResultCardProps> = (props) => {
  return (
    <article
      class={`lr-result-card${props.class ? ` ${props.class}` : ""}`}
      data-outcome={props.outcome}
      data-engine={props.engineId}
    >
      <header class="lr-result-card__header">
        <span class="lr-result-card__engine">{props.engineLabel}</span>
        <Show when={props.outcome === "success" && props.elapsedMs != null}>
          <span class="lr-result-card__meta">{props.elapsedMs} ms</span>
        </Show>
      </header>

      <Show
        when={props.outcome === "success"}
        fallback={
          <p class="lr-result-card__error">
            {props.errorText ?? "Translation failed"}
          </p>
        }
      >
        <p class="lr-result-card__text">{props.text}</p>
      </Show>

      {/* Per MASTER §7: actions belong to the success state. Failure cards
          show only the error text, never action buttons. */}
      <Show when={props.outcome === "success" && props.actions && props.actions.length > 0}>
        <div class="lr-result-card__actions">
          <For each={props.actions}>
            {(a) => (
              <IconButton
                variant="ghost"
                size="sm"
                aria-label={a.label}
                aria-pressed={a.active ? "true" : undefined}
                disabled={a.disabled}
                onClick={() => a.onClick?.()}
              >
                {a.icon}
              </IconButton>
            )}
          </For>
        </div>
      </Show>
    </article>
  );
};

export default ResultCard;
