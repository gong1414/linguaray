import { For, createMemo, type Component, type JSX, splitProps } from "solid-js";
import "./SegmentedControl.css";

export type SegmentedOption = { value: string; label: string; icon?: JSX.Element };

export type SegmentedControlProps = {
  options: SegmentedOption[];
  value: string;
  onChange: (value: string) => void;
  ariaLabel: string; // 必填
  disabled?: boolean;
};

export const SegmentedControl: Component<SegmentedControlProps> = (props) => {
  const [, rest] = splitProps(props, ["options", "value", "onChange", "ariaLabel", "disabled"]);
  const currentIndex = createMemo(() =>
    Math.max(0, props.options.findIndex((o) => o.value === props.value)),
  );
  // 局部 ref 数组，存储 tab DOM 节点（禁止 document.querySelector 全局查询）
  let tabRefs: (HTMLButtonElement | undefined)[] = [];

  function activate(index: number) {
    if (props.disabled) return;
    const len = props.options.length;
    const wrapped = ((index % len) + len) % len;
    props.onChange(props.options[wrapped].value);
    // 用局部 ref 移动焦点
    tabRefs[wrapped]?.focus();
  }

  function onKeyDown(e: KeyboardEvent) {
    if (props.disabled) return;
    const i = currentIndex();
    switch (e.key) {
      case "ArrowRight":
      case "ArrowDown":
        e.preventDefault();
        activate(i + 1);
        break;
      case "ArrowLeft":
      case "ArrowUp":
        e.preventDefault();
        activate(i - 1);
        break;
      case "Home":
        e.preventDefault();
        activate(0);
        break;
      case "End":
        e.preventDefault();
        activate(props.options.length - 1);
        break;
    }
  }

  return (
    <div class="seg-control" role="radiogroup" aria-label={props.ariaLabel} {...rest}>
      <For each={props.options}>
        {(opt, index) => (
          <button
            type="button"
            role="radio"
            class="seg-control__tab"
            aria-checked={opt.value === props.value}
            tabindex={opt.value === props.value ? 0 : -1}
            disabled={props.disabled}
            ref={(el) => (tabRefs[index()] = el)}
            onClick={() => activate(index())}
            onKeyDown={onKeyDown}
          >
            {opt.icon}
            <span>{opt.label}</span>
          </button>
        )}
      </For>
    </div>
  );
};
export default SegmentedControl;
