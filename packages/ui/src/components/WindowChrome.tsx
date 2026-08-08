import { Show, type Component, type JSX } from "solid-js";
import { Minus, X } from "lucide-solid";
import IconButton from "./IconButton";
import "./WindowChrome.css";

export type WindowChromeLabels = {
  minimize: string; // 如 "Minimize" / "最小化"
  close: string;    // 如 "Close" / "关闭"
};
export type WindowChromeProps = {
  title?: string;
  labels: WindowChromeLabels;
  children: JSX.Element;
  sidebar?: JSX.Element;
  onClose?: () => void;
  onMinimize?: () => void;
};

const WindowChrome: Component<WindowChromeProps> = (props) => {
  return (
    <div class="window-chrome">
      <Show when={props.sidebar}>
        <aside class="window-chrome__sidebar">{props.sidebar}</aside>
      </Show>
      <div class="window-chrome__main">
        <Show when={props.title || props.onClose || props.onMinimize}>
          <header class="window-chrome__header" data-tauri-drag-region>
            <Show when={props.title}>
              <h1 class="window-chrome__title">{props.title}</h1>
            </Show>
            <div class="window-chrome__controls">
              <Show when={props.onMinimize}>
                <IconButton
                  variant="ghost"
                  aria-label={props.labels.minimize}
                  data-tauri-drag-region="false"
                  onClick={props.onMinimize}
                >
                  <Minus aria-hidden="true" size={14} />
                </IconButton>
              </Show>
              <Show when={props.onClose}>
                <IconButton
                  variant="ghost"
                  aria-label={props.labels.close}
                  data-tauri-drag-region="false"
                  onClick={props.onClose}
                >
                  <X aria-hidden="true" size={14} />
                </IconButton>
              </Show>
            </div>
          </header>
        </Show>
        <main class="window-chrome__content">{props.children}</main>
      </div>
    </div>
  );
};
export default WindowChrome;
