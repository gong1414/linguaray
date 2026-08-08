import { Show, type Component } from "solid-js";
import { Star } from "lucide-solid";
import "./HistoryRow.css";

export type HistoryRowLabels = {
  addFavorite: string;    // 如 "Add to favorites" / "添加到收藏"
  removeFavorite: string; // 如 "Remove from favorites" / "从收藏移除"
};
export type HistoryRowProps = {
  sourceText: string;
  resultPreview: string;
  timestamp: string;
  engineLabel: string;
  labels: HistoryRowLabels;
  favorite?: boolean;
  onToggleFavorite?: () => void;
  onClick?: () => void;
};

const HistoryRow: Component<HistoryRowProps> = (props) => {
  const inner = (
    <>
      <span class="history-row__source">{props.sourceText}</span>
      <span class="history-row__preview">{props.resultPreview}</span>
      <span class="history-row__meta">
        <span class="history-row__engine">{props.engineLabel}</span>
        <span class="history-row__time">{props.timestamp}</span>
      </span>
    </>
  );
  // 无 onClick 时渲染非交互 div；有 onClick 时渲染 button（原生 Enter/Space 自动支持）
  return (
    <div class="history-row">
      <Show when={props.onClick} fallback={
        <div class="history-row__content">{inner}</div>
      }>
        <button type="button" class="history-row__content" onClick={() => props.onClick?.()}>
          {inner}
        </button>
      </Show>
      <Show when={props.onToggleFavorite}>
        <button
          type="button"
          class="history-row__fav"
          aria-label={props.favorite ? props.labels.removeFavorite : props.labels.addFavorite}
          aria-pressed={props.favorite}
          onClick={() => props.onToggleFavorite?.()}
        >
          <Star size={16} fill={props.favorite ? "currentColor" : "none"} />
        </button>
      </Show>
    </div>
  );
};
export default HistoryRow;
