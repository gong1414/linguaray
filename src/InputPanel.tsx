import { createSignal, createMemo, Show, type Component } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { AlertTriangle } from "lucide-solid";
import { Button, InlineError, ResultCard, type ResultOutcome } from "@linguaray/ui";
import { decodeSessionResult } from "./features/translation/decode";
import { detectLocale, t } from "./i18n";
import type { SessionResultFE, TranslationState } from "./features/translation/types";
import "./App.css";

const InputPanel: Component = () => {
  detectLocale();
  const [text, setText] = createSignal("");
  const [state, setState] = createSignal<TranslationState>({ kind: "loading" });
  const [idle, setIdle] = createSignal(true); // false while a translation is in-flight
  const [hasResult, setHasResult] = createSignal(false); // a result has been shown (Clearable)

  async function translate() {
    const value = text().trim();
    if (!value) return;
    setIdle(false);
    setState({ kind: "loading" });
    try {
      // to: "" is the backend sentinel for "use settings.target_language".
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

  const onKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      void translate();
    }
  };

  // Narrowed snapshots (Solid re-runs reactively; capturing `s` locally lets
  // TS narrow, which two separate state() calls would not).
  const single = createMemo(() => {
    const s = state();
    return s.kind === "single-success" ? { engine: s.engine, text: s.text } : null;
  });
  const errorMessage = createMemo(() => {
    const s = state();
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

  return (
    <main class="container" style={{ padding: "var(--space-3, 12px)" }}>
      <h2 class="input-title">{t("input.title")}</h2>
      <textarea
        rows={4}
        placeholder={t("input.placeholder")}
        value={text()}
        disabled={!idle()}
        onInput={(e) => setText(e.currentTarget.value)}
        onKeyDown={onKeyDown}
        aria-label={t("input.title")}
      />
      <div class="input-actions">
        <Button variant="secondary" size="md" onClick={clear} disabled={!hasResult()}>
          {t("input.action.clear")}
        </Button>
        <Button
          variant="primary"
          size="md"
          loading={!idle()}
          loadingLabel={t("selection.loading")}
          onClick={() => void translate()}
          disabled={!text().trim()}
        >
          {t("input.action.translate")}
        </Button>
      </div>

      <Show when={single()} keyed>
        {(s) => (
          <ResultCard
            engineId={s.engine}
            engineLabel={s.engine}
            text={s.text}
            outcome={"success" as ResultOutcome}
          />
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
};

export default InputPanel;
