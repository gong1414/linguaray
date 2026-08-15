/**
 * History controller — search/filter/paging/favorite/delete/export, with the
 * same state machine as the Solid version (initial → loading → populated |
 * empty | search-empty | disabled). The favorites-only filter is applied
 * client-side on each page (legacy behavior kept).
 */
import { useCallback, useEffect, useRef, useState } from "react";
import { detectLocale } from "../../i18n";
import { HISTORY_COPY } from "./copy";
import * as ipc from "./ipc";
import type { HistoryItem, HistoryLoadState } from "./model";

export type HistoryController = {
  state: HistoryLoadState;
  items: HistoryItem[];
  query: string;
  favoritesOnly: boolean;
  hasMore: boolean;
  notice: string;
  busy: boolean;
  setQuery: (q: string) => void;
  search: () => void;
  setFavoritesOnly: (v: boolean) => void;
  loadMore: () => void;
  toggleFavorite: (item: HistoryItem) => void;
  remove: (item: HistoryItem) => void;
  exportFile: (format: "csv" | "json") => void;
};

export function useHistoryController(): HistoryController {
  const t = HISTORY_COPY[detectLocale()];
  const [state, setState] = useState<HistoryLoadState>("initial");
  const [items, setItems] = useState<HistoryItem[]>([]);
  const [query, setQuery] = useState("");
  const [favoritesOnly, setFavoritesOnly] = useState(false);
  const [hasMore, setHasMore] = useState(false);
  const [notice, setNotice] = useState("");
  const [busy, setBusy] = useState(false);

  const cursorRef = useRef<string | null>(null);
  const queryRef = useRef(query);
  const favoritesRef = useRef(favoritesOnly);
  queryRef.current = query;
  favoritesRef.current = favoritesOnly;
  const cancelledRef = useRef(false);

  const applyPage = useCallback((page: { items: HistoryItem[]; next_cursor: string | null; scan_complete: boolean }, append: boolean) => {
    const next = favoritesRef.current ? page.items.filter((s) => s.is_favorite) : page.items;
    setItems((prev) => {
      const merged = append ? [...prev, ...next] : next;
      if (merged.length === 0) {
        setState(queryRef.current.trim() || favoritesRef.current ? "search-empty" : "empty");
      } else {
        setState("populated");
      }
      return merged;
    });
    cursorRef.current = page.next_cursor;
    setHasMore(!page.scan_complete);
  }, []);

  const load = useCallback(async (append: boolean) => {
    setBusy(true);
    setNotice("");
    setState("loading");
    try {
      const enabled = await ipc.historyPrivacyEnabled();
      if (!enabled) {
        if (!cancelledRef.current) {
          setItems([]);
          setState("disabled");
        }
        return;
      }
      const page = await ipc.historySearch(
        queryRef.current.trim(),
        append ? cursorRef.current : null,
      );
      if (!cancelledRef.current) applyPage(page, append);
    } catch (e) {
      if (!cancelledRef.current) {
        setNotice(e instanceof Error ? e.message : String(e));
        setState((prev) => (prev === "populated" ? "populated" : "empty"));
      }
    } finally {
      if (!cancelledRef.current) setBusy(false);
    }
  }, [applyPage]);

  useEffect(() => {
    cancelledRef.current = false;
    void load(false);
    return () => {
      cancelledRef.current = true;
    };
  }, [load]);

  // Invariant: "populated" never coexists with an empty list (favorites-only
  // can drop the last row client-side after toggleFavorite).
  useEffect(() => {
    if (state === "populated" && items.length === 0) {
      setState(query.trim() || favoritesOnly ? "search-empty" : "empty");
    }
  }, [state, items, query, favoritesOnly]);

  return {
    state,
    items,
    query,
    favoritesOnly,
    hasMore,
    notice,
    busy,
    setQuery,
    search: () => void load(false),
    setFavoritesOnly: (v) => {
      setFavoritesOnly(v);
      favoritesRef.current = v;
      void load(false);
    },
    loadMore: () => void load(true),
    toggleFavorite: (item) => {
      void ipc.historyToggleFavorite(item.session_uuid).then((next) => {
        setItems((prev) =>
          prev.flatMap((s) => {
            if (s.session_uuid !== item.session_uuid) return [s];
            if (favoritesRef.current && !next) return [];
            return [{ ...s, is_favorite: next }];
          }),
        );
      });
    },
    remove: (item) => {
      void ipc.historyDeleteSession(item.session_uuid).then(() => {
        setItems((prev) => prev.filter((s) => s.session_uuid !== item.session_uuid));
      });
    },
    exportFile: (format) => {
      void ipc
        .chooseExportPath(`linguaray-history.${format}`)
        .then((filePath) => {
          if (!filePath) return;
          return ipc
            .historyExport(filePath, format, {
              query: queryRef.current.trim() || null,
              favorites_only: favoritesRef.current,
            })
            .then((written) => setNotice(t.exportDone.replace("{path}", written)));
        })
        .catch(() => setNotice(t.exportFailed));
    },
  };
}
