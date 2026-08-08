import { Show, type Component } from "solid-js";
import ResultCard, { type ResultAction } from "./ResultCard";
import Spinner from "./Spinner";
import Button from "./Button";
import "./TranslationCard.css";

/** MASTER §7 TranslationCard — state is a discriminated union. */
export type TranslationState =
  | { kind: "loading" }
  | { kind: "success"; text: string; elapsedMs: number }
  | { kind: "failure"; errorText: string };

export type TranslationCardLabels = {
  loadingLabel: string;
  failureText: string;
  retryLabel: string;
};

export type TranslationCardProps = {
  engineId: string;
  engineLabel: string;
  state: TranslationState;
  actions?: ResultAction[];
  labels: TranslationCardLabels;
  onRetry?: () => void;
};

const TranslationCard: Component<TranslationCardProps> = (props) => {
  // Solid 的 `<Show when={boolean}>` 不会收窄 JSX 子元素中 `props` 的联合类型，
  // 因此我们需要在 `<Show>` 内部对联合成员使用显式的类型守卫。
  const successState = () =>
    props.state.kind === "success" ? props.state : undefined;
  const failureState = () =>
    props.state.kind === "failure" ? props.state : undefined;

  return (
    <div class="translation-card">
      <div class="translation-card__result">
        <Show when={props.state.kind === "loading"}>
          <Spinner size={16} label={props.labels.loadingLabel} />
        </Show>

        <Show when={successState()}>
          {(s) => (
            <ResultCard
              engineId={props.engineId}
              engineLabel={props.engineLabel}
              outcome="success"
              text={s().text}
              elapsedMs={s().elapsedMs}
              actions={props.actions}
            />
          )}
        </Show>

        <Show when={failureState()}>
          {(s) => (
            <div class="translation-card__retry">
              {/* MASTER §7: labels.failureText introduces the error before the
                  error text itself (no hardcoded English). */}
              <p class="translation-card__failure-text">{props.labels.failureText}</p>
              <ResultCard
                engineId={props.engineId}
                engineLabel={props.engineLabel}
                outcome="failure"
                errorText={s().errorText}
              />
              <Show when={props.onRetry}>
                <Button variant="primary" size="sm" onClick={() => props.onRetry?.()}>
                  {props.labels.retryLabel}
                </Button>
              </Show>
            </div>
          )}
        </Show>
      </div>
    </div>
  );
};
export default TranslationCard;
