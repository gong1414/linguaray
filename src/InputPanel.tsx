import { createSignal, createMemo, Show, For, type Component, type JSX } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { AlertTriangle } from "lucide-solid";
import { Button, InlineError, ResultCard, type ResultOutcome } from "@linguaray/ui";
import { decodeSessionResult } from "./features/translation/decode";
import { ensureProviderNameMap, engineLabel } from "./features/translation/inputController";
import { detectLocale, t } from "./i18n";
import type { SessionResultFE, TranslationState } from "./features/translation/types";
import "./App.css";

/** rev-7-3: pure presentational View. Shared by the production InputPanel mount
 * (src/InputPanel.tsx default export) + the ui-lab visual fixture
 * (apps/ui-lab/src/pages/InputPanel.tsx). No signals, no invoke, no localStorage. */
export type InputPanelViewProps = {
  text: string;
  state: TranslationState;
  idle: boolean;
  hasResult?: boolean;
  engineLabel?: (raw: string) => string;
  onText: (v: string) => void;
  onTranslate: () => void;
  onClear: () => void;
};

export function InputPanelView(props: InputPanelViewProps): JSX.Element {
  const labelOf = (raw: string) => (props.engineLabel ?? ((r: string) => r))(raw);
  // rev-8-6: showClear is a DERIVATION (a function of props), not a value read
  // once at mount — keeps it reactive in Solid's fine-grained model.
  const showClear = () => props.hasResult ?? false;

  const single = createMemo(() => {
    const s = props.state;
    return s.kind === "single-success" ? { engine: s.engine, text: s.text } : null;
  });
  const multi = createMemo(() => {
    const s = props.state;
    return s.kind === "multi-success" || s.kind === "partial" ? s.results : null;
  });
  const errorMessage = createMemo(() => {
    const s = props.state;
    if (s.kind === "error") {
      return s.sub === "network" ? t("selection.error.network")
        : s.sub === "config-key" ? t("selection.error.config.key")
        : s.sub === "config-401" ? t("selection.error.config.auth")
        : s.message;
    }
    if (s.kind === "offline") return t("input.error.offline");
    if (s.kind === "no-permission") return t("selection.error.noPermission");
    if (s.kind === "keystore-corrupt") return t("selection.error.keystore");
    return null;
  });

  const onKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      props.onTranslate();
    }
  };

  return (
    <main class="container" style={{ padding: "var(--space-3, 12px)" }}>
      <h2 class="input-title">{t("input.title")}</h2>
      <textarea
        rows={4}
        placeholder={t("input.placeholder")}
        value={props.text}
        disabled={!props.idle}
        onInput={(e) => props.onText(e.currentTarget.value)}
        onKeyDown={onKeyDown}
        aria-label={t("input.title")}
      />
      <div class="input-actions">
        <Button variant="secondary" size="md" onClick={props.onClear} disabled={!showClear()}>
          {t("input.action.clear")}
        </Button>
        <Button
          variant="primary"
          size="md"
          loading={!props.idle}
          loadingLabel={t("selection.loading")}
          onClick={props.onTranslate}
          disabled={!props.text.trim()}
        >
          {t("input.action.translate")}
        </Button>
      </div>

      <Show when={single()} keyed>
        {(s) => (
          <ResultCard
            engineId={s.engine}
            engineLabel={labelOf(s.engine)}
            text={s.text}
            outcome={"success" as ResultOutcome}
          />
        )}
      </Show>

      <Show when={multi()} keyed>
        {(results) => (
          <div class="input-results" data-multi="true">
            <For each={results}>
              {(r) => (
                <ResultCard
                  engineId={r.uuid}
                  engineLabel={labelOf(r.uuid)}
                  text={r.text ?? ""}
                  outcome={(r.ok ? "success" : "failure") as ResultOutcome}
                  errorText={r.errorText}
                />
              )}
            </For>
          </div>
        )}
      </Show>

      <Show when={errorMessage()} keyed>
        {(msg) => (
          <InlineError icon={<AlertTriangle size={16} />}>
            <span>{msg}</span>
          </InlineError>
        )}
      </Show>
    </main>
  );
}

const InputPanel: Component = () => {
  detectLocale();
  const [text, setText] = createSignal("");
  const [state, setState] = createSignal<TranslationState>({ kind: "loading" });
  const [idle, setIdle] = createSignal(true);
  const [hasResult, setHasResult] = createSignal(false);

  async function translate() {
    const value = text().trim();
    if (!value) return;
    setIdle(false);
    setState({ kind: "loading" });
    try {
      // B1: ensure the provider name map is loaded before rendering results so
      // the ResultCard engine labels resolve to friendly names synchronously.
      await ensureProviderNameMap();
      const res = await invoke<SessionResultFE>("translate_session", {
        req: { text: value, from: "auto", to: "" },
      });
      setState(decodeSessionResult(res));
      setHasResult(true);
    } catch (e) {
      setState({ kind: "error", sub: "generic", message: String(e) });
      setHasResult(true);
    } finally {
      setIdle(true);
    }
  }

  const clear = () => {
    setText("");
    setState({ kind: "loading" });
    setHasResult(false);
  };

  return (
    <InputPanelView
      text={text()}
      state={state()}
      idle={idle()}
      hasResult={hasResult()}
      engineLabel={engineLabel}
      onText={setText}
      onTranslate={translate}
      onClear={clear}
    />
  );
};

export default InputPanel;
