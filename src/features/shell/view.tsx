import type { ReactElement, ReactNode } from "react";
import {
  Button,
  MessageBar,
  MessageBarBody,
  MessageBarTitle,
  Text,
  makeStyles,
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
import { SettingsLayout, SettingsNavigation } from "../../ui/ueli";

// Layout structure follows Ueli's MIT-licensed Fluent UI settings renderer at
// commit f04ebdd82df71949d6b685ca7f2e5dd7e9b1bf90; LinguaRay state and all
// Tauri calls stay outside the view.
const useStyles = makeStyles({
  banner: {
    marginBottom: "20px",
  },
  bannerRow: {
    display: "flex",
    alignItems: "center",
    justifyContent: "space-between",
    flexWrap: "wrap",
    gap: "12px",
  },
  bannerCopy: {
    flex: "1 1 16rem",
  },
  bannerActions: {
    display: "flex",
    gap: "6px",
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

/**
 * Ueli's settings renderer with LinguaRay navigation/state injected through
 * props. The shell contains no Tauri or controller imports.
 */
export function SettingsShellView(props: SettingsShellViewProps) {
  const styles = useStyles();
  const t = SHELL_COPY[props.locale];
  const items = SETTINGS_SECTIONS.map((section) => ({
    value: section,
    label: t.nav[section],
    icon: SECTION_ICONS[section],
  }));

  return (
    <div
      style={{ height: "100vh", overflow: "hidden" }}
      data-testid="shell"
      data-page={props.active}
      data-layout="ueli"
    >
      <SettingsLayout
        navigation={
          <SettingsNavigation
            label={t.navLabel}
            active={props.active}
            items={items}
            onNavigate={props.onNavigate}
          />
        }
      >
        <main>
          {props.a11yGranted === false && (
            <MessageBar intent="warning" className={styles.banner} data-testid="a11y-banner">
              <MessageBarBody>
                <div className={styles.bannerRow}>
                  <div className={styles.bannerCopy}>
                    <MessageBarTitle>{t.a11y.title}</MessageBarTitle>
                    <Text size={300}>{t.a11y.hint}</Text>
                  </div>
                  <div className={styles.bannerActions}>
                    <Button appearance="subtle" size="small" onClick={props.onRecheckA11y} data-testid="a11y-recheck">
                      {t.a11y.recheck}
                    </Button>
                    <Button appearance="secondary" size="small" onClick={props.onOpenA11ySettings} data-testid="a11y-open-settings">
                      {t.a11y.openSettings}
                    </Button>
                  </div>
                </div>
              </MessageBarBody>
            </MessageBar>
          )}
          {props.children}
        </main>
      </SettingsLayout>
    </div>
  );
}

export default SettingsShellView;
