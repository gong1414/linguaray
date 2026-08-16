import { useEffect, useState, type ReactElement, type ReactNode } from "react";
import {
  Button,
  MessageBar,
  MessageBarBody,
  MessageBarTitle,
  NavDrawer,
  NavDrawerBody,
  NavItem,
  Text,
  makeStyles,
  tokens,
} from "@fluentui/react-components";
import {
  ArrowSyncRegular,
  BookLetterRegular,
  BookRegular,
  HistoryRegular,
  KeyboardRegular,
  KeyRegular,
  ServerRegular,
  ShieldRegular,
} from "@fluentui/react-icons";
import { SHELL_COPY } from "./copy";
import { SETTINGS_SECTIONS, type SettingsSection } from "./model";

// Layout structure follows Ueli's MIT-licensed Fluent UI settings renderer at
// commit f04ebdd82df7; LinguaRay state and all Tauri calls stay outside the view.
const useStyles = makeStyles({
  shell: {
    height: "100vh",
    display: "flex",
    overflow: "hidden",
    backgroundColor: tokens.colorNeutralBackground1,
    color: tokens.colorNeutralForeground1,
  },
  navigation: {
    height: "100vh",
    flexShrink: 0,
    overflow: "hidden",
    borderRight: `${tokens.strokeWidthThin} solid ${tokens.colorNeutralStroke2}`,
    transitionProperty: "width",
    transitionDuration: tokens.durationNormal,
  },
  drawer: {
    width: "100%",
    minWidth: "unset",
    height: "100%",
  },
  main: {
    minWidth: 0,
    flexGrow: 1,
    overflowY: "auto",
    padding: tokens.spacingHorizontalL,
  },
  banner: {
    marginBottom: tokens.spacingVerticalL,
  },
  bannerRow: {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    flexWrap: "wrap",
    gap: tokens.spacingHorizontalM,
  },
  bannerCopy: {
    flex: "1 1 16rem",
  },
  bannerActions: {
    display: "flex",
    gap: tokens.spacingHorizontalXS,
  },
});

const SECTION_ICONS: Record<SettingsSection, ReactElement> = {
  "provider-center": <ServerRegular aria-hidden />,
  "keystore-recovery": <KeyRegular aria-hidden />,
  shortcuts: <KeyboardRegular aria-hidden />,
  privacy: <ShieldRegular aria-hidden />,
  history: <HistoryRegular aria-hidden />,
  vocabulary: <BookLetterRegular aria-hidden />,
  dictionary: <BookRegular aria-hidden />,
  updater: <ArrowSyncRegular aria-hidden />,
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
 * Pure presentational settings shell. Fluent UI owns controls, states, focus,
 * theme and accessibility; this component only chooses the desktop layout.
 */
export function SettingsShellView(props: SettingsShellViewProps) {
  const styles = useStyles();
  const t = SHELL_COPY[props.locale];
  const [wide, setWide] = useState(
    typeof window !== "undefined" && window.matchMedia
      ? window.matchMedia(WIDE_QUERY).matches
      : true,
  );

  useEffect(() => {
    if (typeof window === "undefined" || !window.matchMedia) return;
    const mql = window.matchMedia(WIDE_QUERY);
    const onChange = (event: MediaQueryListEvent) => setWide(event.matches);
    mql.addEventListener("change", onChange);
    return () => mql.removeEventListener("change", onChange);
  }, []);

  const layout = wide ? "full" : "rail";

  return (
    <div
      className={styles.shell}
      data-testid="shell"
      data-page={props.active}
      data-layout={layout}
    >
      <div className={styles.navigation} style={{ width: wide ? 220 : 64 }}>
        <NavDrawer
          open
          type="inline"
          density="small"
          selectedValue={props.active}
          aria-label={t.navLabel}
          className={styles.drawer}
        >
          <NavDrawerBody>
            {SETTINGS_SECTIONS.map((section) => (
              <NavItem
                key={section}
                value={section}
                href="#"
                icon={SECTION_ICONS[section]}
                aria-label={t.nav[section]}
                onClick={(event) => {
                  event.preventDefault();
                  props.onNavigate(section);
                }}
              >
                {wide ? t.nav[section] : null}
              </NavItem>
            ))}
          </NavDrawerBody>
        </NavDrawer>
      </div>

      <main className={styles.main}>
        {props.a11yGranted === false && (
          <MessageBar intent="warning" className={styles.banner} data-testid="a11y-banner">
            <MessageBarBody>
              <div className={styles.bannerRow}>
                <div className={styles.bannerCopy}>
                  <MessageBarTitle>{t.a11y.title}</MessageBarTitle>
                  <Text size={300}>{t.a11y.hint}</Text>
                </div>
                <div className={styles.bannerActions}>
                  <Button
                    appearance="subtle"
                    size="small"
                    onClick={props.onRecheckA11y}
                    data-testid="a11y-recheck"
                  >
                    {t.a11y.recheck}
                  </Button>
                  <Button
                    appearance="secondary"
                    size="small"
                    onClick={props.onOpenA11ySettings}
                    data-testid="a11y-open-settings"
                  >
                    {t.a11y.openSettings}
                  </Button>
                </div>
              </div>
            </MessageBarBody>
          </MessageBar>
        )}
        {props.children}
      </main>
    </div>
  );
}

export default SettingsShellView;
