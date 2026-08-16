/**
 * Settings-window IPC: tray-action fan-out + section navigation events.
 * Mirrors the legacy Solid App container's contract exactly. Command wrappers
 * are reused from their owning features (translation/popup-ipc, ocr/ipc).
 */
import { commands } from "../../bridge/invoke";
import { listen } from "../../bridge/event";
import { startOcrCapture } from "../ocr/ipc";
import { translateSelection } from "../translation/popup-ipc";
import type { SettingsSection } from "./model";

const NAVIGABLE: SettingsSection[] = [
  "provider-center",
  "keystore-recovery",
  "shortcuts",
  "privacy",
  "history",
  "vocabulary",
  "dictionary",
];

export type TrayAction =
  | "translate-clipboard"
  | "translate-selection"
  | "ocr-capture"
  | "ocr-capture-shortcut"
  | "history"
  | "switch-provider"
  | "settings";

/** Runs a tray action; returns the section to navigate to, if any. */
export function runTrayAction(action: string): SettingsSection | null {
  switch (action) {
    case "translate-clipboard":
      void commands.translateClipboard();
      return null;
    case "translate-selection":
      void translateSelection();
      return null;
    case "ocr-capture":
      void startOcrCapture("tray");
      return null;
    case "ocr-capture-shortcut":
      void startOcrCapture("shortcut");
      return null;
    case "history":
      return "history";
    case "switch-provider":
    case "settings":
      return "provider-center";
    default:
      return null;
  }
}

export function isNavigableSection(value: string): value is SettingsSection {
  return (NAVIGABLE as string[]).includes(value);
}

/** Subscribe to tray-action + navigate. Returns an unlisten fn. */
export async function onWindowNavigation(
  onTrayAction: (action: string) => void,
  onNavigate: (section: SettingsSection) => void,
): Promise<() => void> {
  const u1 = await listen<string>("tray-action", (e) => onTrayAction(e.payload));
  const u2 = await listen<string>("navigate", (e) => {
    if (isNavigableSection(e.payload)) onNavigate(e.payload);
  });
  return () => {
    u1();
    u2();
  };
}
