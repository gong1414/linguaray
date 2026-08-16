import { useEffect, useRef } from "react";
import {
  Badge,
  Button,
  Card,
  MessageBar,
  MessageBarBody,
  MessageBarTitle,
  ProgressBar,
  Spinner,
  Text,
  makeStyles,
  tokens,
} from "@fluentui/react-components";
import { SettingsRegular } from "@fluentui/react-icons";
import { ONBOARDING_COPY, type Locale } from "./copy";
import { STEP_ORDER, type OnboardingStepName, type PermissionState, type ShortcutCombo } from "./model";

export type OnboardingViewProps = {
  step: OnboardingStepName;
  locale: Locale;
  a11y: PermissionState;
  screenCapture: PermissionState;
  providerCount: number | null;
  historyBusy: boolean;
  shortcuts: ShortcutCombo[];
  advancing: boolean;
  error: string | null;
  onOpenA11ySettings: () => void;
  onOpenScreenCaptureSettings: () => void;
  onRecheckPermissions: () => void;
  onOpenProviderSettings: () => void;
  onOpenShortcutsSettings: () => void;
  onEnableHistory: () => void;
  onAdvance: (event: "start" | "continue" | "skip" | "complete") => void;
  onFinish: (openSettings: boolean) => void;
};

const useStyles = makeStyles({
  shell: {
    height: "100vh",
    display: "flex",
    flexDirection: "column",
    minHeight: 0,
    color: tokens.colorNeutralForeground1,
    backgroundColor: tokens.colorNeutralBackground1,
  },
  progress: {
    padding: `${tokens.spacingVerticalS} ${tokens.spacingHorizontalM} 0`,
    display: "flex",
    flexDirection: "column",
    gap: tokens.spacingVerticalXS,
  },
  content: {
    flex: "1 1 auto",
    minHeight: 0,
    overflowY: "auto",
    display: "flex",
    flexDirection: "column",
    gap: tokens.spacingVerticalS,
    padding: tokens.spacingHorizontalM,
  },
  title: { margin: 0 },
  desc: { whiteSpace: "pre-line", color: tokens.colorNeutralForeground2 },
  card: { width: "100%" },
  row: { display: "flex", alignItems: "center", justifyContent: "space-between", gap: tokens.spacingHorizontalM },
  stack: { display: "flex", flexDirection: "column", gap: tokens.spacingVerticalXS },
  end: { display: "flex", justifyContent: "flex-end", gap: tokens.spacingHorizontalS, flexWrap: "wrap" },
  footer: {
    flex: "0 0 auto",
    display: "flex",
    justifyContent: "flex-end",
    gap: tokens.spacingHorizontalS,
    padding: `${tokens.spacingVerticalS} ${tokens.spacingHorizontalM}`,
    borderTop: `${tokens.strokeWidthThin} solid ${tokens.colorNeutralStroke2}`,
  },
  monospace: { fontFamily: '"SF Mono", "Cascadia Code", Consolas, monospace' },
});

const BADGE_COLOR: Record<PermissionState, "subtle" | "success" | "warning" | "danger"> = {
  checking: "subtle",
  granted: "success",
  denied: "warning",
  error: "danger",
  unsupported: "subtle",
};

function StatusBadge({ state, labels }: { state: PermissionState; labels: Record<string, string> }) {
  return <Badge appearance="tint" color={BADGE_COLOR[state]} role="status">{labels[state] ?? state}</Badge>;
}

function stepTitle(t: (typeof ONBOARDING_COPY)[Locale], step: OnboardingStepName): string {
  if (step === "welcome") return t.welcome.title;
  if (step === "done") return t.done.title;
  if (step === "accessibility") return t.a11y.title;
  if (step === "provider") return t.provider.title;
  if (step === "history") return t.history.title;
  return t.shortcuts.title;
}

/** Pure presentational onboarding (props + callbacks only). */
export function OnboardingView(props: OnboardingViewProps) {
  const t = ONBOARDING_COPY[props.locale];
  const styles = useStyles();
  const titleRef = useRef<HTMLHeadingElement>(null);
  useEffect(() => titleRef.current?.focus(), [props.step]);
  const stepIndex = STEP_ORDER.indexOf(props.step);
  const providerHint = props.providerCount === null
    ? t.provider.checking
    : props.providerCount === 0 ? t.provider.noneBody : t.provider.count(props.providerCount);

  return (
    <main className={styles.shell} data-step={props.step} data-testid="onboarding">
      <div className={styles.progress}>
        <div className={styles.row}>
          <Text size={200} weight="semibold">{t.brand}</Text>
          <Text size={200}>{t.stepLabels[props.step]} · {stepIndex + 1}/{STEP_ORDER.length}</Text>
        </div>
        <ProgressBar value={(stepIndex + 1) / STEP_ORDER.length} aria-label={t.brand} />
      </div>

      <div className={styles.content}>
        <Text as="h1" size={600} weight="semibold" ref={titleRef} tabIndex={-1} className={styles.title} data-testid="onboarding-title">
          {stepTitle(t, props.step)}
        </Text>

        {props.step === "welcome" && <Text size={300} className={styles.desc}>{t.welcome.body}</Text>}

        {props.step === "accessibility" && (
          <>
            <Text size={300} className={styles.desc}>{t.a11y.body}</Text>
            <Card appearance="outline" size="small" className={styles.card}>
              <div className={styles.stack}>
                <div className={styles.row}><Text>{t.a11y.title}</Text><StatusBadge state={props.a11y} labels={t.a11y.status} /></div>
                <div className={styles.end}><Button appearance="secondary" size="small" icon={<SettingsRegular />} onClick={props.onOpenA11ySettings} disabled={props.a11y === "unsupported"}>{t.a11y.openSettings}</Button></div>
              </div>
            </Card>
            <Card appearance="outline" size="small" className={styles.card}>
              <div className={styles.stack}>
                <div className={styles.row}><Text>{t.a11y.screenTitle}</Text><StatusBadge state={props.screenCapture} labels={t.a11y.status} /></div>
                <Text size={200} className={styles.desc}>{t.a11y.screenBody}</Text>
                <div className={styles.end}><Button appearance="secondary" size="small" icon={<SettingsRegular />} onClick={props.onOpenScreenCaptureSettings} disabled={props.screenCapture === "unsupported"}>{t.a11y.openScreenSettings}</Button></div>
              </div>
            </Card>
            <Button appearance="subtle" size="small" onClick={props.onRecheckPermissions} disabled={props.advancing}>{t.a11y.recheck}</Button>
          </>
        )}

        {props.step === "provider" && (
          <>
            <Text size={300} className={styles.desc}>{t.provider.body}</Text>
            <Card appearance="outline" size="small" className={styles.card}>
              <div className={styles.stack}>
                <Text size={300}>{providerHint}</Text>
                <div className={styles.end}><Button appearance="secondary" size="small" icon={<SettingsRegular />} onClick={props.onOpenProviderSettings}>{t.provider.openSettings}</Button></div>
              </div>
            </Card>
          </>
        )}

        {props.step === "history" && <Text size={300} className={styles.desc}>{t.history.body}</Text>}

        {props.step === "shortcuts" && (
          <>
            <Text size={300} className={styles.desc}>{t.shortcuts.body}</Text>
            <Card appearance="outline" size="small" className={styles.card} data-testid="onboarding-shortcuts">
              <div className={styles.stack}>
                {props.shortcuts.map((shortcut) => (
                  <div key={shortcut.action} className={styles.row}>
                    <Text>{t.shortcuts.combos[shortcut.action] ?? shortcut.action}</Text>
                    <Badge appearance="tint"><kbd className={styles.monospace}>{shortcut.combo}</kbd></Badge>
                  </div>
                ))}
              </div>
            </Card>
            <Button appearance="subtle" size="small" icon={<SettingsRegular />} onClick={props.onOpenShortcutsSettings}>{t.shortcuts.openSettings}</Button>
          </>
        )}

        {props.step === "done" && <Text size={300} className={styles.desc}>{t.done.body}</Text>}

        {props.error && <MessageBar intent="error" role="alert" data-testid="onboarding-error"><MessageBarBody><MessageBarTitle>{t.errorPrefix}</MessageBarTitle>{props.error}</MessageBarBody></MessageBar>}
      </div>

      <div className={styles.footer}>
        {props.step === "welcome" && <Button appearance="primary" onClick={() => props.onAdvance("start")}>{t.welcome.start}</Button>}
        {props.step === "accessibility" && <><Button appearance="subtle" onClick={() => props.onAdvance("skip")} disabled={props.advancing}>{t.a11y.later}</Button><Button appearance="primary" onClick={() => props.onAdvance("continue")} disabled={props.advancing || props.a11y === "checking"}>{t.a11y.continue}</Button></>}
        {props.step === "provider" && <Button appearance="primary" onClick={() => props.onAdvance("continue")} disabled={props.advancing}>{t.provider.continue}</Button>}
        {props.step === "history" && <><Button appearance="subtle" onClick={() => props.onAdvance("skip")} disabled={props.advancing || props.historyBusy}>{t.history.skip}</Button><Button appearance="primary" icon={props.historyBusy ? <Spinner size="tiny" /> : undefined} onClick={props.onEnableHistory} disabled={props.advancing || props.historyBusy}>{props.historyBusy ? t.history.enabling : t.history.enable}</Button></>}
        {props.step === "shortcuts" && <Button appearance="primary" onClick={() => props.onAdvance("complete")} disabled={props.advancing}>{t.shortcuts.continue}</Button>}
        {props.step === "done" && <><Button appearance="subtle" onClick={() => props.onFinish(false)}>{t.done.tray}</Button><Button appearance="primary" onClick={() => props.onFinish(true)}>{t.done.openApp}</Button></>}
      </div>
    </main>
  );
}

export default OnboardingView;
