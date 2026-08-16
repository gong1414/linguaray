/**
 * Typed command wrappers for the onboarding Rust commands. Names, args and
 * returns mirror the contract in src-tauri (snake_case commands, camelCase
 * arg keys). The ONLY @tauri-apps touchpoint is ../../bridge/invoke.
 */
import { commands } from "../../bridge/invoke";
import type { AdvanceEvent, OnboardingStatus, OnboardingStepName, ShortcutSnapshot } from "./model";

export const getOnboardingStatus = (): Promise<OnboardingStatus> =>
  commands.onboardingStatus();

export const onboardingNext = (
  step: OnboardingStepName,
  event: AdvanceEvent,
): Promise<OnboardingStepName> => commands.onboardingNext(step, event);

export const completeOnboarding = (): Promise<void> =>
  commands.onboardingComplete().then(() => undefined);

export const a11yStatus = (): Promise<boolean> => commands.a11yStatus();

export const screenCaptureStatus = (): Promise<boolean> => commands.screenCaptureStatus();

export const listProviders = (): Promise<unknown[]> => commands.providerList();

export const listShortcuts = (): Promise<ShortcutSnapshot> =>
  commands.shortcutList();

export const setHistoryEnabled = (enabled: boolean): Promise<void> =>
  commands.historySetEnabled(enabled).then(() => undefined);

export const openSettingsSection = (section: string): Promise<void> =>
  commands.openSettingsWindow(section).then(() => undefined);
