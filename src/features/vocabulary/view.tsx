import {
  Button,
  Card,
  Field,
  Input,
  MessageBar,
  MessageBarBody,
  Spinner,
  Text,
} from "@fluentui/react-components";
import { BookLetterRegular, DeleteRegular } from "@fluentui/react-icons";
import { detectLocale } from "../../app/i18n";
import { useUiStyles } from "../../ui/styles";
import { VOCABULARY_COPY } from "./copy";
import type { VocabularyController } from "./controller";

/** Pure presentational Vocabulary page. */
export function VocabularyView({ c }: { c: VocabularyController }) {
  const t = VOCABULARY_COPY[detectLocale()];
  const styles = useUiStyles();

  return (
    <section className={styles.page} aria-label={t.title} data-testid="vocabulary-page">
      <Text as="h2" size={500} weight="semibold" className={styles.title}>{t.title}</Text>

      <div className={styles.rowWrap}>
        <Field label={t.word} className={styles.fieldSmall}>
          <Input value={c.word} aria-label={t.word} onChange={(e) => c.setWord(e.currentTarget.value)} />
        </Field>
        <Field label={t.definition} className={styles.grow}>
          <Input value={c.definition} aria-label={t.definition} onChange={(e) => c.setDefinition(e.currentTarget.value)} />
        </Field>
        <Button
          appearance="primary"
          icon={c.busy ? <Spinner size="tiny" /> : undefined}
          disabled={c.busy || !c.word.trim()}
          onClick={c.add}
        >
          {t.add}
        </Button>
      </div>
      <div className={styles.end}>
        <Button appearance="subtle" size="small" onClick={() => c.exportFile("csv")}>{t.exportCsv}</Button>
        <Button appearance="subtle" size="small" onClick={() => c.exportFile("json")}>{t.exportJson}</Button>
        <Button appearance="subtle" size="small" onClick={() => c.exportFile("anki")}>{t.exportAnki}</Button>
      </div>

      {c.notice && <MessageBar intent="info" data-testid="vocabulary-notice"><MessageBarBody>{c.notice}</MessageBarBody></MessageBar>}

      {c.items.length === 0 ? (
        <div className={styles.empty} data-testid="vocabulary-empty">
          <BookLetterRegular fontSize={28} aria-hidden />
          <Text weight="semibold">{t.empty}</Text>
          <Text size={300}>{t.hint}</Text>
        </div>
      ) : (
        <div className={styles.list} data-testid="vocabulary-list">
          {c.items.map((item) => (
            <Card key={item.item_uuid} appearance="outline" size="small">
              <div className={styles.rowBetween}>
                <div className={styles.row}>
                  <Text weight="semibold">{item.word}</Text>
                  <Text size={300} className={styles.clamp}>{item.definition}</Text>
                </div>
                <Button
                  appearance="subtle"
                  size="small"
                  icon={<DeleteRegular />}
                  onClick={() => c.remove(item.item_uuid)}
                >
                  {t.delete}
                </Button>
              </div>
            </Card>
          ))}
        </div>
      )}
    </section>
  );
}

export default VocabularyView;
