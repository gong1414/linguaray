import { Alert, Button, Card, Empty, Form, Input, Spin, Typography } from "antd";
import { DeleteOutlined, PlusOutlined, TranslationOutlined } from "@ant-design/icons";
import { detectLocale } from "../../app/i18n";
import { useUiStyles } from "../../ui/styles";
import { VOCABULARY_COPY } from "./copy";
import type { VocabularyController } from "./controller";

/** Pure presentational Ant Design vocabulary page. */
export function VocabularyView({ c }: { c: VocabularyController }) {
  const t = VOCABULARY_COPY[detectLocale()];
  const styles = useUiStyles();

  return (
    <section className={styles.page} aria-label={t.title} data-testid="vocabulary-page">
      <Typography.Title level={4} className={styles.title}>{t.title}</Typography.Title>
      <div className={styles.rowWrap}>
        <Form.Item label={t.word} className={styles.fieldSmall}>
          <Input value={c.word} aria-label={t.word} onChange={(e) => c.setWord(e.currentTarget.value)} />
        </Form.Item>
        <Form.Item label={t.definition} className={styles.grow}>
          <Input value={c.definition} aria-label={t.definition} onChange={(e) => c.setDefinition(e.currentTarget.value)} />
        </Form.Item>
        <Button type="primary" icon={c.busy ? <Spin size="small" /> : <PlusOutlined aria-hidden />} disabled={c.busy || !c.word.trim()} onClick={c.add}>{t.add}</Button>
      </div>
      <div className={styles.end}>
        <Button type="text" size="small" onClick={() => c.exportFile("csv")}>{t.exportCsv}</Button>
        <Button type="text" size="small" onClick={() => c.exportFile("json")}>{t.exportJson}</Button>
        <Button type="text" size="small" onClick={() => c.exportFile("anki")}>{t.exportAnki}</Button>
      </div>
      {c.notice ? <Alert type="info" showIcon title={c.notice} data-testid="vocabulary-notice" /> : null}
      {c.items.length === 0 ? (
        <Empty image={<TranslationOutlined aria-hidden />} description={<><Typography.Text strong>{t.empty}</Typography.Text><br /><Typography.Text type="secondary">{t.hint}</Typography.Text></>} data-testid="vocabulary-empty" />
      ) : (
        <div className={styles.list} data-testid="vocabulary-list">
          {c.items.map((item) => (
            <Card key={item.item_uuid} size="small">
              <div className={styles.rowBetween}>
                <div className={styles.row}>
                  <Typography.Text strong>{item.word}</Typography.Text>
                  <Typography.Text type="secondary" className={styles.clamp}>{item.definition}</Typography.Text>
                </div>
                <Button type="text" size="small" icon={<DeleteOutlined aria-hidden />} onClick={() => c.remove(item.item_uuid)}>{t.delete}</Button>
              </div>
            </Card>
          ))}
        </div>
      )}
    </section>
  );
}

export default VocabularyView;
