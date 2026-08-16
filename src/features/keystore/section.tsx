import { KeystoreRecoveryView } from "./view";
import { useKeystoreController } from "./controller";

/** Self-composing settings section — the window file only routes. */
export function KeystoreSection() {
  const c = useKeystoreController();
  return <KeystoreRecoveryView c={c} />;
}
