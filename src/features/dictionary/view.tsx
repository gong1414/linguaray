import { Alert, Button, Card, Divider, Empty, Form, Input, Spin, Tag, Typography } from "antd";
import { BookOutlined, SearchOutlined } from "@ant-design/icons";
import { detectLocale } from "../../app/i18n";
import { useUiStyles } from "../../ui/styles";
import { DICTIONARY_COPY } from "./copy";
import type { DictionaryController } from "./controller";

/** Pure presentational Ant Design offline Dictionary page. */
export function DictionaryView({ c }: { c: DictionaryController }) {
  const t = DICTIONARY_COPY[detectLocale()];
  const styles = useUiStyles();

  return (
    <section className={styles.page} aria-label={t.title} data-testid="dictionary-page">
      <Typography.Title level={4} className={styles.title}>{t.title}</Typography.Title>
      <div className={styles.row}>
        <Form.Item label={t.word} className={styles.grow}>
          <Input value={c.word} aria-label={t.word} onChange={(e) => c.setWord(e.currentTarget.value)} onKeyDown={(e) => e.key === "Enter" && c.lookup()} />
        </Form.Item>
        <Button type="primary" icon={<SearchOutlined aria-hidden />} onClick={c.lookup} disabled={!c.word.trim()}>{t.lookup}</Button>
      </div>
      {c.result ? (
        <Card size="small" data-testid="dictionary-result">
          <Typography.Paragraph>{c.result.definition}</Typography.Paragraph>
          <Typography.Text type="secondary">{t.source.replace("{source}", c.result.source)}</Typography.Text>
        </Card>
      ) : null}
      {c.miss ? <Typography.Text type="secondary" data-testid="dictionary-miss">{t.noResult}</Typography.Text> : null}
      <Divider className={styles.dividerSpace} />
      <div className={styles.stack}>
        <Typography.Text strong>{t.install}</Typography.Text>
        <Form.Item label={t.sourceDir}>
          <Input value={c.sourceDir} aria-label={t.sourceDir} onChange={(e) => c.setSourceDir(e.currentTarget.value)} />
        </Form.Item>
        <div className={styles.rowWrap}>
          <Form.Item label={t.packageId} className={styles.fieldSmall}><Input value={c.packageId} aria-label={t.packageId} onChange={(e) => c.setPackageId(e.currentTarget.value)} /></Form.Item>
          <Form.Item label={t.packageName} className={styles.fieldSmall}><Input value={c.packageName} aria-label={t.packageName} onChange={(e) => c.setPackageName(e.currentTarget.value)} /></Form.Item>
          <Form.Item label={t.version} className={styles.fieldTiny}><Input value={c.version} aria-label={t.version} onChange={(e) => c.setVersion(e.currentTarget.value)} /></Form.Item>
          <Button icon={c.installing ? <Spin size="small" /> : undefined} disabled={c.installing || !c.sourceDir.trim() || !c.packageId.trim()} onClick={c.install}>{t.install}</Button>
        </div>
      </div>
      {c.packages.length === 0 ? (
        <Empty image={<BookOutlined aria-hidden />} description={t.noPackages} data-testid="dictionary-no-packages" />
      ) : (
        <div className={styles.rowWrap}>{c.packages.map((p) => <Tag key={p.package_id} color="blue">{p.name}</Tag>)}</div>
      )}
      {c.notice ? <Alert type="info" showIcon title={c.notice} data-testid="dictionary-notice" /> : null}
      {c.error ? <Alert type="error" showIcon title={c.error} data-testid="dictionary-error" /> : null}
    </section>
  );
}

export default DictionaryView;
