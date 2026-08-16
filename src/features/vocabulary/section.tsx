import { VocabularyView } from "./view";
import { useVocabularyController } from "./controller";

/** Self-composing settings section — the window file only routes. */
export function VocabularySection() {
  const c = useVocabularyController();
  return <VocabularyView c={c} />;
}
