import { type Component } from "solid-js";
// rev-8-2: the `@app` alias resolves to <repo>/src (see apps/ui-lab configs).
// The lab imports the PRODUCTION presentational View and feeds canned state —
// no second mock UI. This is the InputPanelView pattern applied to Surface 01.
import { PopupView } from "@app/Popup";
import type { SelectionState } from "../i18n";
import { labStateToTranslationState } from "./selectionStateMap";
import "./SelectionPopup.css";

export type SelectionPopupProps = {
  state: SelectionState;
};

// rev-7-7: FIXED canned data — NO invoke calls, NO controller, NO Tauri event
// subscriptions. The lab is a pure renderer of the production PopupView. The
// TranslationState payloads come from the parity map (selectionStateMap), so a
// production-state divergence surfaces here as a compile error.

// Production engineLabel resolves raw ids (uuid / template) to friendly names.
// The lab has no provider_list IPC, so we provide a static display-label map
// for the canned engine ids the parity map emits.
const ENGINE_LABELS: Record<string, string> = {
  deepseek: "DeepSeek",
  openai: "OpenAI",
  google: "Google",
  anthropic: "Anthropic",
  gemini: "Gemini",
  ollama: "Ollama",
};
const engineLabel = (raw: string): string => ENGINE_LABELS[raw] ?? raw;

// Retry is available whenever a SOURCE text is saved (production hasSource).
// The lab fixture has no real source, so mirror the contract: the loading +
// initial-hidden states have no source; every other state is "a source exists".
const NO_SOURCE: ReadonlySet<SelectionState> = new Set(["loading", "initial-hidden"]);

const noop = () => {};

const SelectionPopup: Component<SelectionPopupProps> = (props) => {
  return (
    <div class="sel-popup__body" data-testid="lab-root">
      <PopupView
        state={labStateToTranslationState(props.state)}
        pinned={props.state === "pinned"}
        hasSource={!NO_SOURCE.has(props.state)}
        engineLabel={engineLabel}
        onCopy={() => {}}
        onPin={noop}
        onUnpin={noop}
        onDismiss={noop}
        onRetry={noop}
        onOpenSettings={noop}
      />
    </div>
  );
};

export default SelectionPopup;
