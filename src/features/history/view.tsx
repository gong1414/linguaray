import { Alert, Button, Card, Checkbox, Empty, Input, Spin, Tag, Tooltip, Typography } from "antd";
import { DeleteOutlined, HistoryOutlined, SearchOutlined, StarFilled, StarOutlined } from "@ant-design/icons";
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

/** Pure presentational Ant Design History page. */
export function HistoryView(props: HistoryViewProps) {
  const t = HISTORY_COPY[props.locale];
  const styles = useUiStyles();

  return (
    <section className={styles.page} aria-label={t.title} data-testid="history-page">
      <div className={styles.rowBetween}>
        <Typography.Title level={4} className={styles.title}>{t.title}</Typography.Title>
        <div className={styles.row}>
          <Button type="text" size="small" onClick={() => props.onExport("csv")}>{t.exportCsv}</Button>
          <Button type="text" size="small" onClick={() => props.onExport("json")}>{t.exportJson}</Button>
        </div>
      </div>
      <div className={styles.rowWrap}>
        <Input prefix={<SearchOutlined aria-hidden />} placeholder={t.searchPlaceholder} aria-label={t.search} value={props.query} onChange={(e) => props.onQueryChange(e.currentTarget.value)} onKeyDown={(e) => e.key === "Enter" && props.onSearch()} />
        <Button icon={props.busy && props.state === "loading" ? <Spin size="small" /> : <SearchOutlined aria-hidden />} disabled={props.busy && props.state === "loading"} onClick={props.onSearch}>{t.search}</Button>
        <Checkbox checked={props.favoritesOnly} onChange={(e) => props.onFavoritesOnlyChange(e.target.checked)}>{t.favoritesOnly}</Checkbox>
      </div>
      {props.notice ? <Alert type="info" showIcon title={props.notice} role="status" data-testid="history-notice" /> : null}
      {props.state === "loading" ? <div className={styles.row} data-testid="history-loading"><Spin size="small" /><Typography.Text>{t.search}…</Typography.Text></div> : null}
      {(props.state === "disabled" || props.state === "empty" || props.state === "search-empty") ? (
        <Empty
          image={<HistoryOutlined aria-hidden />}
          data-testid={`history-${props.state}`}
          description={
            <>
              <Typography.Text strong>{props.state === "disabled" ? t.disabledTitle : props.state === "empty" ? t.emptyTitle : t.noMatchesTitle}</Typography.Text>
              {props.state === "disabled" ? <><br /><Typography.Text type="secondary">{t.disabledHint}</Typography.Text></> : null}
            </>
          }
        />
      ) : null}
      {props.state === "populated" ? (
        <>
          <div className={styles.list} data-testid="history-list">
            {props.items.map((session) => (
              <Card key={session.session_uuid} size="small" data-corrupt={session.corrupt ? "true" : undefined}>
                <div className={styles.rowBetween}>
                  <div className={styles.stackTight}>
                    <Typography.Text className={styles.preWrap}>{session.source_text ?? ""}</Typography.Text>
                    <Typography.Text type="secondary">{session.target_language} · {session.trigger_source}</Typography.Text>
                    {session.corrupt ? <Tag color="warning">{t.corruptLabel}</Tag> : null}
                  </div>
                  <div className={styles.row}>
                    <Tooltip title={session.is_favorite ? t.unfavorite : t.favorite}>
                      <Button type={session.is_favorite ? "primary" : "text"} size="small" icon={session.is_favorite ? <StarFilled aria-hidden /> : <StarOutlined aria-hidden />} aria-label={session.is_favorite ? t.unfavorite : t.favorite} onClick={() => props.onToggleFavorite(session)} />
                    </Tooltip>
                    <Button type="text" size="small" icon={<DeleteOutlined aria-hidden />} onClick={() => props.onRemove(session)}>{t.delete}</Button>
                  </div>
                </div>
              </Card>
            ))}
          </div>
          {props.hasMore ? <Button onClick={props.onLoadMore}>{t.loadMore}</Button> : null}
        </>
      ) : null}
    </section>
  );
}

export default HistoryView;
