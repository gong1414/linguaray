import { ProviderCenterView } from "./view";
import { useProviderController } from "./controller";

/** Self-composing settings section — the window file only routes. */
export function ProviderSection() {
  const c = useProviderController();
  return <ProviderCenterView c={c} />;
}
