import { useEffect, useRef } from "react";
import { Alert, Button, Card, Progress, Spin, Tag, Typography } from "antd";
import { SettingOutlined } from "@ant-design/icons";
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

const BADGE_COLOR: Record<PermissionState, string> = {
  checking: "default",
  granted: "success",
  denied: "warning",
  error: "error",
  unsupported: "default",
};

function StatusBadge({ state, labels }: { state: PermissionState; labels: Record<string, string> }) {
  return <Tag color={BADGE_COLOR[state]} role="status">{labels[state] ?? state}</Tag>;
}

function stepTitle(t: (typeof ONBOARDING_COPY)[Locale], step: OnboardingStepName): string {
  if (step === "welcome") return t.welcome.title;
  if (step === "done") return t.done.title;
  if (step === "accessibility") return t.a11y.title;
  if (step === "provider") return t.provider.title;
  if (step === "history") return t.history.title;
  return t.shortcuts.title;
}

/** Pure presentational Ant Design onboarding (props + callbacks only). */
export function OnboardingView(props: OnboardingViewProps) {
  const t = ONBOARDING_COPY[props.locale];
  const titleRef = useRef<HTMLHeadingElement>(null);
  useEffect(() => titleRef.current?.focus(), [props.step]);
  const stepIndex = STEP_ORDER.indexOf(props.step);
  const providerHint = props.providerCount === null ? t.provider.checking : props.providerCount === 0 ? t.provider.noneBody : t.provider.count(props.providerCount);

  return (
    <main className="lr-onboarding" data-step={props.step} data-testid="onboarding">
      <div className="lr-onboarding-progress">
        <div className="lr-row-between"><Typography.Text strong>{t.brand}</Typography.Text><Typography.Text type="secondary">{t.stepLabels[props.step]} · {stepIndex + 1}/{STEP_ORDER.length}</Typography.Text></div>
        <Progress percent={Math.round(((stepIndex + 1) / STEP_ORDER.length) * 100)} showInfo={false} aria-label={t.brand} />
      </div>
      <div className="lr-onboarding-content">
        <Typography.Title level={1} ref={titleRef} tabIndex={-1} className="lr-title" data-testid="onboarding-title">{stepTitle(t, props.step)}</Typography.Title>
        {props.step === "welcome" ? <Typography.Paragraph type="secondary">{t.welcome.body}</Typography.Paragraph> : null}
        {props.step === "accessibility" ? (
          <>
            <Typography.Paragraph type="secondary">{t.a11y.body}</Typography.Paragraph>
            <Card size="small"><div className="lr-stack-tight"><div className="lr-row-between"><Typography.Text>{t.a11y.title}</Typography.Text><StatusBadge state={props.a11y} labels={t.a11y.status} /></div><div className="lr-end"><Button icon={<SettingOutlined aria-hidden />} onClick={props.onOpenA11ySettings} disabled={props.a11y === "unsupported"}>{t.a11y.openSettings}</Button></div></div></Card>
            <Card size="small"><div className="lr-stack-tight"><div className="lr-row-between"><Typography.Text>{t.a11y.screenTitle}</Typography.Text><StatusBadge state={props.screenCapture} labels={t.a11y.status} /></div><Typography.Text type="secondary">{t.a11y.screenBody}</Typography.Text><div className="lr-end"><Button icon={<SettingOutlined aria-hidden />} onClick={props.onOpenScreenCaptureSettings} disabled={props.screenCapture === "unsupported"}>{t.a11y.openScreenSettings}</Button></div></div></Card>
            <Button type="text" onClick={props.onRecheckPermissions} disabled={props.advancing}>{t.a11y.recheck}</Button>
          </>
        ) : null}
        {props.step === "provider" ? (
          <>
            <Typography.Paragraph type="secondary">{t.provider.body}</Typography.Paragraph>
            <Card size="small"><div className="lr-stack-tight"><Typography.Text>{providerHint}</Typography.Text><div className="lr-end"><Button icon={<SettingOutlined aria-hidden />} onClick={props.onOpenProviderSettings}>{t.provider.openSettings}</Button></div></div></Card>
          </>
        ) : null}
        {props.step === "history" ? <Typography.Paragraph type="secondary">{t.history.body}</Typography.Paragraph> : null}
        {props.step === "shortcuts" ? (
          <>
            <Typography.Paragraph type="secondary">{t.shortcuts.body}</Typography.Paragraph>
            <Card size="small" data-testid="onboarding-shortcuts"><div className="lr-stack-tight">{props.shortcuts.map((shortcut) => <div key={shortcut.action} className="lr-row-between"><Typography.Text>{t.shortcuts.combos[shortcut.action] ?? shortcut.action}</Typography.Text><Tag><kbd className="lr-monospace">{shortcut.combo}</kbd></Tag></div>)}</div></Card>
            <Button type="text" icon={<SettingOutlined aria-hidden />} onClick={props.onOpenShortcutsSettings}>{t.shortcuts.openSettings}</Button>
          </>
        ) : null}
        {props.step === "done" ? <Typography.Paragraph type="secondary">{t.done.body}</Typography.Paragraph> : null}
        {props.error ? <Alert type="error" showIcon title={t.errorPrefix} description={props.error} role="alert" data-testid="onboarding-error" /> : null}
      </div>
      <div className="lr-onboarding-footer">
        {props.step === "welcome" ? <Button type="primary" onClick={() => props.onAdvance("start")}>{t.welcome.start}</Button> : null}
        {props.step === "accessibility" ? <><Button type="text" onClick={() => props.onAdvance("skip")} disabled={props.advancing}>{t.a11y.later}</Button><Button type="primary" onClick={() => props.onAdvance("continue")} disabled={props.advancing || props.a11y === "checking"}>{t.a11y.continue}</Button></> : null}
        {props.step === "provider" ? <Button type="primary" onClick={() => props.onAdvance("continue")} disabled={props.advancing}>{t.provider.continue}</Button> : null}
        {props.step === "history" ? <><Button type="text" onClick={() => props.onAdvance("skip")} disabled={props.advancing || props.historyBusy}>{t.history.skip}</Button><Button type="primary" icon={props.historyBusy ? <Spin size="small" /> : undefined} onClick={props.onEnableHistory} disabled={props.advancing || props.historyBusy}>{props.historyBusy ? t.history.enabling : t.history.enable}</Button></> : null}
        {props.step === "shortcuts" ? <Button type="primary" onClick={() => props.onAdvance("complete")} disabled={props.advancing}>{t.shortcuts.continue}</Button> : null}
        {props.step === "done" ? <><Button type="text" onClick={() => props.onFinish(false)}>{t.done.tray}</Button><Button type="primary" onClick={() => props.onFinish(true)}>{t.done.openApp}</Button></> : null}
      </div>
    </main>
  );
}

export default OnboardingView;
