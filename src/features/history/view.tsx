import {
  Badge,
  Button,
  Card,
  Checkbox,
  Input,
  MessageBar,
  MessageBarBody,
  Spinner,
  Text,
  Tooltip,
} from "@fluentui/react-components";
import { DeleteRegular, SearchRegular, StarFilled, StarRegular } from "@fluentui/react-icons";
import { useUiStyles } from "../../ui/styles";
import { HISTORY_COPY } from "./copy";
import type { HistoryItem, HistoryLoadState } from "./model";

export type HistoryViewProps = {
  locale: "zh" | "en";
  state: HistoryLoadState;
  items: HistoryItem[];
  query: string;
  favoritesOnly: boolean;
  hasMore: boolean;
  notice: string;
  busy: boolean;
  onQueryChange: (q: string) => void;
  onSearch: () => void;
  onFavoritesOnlyChange: (v: boolean) => void;
  onLoadMore: () => void;
  onToggleFavorite: (item: HistoryItem) => void;
  onRemove: (item: HistoryItem) => void;
  onExport: (format: "csv" | "json") => void;
};

/** Pure presentational History page. */
export function HistoryView(props: HistoryViewProps) {
  const t = HISTORY_COPY[props.locale];
  const styles = useUiStyles();

  return (
    <section className={styles.page} aria-label={t.title} data-testid="history-page">
      <div className={styles.rowBetween}>
        <Text as="h2" size={300} weight="semibold" className={styles.title}>{t.title}</Text>
        <div className={styles.row}>
          <Button appearance="subtle" size="small" onClick={() => props.onExport("csv")}>{t.exportCsv}</Button>
          <Button appearance="subtle" size="small" onClick={() => props.onExport("json")}>{t.exportJson}</Button>
        </div>
      </div>
      <div className={styles.rowWrap}>
          <Input
            contentBefore={<SearchRegular />}
            placeholder={t.searchPlaceholder}
            aria-label={t.search}
            value={props.query}
            onChange={(e) => props.onQueryChange(e.currentTarget.value)}
            onKeyDown={(e) => e.key === "Enter" && props.onSearch()}
          />
          <Button
            appearance="secondary"
            icon={props.busy && props.state === "loading" ? <Spinner size="tiny" /> : undefined}
            disabled={props.busy && props.state === "loading"}
            onClick={props.onSearch}
          >
            {t.search}
          </Button>
          <Checkbox
            label={t.favoritesOnly}
            checked={props.favoritesOnly}
            onChange={(_, data) => props.onFavoritesOnlyChange(Boolean(data.checked))}
          />
      </div>

      {props.notice && <MessageBar intent="info" role="status" data-testid="history-notice"><MessageBarBody>{props.notice}</MessageBarBody></MessageBar>}

      {props.state === "loading" && (
        <div className={styles.row} data-testid="history-loading"><Spinner size="tiny" /><Text size={300}>{t.search}…</Text></div>
      )}

      {(props.state === "disabled" || props.state === "empty" || props.state === "search-empty") && (
        <div className={styles.empty} data-testid={`history-${props.state}`}>
          <SearchRegular fontSize={28} aria-hidden />
          <Text weight="semibold">
            {props.state === "disabled" ? t.disabledTitle : props.state === "empty" ? t.emptyTitle : t.noMatchesTitle}
          </Text>
          {props.state === "disabled" && <Text size={300}>{t.disabledHint}</Text>}
        </div>
      )}

      {props.state === "populated" && (
        <>
          <div className={styles.list} data-testid="history-list">
            {props.items.map((session) => (
              <Card key={session.session_uuid} appearance="filled-alternative" size="small" data-corrupt={session.corrupt ? "true" : undefined}>
                <div className={styles.rowBetween}>
                  <div className={styles.stackTight}>
                    <Text className={styles.preWrap}>{session.source_text ?? ""}</Text>
                    <Text size={200} className={styles.muted}>{session.target_language} · {session.trigger_source}</Text>
                    {session.corrupt && <Badge appearance="tint" color="warning">{t.corruptLabel}</Badge>}
                  </div>
                  <div className={styles.row}>
                    <Tooltip content={session.is_favorite ? t.unfavorite : t.favorite} relationship="label">
                      <Button
                        appearance={session.is_favorite ? "primary" : "subtle"}
                        size="small"
                        icon={session.is_favorite ? <StarFilled /> : <StarRegular />}
                        aria-label={session.is_favorite ? t.unfavorite : t.favorite}
                        onClick={() => props.onToggleFavorite(session)}
                      />
                    </Tooltip>
                    <Button appearance="subtle" size="small" icon={<DeleteRegular />} onClick={() => props.onRemove(session)}>{t.delete}</Button>
                  </div>
                </div>
              </Card>
            ))}
          </div>
          {props.hasMore && <Button appearance="secondary" onClick={props.onLoadMore}>{t.loadMore}</Button>}
        </>
      )}
    </section>
  );
}

export default HistoryView;
