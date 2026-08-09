import {
  For,
  Show,
  Switch,
  Match,
  createSignal,
  createMemo,
  createEffect,
  type Component,
} from "solid-js";
import { strings, galleryStrings, type Locale, type SelectionState, type ProviderState } from "./i18n";
import SelectionPopup from "./pages/SelectionPopup";
import ProviderCenter from "./pages/ProviderCenter";
import InputPanel from "./pages/InputPanel";
import KeystoreRecovery from "./pages/KeystoreRecovery";
import { ComponentGallery } from "./pages/ComponentGallery";
import { SidebarItem, Confirm } from "@linguaray/ui";
import { Settings } from "lucide-solid";
import "./App.css";

type Theme = "light" | "dark";
type Motion = "full" | "reduced";
type WindowSize = {
  w: number;
  h: number;
  key: "single" | "expanded" | "compact";
  label: string;
};

type NavKey =
  | "selection-popup"
  | "input-window"
  | "provider-center"
  | "ocr-overlay"
  | "history"
  | "tray-menubar"
  | "onboarding"
  | "multi-result"
  | "shortcuts"
  | "privacy"
  | "keystore"
  | "vocabulary"
  | "dictionary"
  | "tts"
  | "external-api"
  | "updater"
  | "component-gallery"
  | "sidebar-isolated"
  | "confirm-isolated";

// Complete S0 §4.1 state matrix.
const SELECTION_STATES: SelectionState[] = [
  "initial-hidden",
  "loading",
  "success-single",
  "success-dual",
  "success-multi",
  "partial",
  "error-network",
  "error-config-key",
  "error-config-401",
  "error-no-selection",
  "error-no-provider",
  "error-no-permission",
  "keystore-corrupt",
  "offline-fallback",
  "offline-error",
  "pinned",
];

// All S0 §4.1–§4.16 surfaces (16 total, including §4.6 Tray/Menu-bar).
const NAV_ITEMS: {
  key: NavKey;
  labelKey: keyof (typeof strings)["en"]["nav"];
}[] = [
  { key: "selection-popup", labelKey: "selectionPopup" },
  { key: "input-window", labelKey: "inputWindow" },
  { key: "provider-center", labelKey: "providerCenter" },
  { key: "ocr-overlay", labelKey: "ocrOverlay" },
  { key: "history", labelKey: "history" },
  { key: "tray-menubar", labelKey: "trayMenubar" },
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
  { key: "component-gallery", labelKey: "componentGallery" },
  { key: "sidebar-isolated", labelKey: "sidebarIsolated" },
  { key: "confirm-isolated", labelKey: "confirmIsolated" },
];

// Implemented surfaces.
const IMPLEMENTED: NavKey[] = ["selection-popup", "input-window", "provider-center", "keystore", "component-gallery", "sidebar-isolated", "confirm-isolated"];

const PROVIDER_STATES: ProviderState[] = [
  "empty",
  "loading-models",
  "model-fetch-error",
  "model-manual-entry",
  "connection-testing",
  "connection-ok",
  "connection-failed",
  "key-saved",
  "key-missing",
  "duplicate",
  "saving",
  "save-failed",
  "save-conflict",
  "delete-confirm",
  "deleting",
  "delete-retry",
  "drag-reorder",
  "reorder-failed",
  "balance-loading",
  "balance-unsupported",
  "balance-rate-limited",
  "balance-error",
  "endpoint-invalid",
];

const App: Component = () => {
  // Read URL query params so Playwright e2e can deep-link a surface/theme
  // (e.g. ?nav=component-gallery&theme=dark). Falls back to defaults.
  const params = new URLSearchParams(
    typeof window !== "undefined" ? window.location.search : "",
  );
  const validNav = (NAV_ITEMS.map((i) => i.key) as NavKey[]);
  const initialNav = validNav.includes(params.get("nav") as NavKey)
    ? (params.get("nav") as NavKey)
    : "selection-popup";
  const initialTheme =
    params.get("theme") === "dark" || params.get("theme") === "light"
      ? (params.get("theme") as Theme)
      : "light";

  const [locale, setLocale] = createSignal<Locale>("en");
  const [theme, setTheme] = createSignal<Theme>(initialTheme);
  const [motion, setMotion] = createSignal<Motion>("full");
  const [nav, setNav] = createSignal<NavKey>(initialNav);
  // C1: seed selState from ?state= so the surfaces visual spec can deep-link
  // each popup state (loading/success-single/success-multi/partial/error-network).
  // Previously hardcoded to "success-single", which made 4 of 5 popup baselines
  // duplicates of the single-success shot. Mirrors the initialNav param-read.
  const initialSel = SELECTION_STATES.includes(params.get("state") as SelectionState)
    ? (params.get("state") as SelectionState)
    : "success-single";
  const [selState, setSelState] = createSignal<SelectionState>(initialSel);
  const [provState, setProvState] = createSignal<ProviderState>(
    // rev-6-8: ProviderState has NO "configured" value. The populated variant
    // seeds "key-saved" (a real ProviderState); the empty variant seeds "empty".
    params.get("fixture") === "populated" ? "key-saved" : "empty",
  );
  const [keystoreState, setKeystoreState] = createSignal<"healthy" | "corrupt">(
    params.get("fixture") === "corrupt" ? "corrupt" : "healthy",
  );
  const [inputState, setInputState] = createSignal<"idle" | "multi" | "partial" | "error">(
    params.get("state") === "multi"
      ? "multi"
      : params.get("state") === "partial"
        ? "partial"
        : params.get("state") === "error"
          ? "error"
          : "idle",
  );
  const [settingsSize, setSettingsSize] = createSignal<"min" | "default" | "narrow-699" | "boundary-700">("default");

  const t = createMemo(() => strings[locale()]);
  const selT = createMemo(() => t().selection);
  const provT = createMemo(() => t().provider);

  // Window size follows MASTER §8.2 + S0 §4.1:
  //  - multi-engine states → expanded 600×400
  //  - loading → the native popup window itself is a compact card (~200×40)
  //    at the cursor, NOT a 400×300 frame with a small body inside
  //  - initial-hidden → no popup window exists at all (rendered elsewhere)
  //  - everything else → single 400×300
  const isMultiState = () =>
    selState() === "success-dual" ||
    selState() === "success-multi" ||
    selState() === "partial";
  const isHidden = () => selState() === "initial-hidden";
  const isLoadingCompact = () => selState() === "loading";
  const windowSize = createMemo<WindowSize>(() => {
    if (isMultiState())
      return { w: 600, h: 400, key: "expanded", label: "600×400" };
    if (isLoadingCompact())
      return { w: 200, h: 40, key: "compact", label: "200×40" };
    return { w: 400, h: 300, key: "single", label: "400×300" };
  });

  // Sync theme + motion onto <html> so token + base rules apply globally.
  createEffect(() => {
    const html = document.documentElement;
    html.setAttribute("data-theme", theme());
    html.setAttribute("data-motion", motion());
    html.setAttribute("lang", locale());
  });

  // sidebar-isolated: bare fixture for Playwright keyboard e2e — renders ONLY
  // a single SidebarItem with no header/nav/controls, so the first Tab always
  // lands on it. Bypasses the full lab shell entirely.
  if (nav() === "sidebar-isolated") {
    return (
      <div class="gallery__iso">
        <SidebarItem
          label={t().nav.componentGallery}
          icon={<Settings size={16} />}
          active
          onClick={() => {}}
        />
      </div>
    );
  }

  // confirm-isolated: bare fixture for Playwright visual baseline — renders
  // ONLY an open Confirm dialog (open=true) with no header/nav/controls, so
  // the screenshot captures the dialog body instead of just its trigger
  // button. Reuses the gallery's confirm copy for consistency.
  if (nav() === "confirm-isolated") {
    const g = galleryStrings[locale()].confirm;
    return (
      <div class="gallery__iso">
        <Confirm
          open={true}
          onOpenChange={() => {}}
          title={g.deleteTitle}
          message={g.deleteMsg}
          confirmLabel={g.confirm}
          cancelLabel={g.cancel}
          variant="destructive"
          onConfirm={() => {}}
          onCancel={() => {}}
        />
      </div>
    );
  }

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
              ariaLabel={t().controls.localeGroup}
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
              ariaLabel={t().controls.themeGroup}
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
              ariaLabel={t().controls.motionGroup}
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
              ariaLabel={t().controls.windowSizeGroup}
              value={windowSize().key}
              options={[
                { value: "single", label: t().controls.size400x300 },
                { value: "expanded", label: t().controls.size600x400 },
              ]}
              onChange={(v) => {
                if (v === "expanded") setSelState("success-dual");
                else setSelState("success-single");
              }}
            />
          </div>
        </div>
      </header>

      <nav class="lab__nav" aria-label={t().navGroupLabel}>
        <ul class="lab__nav-list">
          <For each={NAV_ITEMS}>
            {(item) => {
              const implemented = IMPLEMENTED.includes(item.key);
              return (
                <li>
                  <button
                    class="lr-focusable"
                    aria-current={nav() === item.key ? "page" : undefined}
                    disabled={!implemented}
                    onClick={() => setNav(item.key)}
                  >
                    {t().nav[item.labelKey]}
                    <Show when={!implemented}>
                      <span class="lr-visually-hidden">
                        {" "}
                        ({t().notImplemented})
                      </span>
                    </Show>
                  </button>
                </li>
              );
            }}
          </For>
        </ul>
      </nav>

      <main class="lab__main">
        <div class="lab__stage">
          <Switch>
            <Match when={nav() === "selection-popup"}>
              {/* initial-hidden: the popup does not exist. Render NO frame. */}
              <Show
                when={!isHidden()}
                fallback={<p class="lab__hidden-note">{selT().initialHidden}</p>}
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
            </Match>

            <Match when={nav() === "provider-center"}>
              <div
                class="lab__frame lab__frame--settings"
                style={{
                  // Boundary-probe modes set the frame width so the
                  // settings-shell's OWN clientWidth straddles the 699/700
                  // container-query boundary. The shell is ~2px narrower than
                  // the frame (its own border), so:
                  //   frame 699 → shell ≈697 (≤699 → icon-only rail)
                  //   frame 702 → shell ≈700 (≥700 → full labels)
                  // This is the only way to exercise the transition in a layout.
                  width:
                    settingsSize() === "min" ? "600px"
                    : settingsSize() === "narrow-699" ? "699px"
                    : settingsSize() === "boundary-700" ? "702px"
                    : "800px",
                  height: settingsSize() === "min" ? "400px" : "600px",
                }}
              >
                <ProviderCenter
                  state={provState()}
                  locale={locale()}
                  t={provT()}
                />
              </div>
              <span class="lab__frame-meta">
                {settingsSize() === "min" ? "600×400"
                  : settingsSize() === "narrow-699" ? "shell≈697 (icon-only)"
                  : settingsSize() === "boundary-700" ? "shell≈700 (labels)"
                  : "800×600"} ·{" "}
                {t().provider.states[provState()]}
              </span>
            </Match>

            <Match when={nav() === "component-gallery"}>
              <ComponentGallery locale={locale()} theme={theme()} />
            </Match>

            <Match when={nav() === "input-window"}>
              <InputPanel state={inputState()} />
            </Match>

            <Match when={nav() === "keystore"}>
              <KeystoreRecovery state={keystoreState()} />
            </Match>

            <Match when={!IMPLEMENTED.includes(nav())}>
              <div class="lab__frame-placeholder">
                <p>
                  {t().nav[NAV_ITEMS.find((i) => i.key === nav())!.labelKey]}
                </p>
                <p class="lab__frame-placeholder-sub">{t().upcomingSlice}</p>
              </div>
            </Match>
          </Switch>
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

        <Show when={nav() === "provider-center"}>
          <div class="lab__state-bar" role="group" aria-label={t().controls.state}>
            <span class="lab__state-label">{t().controls.state}</span>
            <For each={PROVIDER_STATES}>
              {(s) => (
                <button
                  type="button"
                  class="lab__state-chip lr-focusable"
                  aria-pressed={provState() === s ? "true" : "false"}
                  onClick={() => setProvState(s)}
                >
                  {t().provider.states[s]}
                </button>
              )}
            </For>
            <button
              type="button"
              class="lab__state-chip lr-focusable"
              onClick={() => setSettingsSize((v) =>
                v === "default" ? "min"
                : v === "min" ? "narrow-699"
                : v === "narrow-699" ? "boundary-700"
                : "default",
              )}
            >
              {settingsSize() === "min" ? t().provider.frameMin
                : settingsSize() === "narrow-699" ? t().provider.frameNarrow699
                : settingsSize() === "boundary-700" ? t().provider.frameBoundary700
                : t().provider.frameDefault}
            </button>
          </div>
        </Show>

        <Show when={nav() === "input-window"}>
          <div class="lab__state-bar" role="group" aria-label={t().controls.state}>
            <span class="lab__state-label">{t().controls.state}</span>
            <For each={["idle", "multi", "partial", "error"] as const}>
              {(s) => (
                <button
                  type="button"
                  class="lab__state-chip lr-focusable"
                  aria-pressed={inputState() === s ? "true" : "false"}
                  onClick={() => setInputState(s)}
                >
                  {s}
                </button>
              )}
            </For>
          </div>
        </Show>

        <Show when={nav() === "keystore"}>
          <div class="lab__state-bar" role="group" aria-label={t().controls.state}>
            <span class="lab__state-label">{t().controls.state}</span>
            <For each={["healthy", "corrupt"] as const}>
              {(s) => (
                <button
                  type="button"
                  class="lab__state-chip lr-focusable"
                  aria-pressed={keystoreState() === s ? "true" : "false"}
                  onClick={() => setKeystoreState(s)}
                >
                  {s}
                </button>
              )}
            </For>
          </div>
        </Show>
      </main>
    </div>
  );
};

// --- Segmented control ----------------------------------------------------

type SegmentedProps = {
  value: string;
  options: { value: string; label: string }[];
  onChange: (v: string) => void;
  /** Required accessible name for the group. */
  ariaLabel: string;
};

const Segmented: Component<SegmentedProps> = (props) => {
  return (
    <div class="lab__segmented" role="group" aria-label={props.ariaLabel}>
      <For each={props.options}>
        {(opt) => (
          <button
            type="button"
            class="lr-focusable"
            aria-pressed={props.value === opt.value ? "true" : "false"}
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
