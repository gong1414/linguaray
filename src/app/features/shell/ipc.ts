/** Shell-scoped IPC: macOS Accessibility status. */
import { invoke } from "../../../bridge/invoke";

export const a11yStatus = (): Promise<boolean> => invoke<boolean>("a11y_status");
