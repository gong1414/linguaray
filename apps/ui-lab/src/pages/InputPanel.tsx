import { type Component } from "solid-js";
// rev-8-2: the `@app` alias resolves to <repo>/src (see apps/ui-lab configs).
import { InputPanelView, type InputPanelViewProps } from "@app/InputPanel";
import "./InputPanel.css";

export type InputState = "idle" | "multi" | "partial" | "error";

export type InputPanelProps = {
  state: InputState;
};

// rev-7-7: FIXED canned data — NO invoke calls. The lab is a pure renderer.
// The shape matches InputPanelViewProps (the production View's prop type).
// ResultEntry = { uuid, engine (REQUIRED), text?, errorText?, ok }.
const SAMPLE_TEXT = "The quick brown fox jumps over the lazy dog.";

const STATE_PROPS: Record<InputState, InputPanelViewProps> = {
  // rev-7-7: there is NO `idle` kind on TranslationState. "idle" (no in-flight
  // request) is represented as { kind: "loading" } + idle: true.
  idle: {
    text: SAMPLE_TEXT,
    state: { kind: "loading" },
    idle: true,
    hasResult: false,
    onText: () => {},
    onTranslate: () => {},
    onClear: () => {},
  },
  multi: {
    text: SAMPLE_TEXT,
    state: {
      kind: "multi-success",
      results: [
        { uuid: "openai", engine: "OpenAI", ok: true, text: "你好" },
        { uuid: "anthropic", engine: "Claude", ok: true, text: "您好" },
      ],
    },
    idle: true,
    hasResult: true,
    engineLabel: (raw: string) => raw,
    onText: () => {},
    onTranslate: () => {},
    onClear: () => {},
  },
  partial: {
    text: SAMPLE_TEXT,
    state: {
      kind: "partial",
      results: [
        { uuid: "openai", engine: "OpenAI", ok: true, text: "你好" },
        { uuid: "anthropic", engine: "Claude", ok: false, errorText: "config-401" },
      ],
    },
    idle: true,
    hasResult: true,
    engineLabel: (raw: string) => raw,
    onText: () => {},
    onTranslate: () => {},
    onClear: () => {},
  },
  error: {
    text: SAMPLE_TEXT,
    state: { kind: "error", sub: "network", message: "Network error — all engines failed" },
    idle: true,
    hasResult: true,
    onText: () => {},
    onTranslate: () => {},
    onClear: () => {},
  },
};

const InputPanel: Component<InputPanelProps> = (props) => {
  return (
    <div class="input-shell" data-testid="lab-root">
      <InputPanelView {...STATE_PROPS[props.state]} />
    </div>
  );
};

export default InputPanel;
