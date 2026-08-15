/** Keystore commands. `keystore_health` "" = healthy; non-empty = fail-closed reason. */
import { invoke } from "../../../bridge/invoke";

export const keystoreHealth = (): Promise<string> => invoke<string>("keystore_health");
export const archiveKeystore = (): Promise<string> => invoke<string>("archive_keystore");
export const resetKeystore = (): Promise<string | null> =>
  invoke<string | null>("reset_keystore");
