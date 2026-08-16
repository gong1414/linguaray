/** Onboarding domain model — mirrors the Rust command contract exactly. */

export type OnboardingStepName =
  | "welcome"
  | "accessibility"
  | "provider"
  | "history"
  | "shortcuts"
  | "done";

export const STEP_ORDER: OnboardingStepName[] = [
  "welcome",
  "accessibility",
  "provider",
  "history",
  "shortcuts",
  "done",
];

export type AdvanceEvent = "start" | "continue" | "skip" | "complete";

/** null = still checking; "unsupported" = platform has no such permission. */
export type PermissionState = "checking" | "granted" | "denied" | "error" | "unsupported";

export type ShortcutCombo = { action: string; combo: string };

export type OnboardingStatus = { complete: boolean; step: OnboardingStepName };

export type ShortcutSnapshot = { entries?: ShortcutCombo[] };
