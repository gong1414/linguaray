import { useEffect, useState, type ReactNode } from "react";
import { Alert, AppShell, Group, NavLink, Text } from "@mantine/core";
import {
  BookOpen,
  BookMarked,
  History as HistoryIcon,
  Keyboard,
  RefreshCw,
  Server,
  ShieldAlert,
  ShieldCheck,
} from "lucide-react";
import { SHELL_COPY } from "./copy";
import { SETTINGS_SECTIONS, type SettingsSection } from "./model";
import classes from "./shell.module.css";

const SECTION_ICONS: Record<SettingsSection, ReactNode> = {
  "provider-center": <Server size={16} aria-hidden />,
  "keystore-recovery": <ShieldCheck size={16} aria-hidden />,
  shortcuts: <Keyboard size={16} aria-hidden />,
  privacy: <ShieldAlert size={16} aria-hidden />,
  history: <HistoryIcon size={16} aria-hidden />,
  vocabulary: <BookMarked size={16} aria-hidden />,
  dictionary: <BookOpen size={16} aria-hidden />,
  updater: <RefreshCw size={16} aria-hidden />,
};

export type SettingsShellViewProps = {
  locale: "zh" | "en";
  active: SettingsSection;
  /** null = still checking / banner hidden. */
  a11yGranted: boolean | null;
  children: ReactNode;
  onNavigate: (section: SettingsSection) => void;
  onRecheckA11y: () => void;
  onOpenA11ySettings: () => void;
};

/** Wide breakpoint: >=700px shows full labels; 600-699px collapses to a rail. */
const WIDE_QUERY = "(min-width: 700px)";

/**
 * Pure presentational settings shell: OS-native title bar + AppShell navbar
 * nav (never a custom window chrome — docs/UI-RULES.md rule 2). The sidebar
 * is full-label at >=700px and an icon rail below it, driven by matchMedia;
 * rail items keep their accessible name via NavLink aria-label.
 */
export function SettingsShellView(props: SettingsShellViewProps) {
  const t = SHELL_COPY[props.locale];
  const [wide, setWide] = useState(
    typeof window !== "undefined" && window.matchMedia
      ? window.matchMedia(WIDE_QUERY).matches
      : true,
  );

  useEffect(() => {
    if (typeof window === "undefined" || !window.matchMedia) return;
    const mql = window.matchMedia(WIDE_QUERY);
    const onChange = (e: MediaQueryListEvent) => setWide(e.matches);
    mql.addEventListener("change", onChange);
    return () => mql.removeEventListener("change", onChange);
  }, []);

  const layout = wide ? "full" : "rail";

  return (
    <AppShell
      header={{ height: 0 }}
      navbar={{
        width: 220,
        breakpoint: 0,
        collapsed: { mobile: false, desktop: false },
      } as never}
      padding="md"
      data-testid="shell"
      data-page={props.active}
      data-layout={layout}
      className={classes.shell}
    >
      <AppShell.Navbar aria-label={t.navLabel} className={classes.nav}>
        {SETTINGS_SECTIONS.map((section) => (
          <NavLink
            key={section}
            href="#"
            active={props.active === section}
            label={wide ? t.nav[section] : undefined}
            aria-label={t.nav[section]}
            leftSection={SECTION_ICONS[section]}
            onClick={(e) => {
              e.preventDefault();
              props.onNavigate(section);
            }}
            className={classes.item}
          />
        ))}
      </AppShell.Navbar>
      <AppShell.Main className={classes.main}>
        {props.a11yGranted === false && (
          <Alert
            color="warning"
            title={t.a11y.title}
            mb="md"
            data-testid="a11y-banner"
            className={classes.a11y}
          >
            <Group justify="space-between" wrap="wrap" gap="sm">
              <Text size="sm" style={{ flex: "1 1 16rem" }}>
                {t.a11y.hint}
              </Text>
              <Group gap="xs">
                <a
                  href="#"
                  className="lr-link-styled"
                  onClick={(e) => {
                    e.preventDefault();
                    props.onRecheckA11y();
                  }}
                  style={{ fontSize: "var(--mantine-font-size-sm)" }}
                  data-testid="a11y-recheck"
                >
                  {t.a11y.recheck}
                </a>
                <a
                  href="#"
                  className="lr-link-styled"
                  onClick={(e) => {
                    e.preventDefault();
                    props.onOpenA11ySettings();
                  }}
                  style={{ fontSize: "var(--mantine-font-size-sm)" }}
                  data-testid="a11y-open-settings"
                >
                  {t.a11y.openSettings}
                </a>
              </Group>
            </Group>
          </Alert>
        )}
        {props.children}
      </AppShell.Main>
    </AppShell>
  );
}

export default SettingsShellView;
