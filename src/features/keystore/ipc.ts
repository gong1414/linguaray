/** Keystore commands. `keystore_health` "" = healthy; non-empty = fail-closed reason. */
import { commands } from "../../bridge/invoke";

export const keystoreHealth = (): Promise<string> => commands.keystoreHealth();
export const archiveKeystore = (): Promise<string> => commands.archiveKeystore();
export const resetKeystore = (): Promise<string | null> => commands.resetKeystore();
