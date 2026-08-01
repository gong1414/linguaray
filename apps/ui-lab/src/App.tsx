import {
  For,
  Show,
  createSignal,
  createMemo,
  createEffect,
  type Component,
} from "solid-js";
import { strings, type Locale, type SelectionState } from "./i18n";
import SelectionPopup from "./pages/SelectionPopup";
import "./App.css";

type Theme = "light" | "dark";
type Motion = "full" | "reduced";
type WindowSize = { w: number; h: number; key: "single" | "expanded"; label: string };

type NavKey =
  | "selection-popup"
  | "input-window"
  | "provider-center"
  | "ocr-overlay"
  | "history"
  | "onboarding"
  | "multi-result"
  | "shortcuts"
  | "privacy"
  | "keystore"
  | "vocabulary"
  | "dictionary"
  | "tts"
  | "external-api"
  | "updater";

const SELECTION_STATES: SelectionState[] = [
  "loading",
  "success-single",
  "success-dual",
  "success-multi",
  "partial",
  "error-network",
  "error-config",
  "error-no-selection",
  "error-no-provider",
  "error-no-permission",
  "keystore-corrupt",
  "offline",
  "pinned",
];

const NAV_ITEMS: { key: NavKey; labelKey: keyof (typeof strings)["en"]["nav"] }[] = [
  { key: "selection-popup", labelKey: "selectionPopup" },
  { key: "input-window", labelKey: "inputWindow" },
  { key: "provider-center", labelKey: "providerCenter" },
  { key: "ocr-overlay", labelKey: "ocrOverlay" },
  { key: "history", labelKey: "history" },
  { key: "onboarding", labelKey: "onboarding" },
  { key: "multi-result", labelKey: "multiResult" },
  { key: "shortcuts", labelKey: "shortcuts" },
  { key: "privacy", labelKey: "privacy" },
  { key: "keystore", labelKey: "keystore" },
  { key: "vocabulary", labelKey: "vocabulary" },
  { key: "dictionary", labelKey: "dictionary" },
  { key: "tts", labelKey: "tts" },
  { key: "external-api", labelKey: "externalApi" },
  { key: "updater", labelKey: "updater" },
];

// Only selection-popup is implemented in this first vertical slice; the rest
// are nav placeholders for upcoming slices.
const IMPLEMENTED: NavKey[] = ["selection-popup"];

const App: Component = () => {
  const [locale, setLocale] = createSignal<Locale>("en");
  const [theme, setTheme] = createSignal<Theme>("light");
  const [motion, setMotion] = createSignal<Motion>("full");
  const [nav, setNav] = createSignal<NavKey>("selection-popup");
  const [selState, setSelState] = createSignal<SelectionState>("success-single");

  const t = createMemo(() => strings[locale()]);
  const selT = createMemo(() => t().selection);

  // Window size follows MASTER §8.2: multi-engine states use the expanded max.
  const isMultiState = () =>
    selState() === "success-dual" ||
    selState() === "success-multi" ||
    selState() === "partial";
  const windowSize = createMemo<WindowSize>(() =>
    isMultiState()
      ? { w: 600, h: 400, key: "expanded", label: "600×400" }
      : { w: 400, h: 300, key: "single", label: "400×300" },
  );

  // Sync theme + motion onto <html> so token + base rules apply globally.
  createEffect(() => {
    const html = document.documentElement;
    html.setAttribute("data-theme", theme());
    html.setAttribute("data-motion", motion());
    html.setAttribute("lang", locale());
  });

  return (
    <div class="lab lr-u-surface">
      <header class="lab__header">
        <div class="lab__brand">
          <span class="lab__title">{t().appTitle}</span>
          <span class="lab__subtitle">{t().appSubtitle}</span>
        </div>

        <div class="lab__controls">
          <div class="lab__control">
            <span class="lab__control-label">{t().controls.locale}</span>
            <Segmented
              value={locale()}
              options={[
                { value: "en", label: "EN" },
                { value: "zh", label: "中文" },
              ]}
              onChange={(v) => setLocale(v as Locale)}
            />
          </div>

          <div class="lab__control">
            <span class="lab__control-label">{t().controls.theme}</span>
            <Segmented
              value={theme()}
              options={[
                { value: "light", label: t().controls.themeLight },
                { value: "dark", label: t().controls.themeDark },
              ]}
              onChange={(v) => setTheme(v as Theme)}
            />
          </div>

          <div class="lab__control">
            <span class="lab__control-label">{t().controls.motion}</span>
            <Segmented
              value={motion()}
              options={[
                { value: "full", label: t().controls.motionFull },
                { value: "reduced", label: t().controls.motionReduced },
              ]}
              onChange={(v) => setMotion(v as Motion)}
            />
          </div>

          <div class="lab__control">
            <span class="lab__control-label">{t().controls.windowSize}</span>
            <Segmented
              value={windowSize().key}
              options={[
                { value: "single", label: t().controls.size400x300 },
                { value: "expanded", label: t().controls.size600x400 },
              ]}
              onChange={(v) => {
                // Manual size override: jump to a representative state.
                if (v === "expanded") setSelState("success-dual");
                else setSelState("success-single");
              }}
            />
          </div>
        </div>
      </header>

      <nav class="lab__nav" aria-label="Prototypes">
        <ul class="lab__nav-list">
          <For each={NAV_ITEMS}>
            {(item) => (
              <li>
                <button
                  class="lr-focusable"
                  aria-current={nav() === item.key ? "page" : undefined}
                  disabled={!IMPLEMENTED.includes(item.key)}
                  onClick={() => setNav(item.key)}
                  title={
                    IMPLEMENTED.includes(item.key)
                      ? undefined
                      : "Upcoming slice"
                  }
                >
                  {t().nav[item.labelKey]}
                  <Show when={!IMPLEMENTED.includes(item.key)}>
                    <span class="lr-visually-hidden"> (not yet implemented)</span>
                  </Show>
                </button>
              </li>
            )}
          </For>
        </ul>
      </nav>

      <main class="lab__main">
        <div class="lab__stage">
          <Show
            when={nav() === "selection-popup"}
            fallback={
              <div class="lab__frame-placeholder">
                <p>
                  {t().nav[
                    NAV_ITEMS.find((i) => i.key === nav())!.labelKey
                  ]}
                </p>
                <p class="lab__frame-placeholder-sub">Upcoming slice</p>
              </div>
            }
          >
            <div
              class="lab__frame"
              style={{
                width: `${windowSize().w}px`,
                height: `${windowSize().h}px`,
                "max-width": "100%",
              }}
            >
              <SelectionPopup
                state={selState()}
                locale={locale()}
                t={selT()}
              />
            </div>
            <span class="lab__frame-meta">
              {windowSize().label} · {t().selection.states[selState()]}
            </span>
          </Show>
        </div>

        <Show when={nav() === "selection-popup"}>
          <div class="lab__state-bar" role="group" aria-label={t().controls.state}>
            <span class="lab__state-label">{t().controls.state}</span>
            <For each={SELECTION_STATES}>
              {(s) => (
                <button
                  type="button"
                  class="lab__state-chip lr-focusable"
                  aria-pressed={selState() === s ? "true" : "false"}
                  onClick={() => setSelState(s)}
                >
                  {t().selection.states[s]}
                </button>
              )}
            </For>
          </div>
        </Show>
      </main>
    </div>
  );
};

// --- Small Segmented control ----------------------------------------------

type SegmentedProps = {
  value: string;
  options: { value: string; label: string }[];
  onChange: (v: string) => void;
  disabled?: boolean;
};

const Segmented: Component<SegmentedProps> = (props) => {
  return (
    <div class="lab__segmented" role="group">
      <For each={props.options}>
        {(opt) => (
          <button
            type="button"
            class="lr-focusable"
            aria-pressed={props.value === opt.value ? "true" : "false"}
            disabled={props.disabled}
            onClick={() => props.onChange(opt.value)}
          >
            {opt.label}
          </button>
        )}
      </For>
    </div>
  );
};

export default App;
