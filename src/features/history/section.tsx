import { detectLocale } from "../../app/i18n";
import { useHistoryController } from "./controller";
import { HistoryView } from "./view";

/** Self-composing settings section — the window file only routes. */
export function HistorySection() {
  const c = useHistoryController();
  return (
    <HistoryView
      locale={detectLocale()}
      state={c.state}
      items={c.items}
      query={c.query}
      favoritesOnly={c.favoritesOnly}
      hasMore={c.hasMore}
      notice={c.notice}
      busy={c.busy}
      onQueryChange={c.setQuery}
      onSearch={c.search}
      onFavoritesOnlyChange={c.setFavoritesOnly}
      onLoadMore={c.loadMore}
      onToggleFavorite={c.toggleFavorite}
      onRemove={c.remove}
      onExport={c.exportFile}
    />
  );
}
