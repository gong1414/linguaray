import { ShortcutsView } from "./view";
import { useShortcutsController } from "./controller";

/** Self-composing settings section — the window file only routes. */
export function ShortcutsSection() {
  const c = useShortcutsController();
  return <ShortcutsView c={c} />;
}
