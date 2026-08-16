/** Shell-scoped IPC: macOS Accessibility status. */
import { commands } from "../../bridge/invoke";

export const a11yStatus = (): Promise<boolean> => commands.a11yStatus();
