import { Show, type Component } from "solid-js";
import { X } from "lucide-solid";
import "./ShortcutChip.css";

export type ShortcutChipLabels = {
  recording: string;   // 录制中提示，如 "Recording…"
  conflict: string;    // 冲突提示，如 "Conflict"
  clear: string;       // 清除按钮 aria-label，如 "Clear shortcut"
};
export type ShortcutChipStatus = "recording" | "conflict" | "clear";
export type ShortcutChipProps = {
  shortcut: string;
  status: ShortcutChipStatus;
  labels: ShortcutChipLabels;
  onClear?: () => void;
  disabled?: boolean;
};

const ShortcutChip: Component<ShortcutChipProps> = (props) => {
  return (
    <span
      class="shortcut-chip"
      classList={{
        "shortcut-chip--recording": props.status === "recording",
        "shortcut-chip--conflict": props.status === "conflict",
        "shortcut-chip--disabled": props.disabled,
      }}
      role="status"
      aria-live="polite"
    >
      <kbd class="shortcut-chip__keys">
        {props.status === "recording" ? props.labels.recording : props.shortcut}
      </kbd>
      <Show when={props.status === "conflict"}>
        <span class="shortcut-chip__conflict-text">{props.labels.conflict}</span>
      </Show>
      <Show when={props.onClear && !props.disabled}>
        <button
          type="button"
          class="shortcut-chip__clear"
          aria-label={props.labels.clear}
          onClick={() => props.onClear?.()}
        >
          <X size={14} />
        </button>
      </Show>
    </span>
  );
};
export default ShortcutChip;
