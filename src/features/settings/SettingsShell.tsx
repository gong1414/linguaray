import {
  createSignal,
  onCleanup,
  onMount,
  Show,
  type Component,
  type JSX,
} from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { Server, ShieldCheck, Keyboard, ShieldAlert, History, BookOpen, BookMarked, RefreshCw } from "lucide-solid";
import { SidebarItem, Tooltip } from "@linguaray/ui";
import { SETTINGS_COPY } from "./copy";
import { detectLocale } from "../../i18n";
import "./SettingsShell.css";

export type SettingsSection =
  | "provider-center"
  | "keystore-recovery"
  | "shortcuts"
  | "privacy"
  | "history"
  | "vocabulary"
  | "dictionary"
  | "updater";

export type SettingsShellProps = {
  /** Initial active section (default: "provider-center"). */
  initialSection?: SettingsSection;
  /** Controlled active section. When supplied, the parent owns the active
   *  state; the shell reads `props.activePage` instead of its own signal. */
  activePage?: SettingsSection;
  /** Called when the user clicks an enabled nav item. */
  onNavigate?: (section: SettingsSection) => void;
  /** Content for the currently-active section. */
  children: JSX.Element;
};

/** Wide breakpoint: >=700px shows full labels; 600-699px collapses to a rail. */
const WIDE_QUERY = "(min-width: 700px)";

type NavDef = {
  id: SettingsSection;
  label: string;
  icon: JSX.Element;
  disabled: boolean;
};

/**
 * Settings shell: sidebar nav + content slot inside the OS-native title bar
 * (the window keeps system decorations; custom chrome on a decorated window
 * produced a double title bar). The sidebar is full-label at >=700px
 * (`data-layout="full"`) and collapses to an icon rail at 600-699px
 * (`data-layout="rail"`), driven by matchMedia. Rail items are wrapped in a
 * Tooltip so their label remains reachable by hover or keyboard focus.
 */
const SettingsShell: Component<SettingsShellProps> = (props) => {
  const locale = detectLocale();
  const t = SETTINGS_COPY[locale];
  // rev-9-2: controlled-uncontrolled dual mode. `active` is a DERIVATION of
  // props.activePage (when the parent supplies it, the parent owns the state)
  // falling back to the internal signal (uncontrolled mode). A plain
  // createSignal(props.activePage ?? ...) initializer would read props.activePage
  // ONCE at first render and then go stale when the parent passes a new value.
  const [internalActive, setInternalActive] = createSignal<SettingsSection>(
    props.initialSection ?? "provider-center",
  );
  const active = (): SettingsSection => props.activePage ?? internalActive();
  const [wide, setWide] = createSignal(
    typeof window !== "undefined" && window.matchMedia
      ? window.matchMedia(WIDE_QUERY).matches
      : true,
  );

  // Subscribe to the breakpoint so window resize reflows the rail.
  let mql: MediaQueryList | undefined;
  if (typeof window !== "undefined" && window.matchMedia) {
    mql = window.matchMedia(WIDE_QUERY);
    const onChange = (e: MediaQueryListEvent) => setWide(e.matches);
    mql.addEventListener?.("change", onChange);
    onCleanup(() => mql?.removeEventListener?.("change", onChange));
  }

  const navItems: NavDef[] = [
    { id: "provider-center", label: t.nav.providerCenter, icon: <Server size={16} />, disabled: false },
    { id: "keystore-recovery", label: t.nav.keystoreRecovery, icon: <ShieldCheck size={16} />, disabled: false },
    { id: "shortcuts", label: t.nav.shortcuts, icon: <Keyboard size={16} />, disabled: false },
    { id: "privacy", label: t.nav.privacy, icon: <ShieldAlert size={16} />, disabled: false },
    { id: "history", label: t.nav.history, icon: <History size={16} />, disabled: false },
    { id: "vocabulary", label: t.nav.vocabulary, icon: <BookMarked size={16} />, disabled: false },
    { id: "dictionary", label: t.nav.dictionary, icon: <BookOpen size={16} />, disabled: false },
    { id: "updater", label: t.nav.updater, icon: <RefreshCw size={16} />, disabled: false },
  ];

  // macOS Accessibility permission. null = unknown (pre-first-resolve), true =
  // granted, false = not granted → render the recovery banner. Selection
  // capture needs the AX permission for both the direct-read and the simulated
  // Cmd+C fallback, so a missing grant is surfaced here, not silently ignored.
  const [a11yGranted, setA11yGranted] = createSignal<boolean | null>(null);
  const recheckA11y = async () => {
    try {
      const granted = await invoke<boolean>("a11y_status");
      setA11yGranted(granted);
    } catch {
      // Swallow: a non-Tauri context (or a backend that lacks the command)
      // leaves the banner hidden rather than blocking the whole shell.
      setA11yGranted(true);
    }
  };
  onMount(() => {
    void recheckA11y();
    // Re-check when the window regains focus: the user likely just toggled the
    // grant in System Settings, so refresh the banner without a manual Re-check.
    // Lazy-import the Tauri window API so jsdom tests that don't mock it (and
    // non-Tauri contexts) never touch the bridge at import time.
    let unlisten: (() => void) | undefined;
    // Race guard: if the component unmounts BEFORE the dynamic import +
    // onFocusChanged() promise resolves, `unlisten` is still undefined and
    // onCleanup would be a no-op, leaking the listener (whose closure captures
    // this component's recheckA11y). `cancelled` lets the resolve path tear
    // down a listener that arrived after teardown.
    let cancelled = false;
    import("@tauri-apps/api/window")
      .then(({ getCurrentWindow }) =>
        getCurrentWindow().onFocusChanged(({ payload: focused }) => {
          if (focused) void recheckA11y();
        }),
      )
      .then((u) => {
        if (cancelled) u();
        else unlisten = u;
      })
      .catch(() => {});
    onCleanup(() => {
      cancelled = true;
      unlisten?.();
    });
  });

  const handleClick = (id: SettingsSection) => {
    // rev-9-2: only mutate the internal signal in UNCONTROLLED mode
    // (props.activePage === undefined). In controlled mode the parent is the
    // source of truth and updates `activePage` via the onNavigate callback.
    if (props.activePage === undefined) {
      setInternalActive(id);
    }
    props.onNavigate?.(id);
  };

  const renderItem = (item: NavDef) => {
    const ariaLabel = item.disabled
      ? `${item.label} — ${t.nav.placeholderHint}`
      : item.label;
    const node = (
      <SidebarItem
        label={item.label}
        ariaLabel={ariaLabel}
        icon={item.icon}
        active={active() === item.id}
        disabled={item.disabled}
        onClick={() => handleClick(item.id)}
      />
    );
    // In rail mode every item is wrapped so the hidden label remains
    // discoverable by pointer and keyboard users.
    const needsTooltip = item.disabled || !wide();
    const content = item.disabled ? t.nav.placeholderHint : item.label;
    return (
      <Show when={needsTooltip} fallback={node}>
        <Tooltip content={content} side="right">
          {node}
        </Tooltip>
      </Show>
    );
  };

  return (
    <div
      class="settings-shell"
      data-layout={wide() ? "full" : "rail"}
      data-testid="shell"
      data-page={active()}
    >
      <nav class="settings-shell__nav" aria-label={t.window.title}>
        {navItems.map(renderItem)}
      </nav>
      <div class="settings-shell__content">
          <Show when={a11yGranted() === false}>
            <div
              class="settings-shell__a11y-banner"
              role="alert"
              data-testid="a11y-banner"
            >
              <div class="settings-shell__a11y-body">
                <strong class="settings-shell__a11y-title">{t.a11y.title}</strong>
                <p class="settings-shell__a11y-hint">{t.a11y.hint}</p>
              </div>
              <div class="settings-shell__a11y-actions">
                <button
                  type="button"
                  class="settings-shell__a11y-action"
                  data-testid="a11y-recheck"
                  onClick={() => void recheckA11y()}
                >
                  {t.a11y.recheck}
                </button>
                <button
                  type="button"
                  class="settings-shell__a11y-action"
                  data-testid="a11y-open-settings"
                  onClick={() =>
                    // Lazy-import so a non-Tauri context (or a test that doesn't
                    // mock the plugin) never touches the opener bridge.
                    import("@tauri-apps/plugin-opener")
                      .then(({ openUrl }) =>
                        openUrl(
                          "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
                        ),
                      )
                      .catch(() => {})
                  }
                >
                  {t.a11y.openSettings}
                </button>
              </div>
            </div>
          </Show>
          {props.children}
        </div>
    </div>
  );
};

export default SettingsShell;
