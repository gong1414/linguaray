/**
 * Typed command wrappers for the onboarding Rust commands. Names, args and
 * returns mirror the contract in src-tauri (snake_case commands, camelCase
 * arg keys). The ONLY @tauri-apps touchpoint is ../../bridge/invoke.
 */
import { invoke } from "../../../bridge/invoke";
import type { AdvanceEvent, OnboardingStatus, OnboardingStepName, ShortcutSnapshot } from "./model";

export const getOnboardingStatus = (): Promise<OnboardingStatus> =>
  invoke<OnboardingStatus>("onboarding_status");

export const onboardingNext = (
  step: OnboardingStepName,
  event: AdvanceEvent,
): Promise<OnboardingStepName> => invoke<OnboardingStepName>("onboarding_next", { step, event });

export const completeOnboarding = (): Promise<void> => invoke<void>("onboarding_complete");

export const a11yStatus = (): Promise<boolean> => invoke<boolean>("a11y_status");

export const screenCaptureStatus = (): Promise<boolean> =>
  invoke<boolean>("screen_capture_status");

export const listProviders = (): Promise<unknown[]> => invoke<unknown[]>("provider_list");

export const listShortcuts = (): Promise<ShortcutSnapshot> =>
  invoke<ShortcutSnapshot>("shortcut_list");

export const setHistoryEnabled = (enabled: boolean): Promise<void> =>
  invoke<void>("history_set_enabled", { enabled });

export const openSettingsSection = (section: string): Promise<void> =>
  invoke<void>("open_settings_window", { section });
