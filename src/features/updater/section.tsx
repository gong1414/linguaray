import { UpdaterPanelView } from "./view";
import { useUpdaterController } from "./controller";

/** Self-composing settings section — the window file only routes. */
export function UpdaterSection() {
  const c = useUpdaterController();
  return <UpdaterPanelView c={c} />;
}
