import { DictionaryView } from "./view";
import { useDictionaryController } from "./controller";

/** Self-composing settings section — the window file only routes. */
export function DictionarySection() {
  const c = useDictionaryController();
  return <DictionaryView c={c} />;
}
