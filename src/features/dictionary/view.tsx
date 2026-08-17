import {
  Badge,
  Button,
  Card,
  Divider,
  Field,
  Input,
  MessageBar,
  MessageBarBody,
  Spinner,
  Text,
} from "@fluentui/react-components";
import { BookRegular, SearchRegular } from "@fluentui/react-icons";
import { detectLocale } from "../../app/i18n";
import { useUiStyles } from "../../ui/styles";
import { DICTIONARY_COPY } from "./copy";
import type { DictionaryController } from "./controller";

/** Pure presentational offline Dictionary page. */
export function DictionaryView({ c }: { c: DictionaryController }) {
  const t = DICTIONARY_COPY[detectLocale()];
  const styles = useUiStyles();

  return (
    <section className={styles.page} aria-label={t.title} data-testid="dictionary-page">
      <Text as="h2" size={300} weight="semibold" className={styles.title}>{t.title}</Text>

      <div className={styles.row}>
        <Field label={t.word} className={styles.grow}>
          <Input
            value={c.word}
            aria-label={t.word}
            onChange={(e) => c.setWord(e.currentTarget.value)}
            onKeyDown={(e) => e.key === "Enter" && c.lookup()}
          />
        </Field>
        <Button appearance="primary" icon={<SearchRegular />} onClick={c.lookup} disabled={!c.word.trim()}>{t.lookup}</Button>
      </div>

      {c.result && (
        <Card appearance="filled-alternative" size="small" data-testid="dictionary-result">
          <Text>{c.result.definition}</Text>
          <Text size={200} className={styles.muted}>{t.source.replace("{source}", c.result.source)}</Text>
        </Card>
      )}
      {c.miss && <Text size={300} className={styles.muted} data-testid="dictionary-miss">{t.noResult}</Text>}

      <Divider className={styles.dividerSpace} />

      <div className={styles.stack}>
        <Text weight="semibold">{t.install}</Text>
        <Field label={t.sourceDir}>
          <Input value={c.sourceDir} aria-label={t.sourceDir} onChange={(e) => c.setSourceDir(e.currentTarget.value)} />
        </Field>
        <div className={styles.rowWrap}>
          <Field label={t.packageId} className={styles.fieldSmall}>
            <Input value={c.packageId} aria-label={t.packageId} onChange={(e) => c.setPackageId(e.currentTarget.value)} />
          </Field>
          <Field label={t.packageName} className={styles.fieldSmall}>
            <Input value={c.packageName} aria-label={t.packageName} onChange={(e) => c.setPackageName(e.currentTarget.value)} />
          </Field>
          <Field label={t.version} className={styles.fieldTiny}>
            <Input value={c.version} aria-label={t.version} onChange={(e) => c.setVersion(e.currentTarget.value)} />
          </Field>
          <Button
            appearance="secondary"
            icon={c.installing ? <Spinner size="tiny" /> : undefined}
            disabled={c.installing || !c.sourceDir.trim() || !c.packageId.trim()}
            onClick={c.install}
          >
            {t.install}
          </Button>
        </div>
      </div>

      {c.packages.length === 0 ? (
        <div className={styles.empty} data-testid="dictionary-no-packages">
          <BookRegular fontSize={26} aria-hidden />
          <Text size={300}>{t.noPackages}</Text>
        </div>
      ) : (
        <div className={styles.rowWrap}>
          {c.packages.map((p) => <Badge key={p.package_id} appearance="tint" color="brand">{p.name}</Badge>)}
        </div>
      )}

      {c.notice && <MessageBar intent="info" data-testid="dictionary-notice"><MessageBarBody>{c.notice}</MessageBarBody></MessageBar>}
      {c.error && <MessageBar intent="error" data-testid="dictionary-error"><MessageBarBody>{c.error}</MessageBarBody></MessageBar>}
    </section>
  );
}

export default DictionaryView;
