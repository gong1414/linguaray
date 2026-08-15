import { useEffect, useRef } from "react";
import {
  Alert,
  Badge,
  Button,
  Group,
  Kbd,
  Paper,
  Stack,
  Stepper,
  Text,
  Title,
} from "@mantine/core";
import { ONBOARDING_COPY, type Locale } from "./copy";
import { STEP_ORDER, type OnboardingStepName, type PermissionState, type ShortcutCombo } from "./model";
import classes from "./onboarding.module.css";

export type OnboardingViewProps = {
  step: OnboardingStepName;
  locale: Locale;
  a11y: PermissionState;
  screenCapture: PermissionState;
  /** null = still checking. */
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

/** Soft Badge palette per honest permission state. */
const BADGE_COLOR: Record<PermissionState, string> = {
  checking: "gray",
  granted: "success",
  denied: "warning",
  error: "danger",
  unsupported: "gray",
};

function StatusBadge({ state, labels }: { state: PermissionState; labels: Record<string, string> }) {
  return (
    <Badge variant="light" color={BADGE_COLOR[state]} role="status">
      {labels[state] ?? state}
    </Badge>
  );
}

function stepTitle(t: (typeof ONBOARDING_COPY)[Locale], step: OnboardingStepName): string {
  if (step === "welcome") return t.welcome.title;
  if (step === "done") return t.done.title;
  if (step === "accessibility") return t.a11y.title;
  if (step === "provider") return t.provider.title;
  if (step === "history") return t.history.title;
  return t.shortcuts.title;
}

/**
 * Pure presentational onboarding (props + callbacks only — no IPC imports).
 * Focus follows the step change: the new step's heading receives focus for
 * keyboard and screen-reader users.
 */
export function OnboardingView(props: OnboardingViewProps) {
  const t = ONBOARDING_COPY[props.locale];
  const titleRef = useRef<HTMLHeadingElement>(null);

  useEffect(() => {
    titleRef.current?.focus();
  }, [props.step]);

  const stepIndex = STEP_ORDER.indexOf(props.step);

  const providerHint =
    props.providerCount === null
      ? t.provider.checking
      : props.providerCount === 0
        ? t.provider.noneBody
        : t.provider.count(props.providerCount);

  return (
    <main className={classes.shell} data-step={props.step} data-testid="onboarding">
      <Stepper
        active={stepIndex}
        size="xs"
        allowNextStepsSelect={false}
        aria-label={t.brand}
        className={classes.stepper}
      >
        {STEP_ORDER.map((s) => (
          <Stepper.Step key={s} label={t.stepLabels[s]} aria-label={t.stepLabels[s]} />
        ))}
      </Stepper>

      <div className={classes.content}>
        <Title order={2} ref={titleRef} tabIndex={-1} data-testid="onboarding-title">
          {stepTitle(t, props.step)}
        </Title>

        {props.step === "welcome" && (
          <Text c="dimmed" size="sm" className={classes.desc}>
            {t.welcome.body}
          </Text>
        )}

        {props.step === "accessibility" && (
          <>
            <Text c="dimmed" size="sm" className={classes.desc}>
              {t.a11y.body}
            </Text>
            <Paper withBorder p="sm" radius="md" className={classes.card}>
              <Group justify="space-between" wrap="nowrap">
                <Text size="sm">{t.a11y.title}</Text>
                <StatusBadge state={props.a11y} labels={t.a11y.status} />
              </Group>
              <Group justify="flex-end" mt="xs">
                <Button
                  variant="light"
                  size="xs"
                  onClick={props.onOpenA11ySettings}
                  disabled={props.a11y === "unsupported"}
                >
                  {t.a11y.openSettings}
                </Button>
              </Group>
            </Paper>
            <Paper withBorder p="sm" radius="md" className={classes.card}>
              <Group justify="space-between" wrap="nowrap">
                <Text size="sm">{t.a11y.screenTitle}</Text>
                <StatusBadge state={props.screenCapture} labels={t.a11y.status} />
              </Group>
              <Text size="xs" c="dimmed" mt={4}>
                {t.a11y.screenBody}
              </Text>
              <Group justify="flex-end" mt="xs">
                <Button
                  variant="light"
                  size="xs"
                  onClick={props.onOpenScreenCaptureSettings}
                  disabled={props.screenCapture === "unsupported"}
                >
                  {t.a11y.openScreenSettings}
                </Button>
              </Group>
            </Paper>
            <Button variant="subtle" size="xs" onClick={props.onRecheckPermissions} disabled={props.advancing}>
              {t.a11y.recheck}
            </Button>
          </>
        )}

        {props.step === "provider" && (
          <>
            <Text c="dimmed" size="sm" className={classes.desc}>
              {t.provider.body}
            </Text>
            <Paper withBorder p="sm" radius="md" className={classes.card}>
              <Text size="sm" c={props.providerCount === 0 ? "warning" : "dimmed"}>
                {providerHint}
              </Text>
              <Group justify="flex-end" mt="xs">
                <Button variant="light" size="xs" onClick={props.onOpenProviderSettings}>
                  {t.provider.openSettings}
                </Button>
              </Group>
            </Paper>
          </>
        )}

        {props.step === "history" && (
          <Text c="dimmed" size="sm" className={classes.desc}>
            {t.history.body}
          </Text>
        )}

        {props.step === "shortcuts" && (
          <>
            <Text c="dimmed" size="sm" className={classes.desc}>
              {t.shortcuts.body}
            </Text>
            <Stack gap="xs" data-testid="onboarding-shortcuts" className={classes.card}>
              {props.shortcuts.map((s) => (
                <Group key={s.action} justify="space-between" wrap="nowrap">
                  <Text size="sm">{t.shortcuts.combos[s.action] ?? s.action}</Text>
                  <Kbd>{s.combo}</Kbd>
                </Group>
              ))}
            </Stack>
            <Button variant="subtle" size="xs" onClick={props.onOpenShortcutsSettings}>
              {t.shortcuts.openSettings}
            </Button>
          </>
        )}

        {props.step === "done" && (
          <Text c="dimmed" size="sm" className={classes.desc}>
            {t.done.body}
          </Text>
        )}

        {props.error && (
          <Alert
            color="red"
            role="alert"
            data-testid="onboarding-error"
            title={t.errorPrefix}
            className={classes.card}
          >
            {props.error}
          </Alert>
        )}
      </div>

      <Group justify="flex-end" gap="sm" className={classes.footer}>
        {props.step === "welcome" && (
          <Button onClick={() => props.onAdvance("start")}>{t.welcome.start}</Button>
        )}
        {props.step === "accessibility" && (
          <>
            <Button variant="subtle" onClick={() => props.onAdvance("skip")} disabled={props.advancing}>
              {t.a11y.later}
            </Button>
            <Button
              onClick={() => props.onAdvance("continue")}
              disabled={props.advancing || props.a11y === "checking"}
            >
              {t.a11y.continue}
            </Button>
          </>
        )}
        {props.step === "provider" && (
          <Button onClick={() => props.onAdvance("continue")} disabled={props.advancing}>
            {t.provider.continue}
          </Button>
        )}
        {props.step === "history" && (
          <>
            <Button
              variant="subtle"
              onClick={() => props.onAdvance("skip")}
              disabled={props.advancing || props.historyBusy}
            >
              {t.history.skip}
            </Button>
            <Button
              onClick={props.onEnableHistory}
              disabled={props.advancing || props.historyBusy}
              loading={props.historyBusy}
            >
              {props.historyBusy ? t.history.enabling : t.history.enable}
            </Button>
          </>
        )}
        {props.step === "shortcuts" && (
          <Button onClick={() => props.onAdvance("complete")} disabled={props.advancing}>
            {t.shortcuts.continue}
          </Button>
        )}
        {props.step === "done" && (
          <>
            <Button variant="subtle" onClick={() => props.onFinish(false)}>
              {t.done.tray}
            </Button>
            <Button onClick={() => props.onFinish(true)}>{t.done.openApp}</Button>
          </>
        )}
      </Group>
    </main>
  );
}

export default OnboardingView;
