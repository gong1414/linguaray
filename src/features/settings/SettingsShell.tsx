import {
  createSignal,
  onCleanup,
  Show,
  type Component,
  type JSX,
} from "solid-js";
import { Server, ShieldCheck, Keyboard, ShieldAlert } from "lucide-solid";
import { WindowChrome, SidebarItem, Tooltip } from "@linguaray/ui";
import { SETTINGS_COPY } from "./copy";
import { detectLocale } from "../../i18n";
import "./SettingsShell.css";

export type SettingsSection =
  | "provider-center"
  | "keystore-recovery"
  | "shortcuts"
  | "privacy";

export type SettingsShellProps = {
  /** Initial active section (default: "provider-center"). */
  initialSection?: SettingsSection;
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
 * Settings shell: WindowChrome frame + a SidebarItem nav. The sidebar is
 * full-label at >=700px (`data-layout="full"`) and collapses to an icon rail
 * at 600-699px (`data-layout="rail"`), driven by matchMedia. Shortcuts and
 * Privacy are disabled placeholders surfaced in R3b; each disabled item plus
 * every rail item is wrapped in a Tooltip so the label is reachable via hover
 * or keyboard focus.
 */
const SettingsShell: Component<SettingsShellProps> = (props) => {
  const locale = detectLocale();
  const t = SETTINGS_COPY[locale];
  const [active, setActive] = createSignal<SettingsSection>(
    props.initialSection ?? "provider-center",
  );
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
    { id: "shortcuts", label: t.nav.shortcuts, icon: <Keyboard size={16} />, disabled: true },
    { id: "privacy", label: t.nav.privacy, icon: <ShieldAlert size={16} />, disabled: true },
  ];

  const handleClick = (id: SettingsSection) => {
    setActive(id);
    props.onNavigate?.(id);
  };

  const renderItem = (item: NavDef) => {
    const node = (
      <SidebarItem
        label={item.label}
        icon={item.icon}
        active={active() === item.id}
        disabled={item.disabled}
        onClick={() => handleClick(item.id)}
      />
    );
    // Disabled placeholders always show a "Coming in R3b" hint; in rail mode
    // every item is wrapped so the hidden label is still discoverable.
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

  // Window controls: lazily import the Tauri window API so jsdom tests do not
  // require the bridge. Swallow any throw in a non-Tauri context.
  const handleClose = () => {
    import("@tauri-apps/api/window")
      .then(({ getCurrentWindow }) => getCurrentWindow().close())
      .catch(() => {});
  };
  const handleMinimize = () => {
    import("@tauri-apps/api/window")
      .then(({ getCurrentWindow }) => getCurrentWindow().minimize())
      .catch(() => {});
  };

  return (
    <div class="settings-shell" data-layout={wide() ? "full" : "rail"}>
      <WindowChrome
        title={t.window.title}
        labels={{ minimize: t.window.minimize, close: t.window.close }}
        onClose={handleClose}
        onMinimize={handleMinimize}
        sidebar={<nav class="settings-shell__nav" aria-label={t.window.title}>{navItems.map(renderItem)}</nav>}
      >
        <div class="settings-shell__content">{props.children}</div>
      </WindowChrome>
    </div>
  );
};

export default SettingsShell;
