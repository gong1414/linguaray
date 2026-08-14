import { createSignal, For, onMount, Show, type Component } from "solid-js";
import { Button, EmptyState, TextField } from "@linguaray/ui";
import { Search, Star } from "lucide-solid";
import { detectLocale } from "../../i18n";
import { HISTORY_COPY } from "./history-copy";
import {
  historyDeleteSession,
  historyExport,
  historySearch,
  historyToggleFavorite,
} from "./history-ipc";
import { historyPrivacyStatus } from "./privacy-ipc";
import type { HistoryItem } from "./history-types";
import "./HistoryView.css";

type LoadState = "initial" | "loading" | "populated" | "empty" | "search-empty" | "disabled";

export const HistoryView: Component = () => {
  const t = HISTORY_COPY[detectLocale()];
  const [state, setState] = createSignal<LoadState>("initial");
  const [items, setItems] = createSignal<HistoryItem[]>([]);
  const [query, setQuery] = createSignal("");
  const [favoritesOnly, setFavoritesOnly] = createSignal(false);
  const [cursor, setCursor] = createSignal<string | null>(null);
  const [complete, setComplete] = createSignal(true);
  const [notice, setNotice] = createSignal("");

  const applyPage = (page: { items: HistoryItem[]; next_cursor: string | null; scan_complete: boolean }, append: boolean) => {
    const next = favoritesOnly() ? page.items.filter((s) => s.is_favorite) : page.items;
    const merged = append ? [...items(), ...next] : next;
    setItems(merged);
    setCursor(page.next_cursor);
    setComplete(page.scan_complete);
    if (merged.length === 0) {
      setState(query().trim() || favoritesOnly() ? "search-empty" : "empty");
    } else {
      setState("populated");
    }
  };

  const load = async (append = false) => {
    setState("loading");
    setNotice("");
    try {
      const privacy = await historyPrivacyStatus();
      if (!privacy.enabled) {
        setItems([]);
        setState("disabled");
        return;
      }
      const page = await historySearch(query().trim(), append ? cursor() : null);
      applyPage(page, append);
    } catch (e) {
      setNotice(String(e));
      setState(items().length ? "populated" : "empty");
    }
  };

  onMount(() => {
    void load(false);
  });

  const toggleFavorite = async (item: HistoryItem) => {
    const next = await historyToggleFavorite(item.session_uuid);
    setItems((prev) =>
      prev.map((s) => (s.session_uuid === item.session_uuid ? { ...s, is_favorite: next } : s)),
    );
    if (favoritesOnly() && !next) {
      setItems((prev) => prev.filter((s) => s.session_uuid !== item.session_uuid));
      if (items().length === 0) setState("search-empty");
    }
  };

  const remove = async (item: HistoryItem) => {
    await historyDeleteSession(item.session_uuid);
    setItems((prev) => prev.filter((s) => s.session_uuid !== item.session_uuid));
    if (items().length === 0) setState(query().trim() ? "search-empty" : "empty");
  };

  const exportFile = async (format: "csv" | "json") => {
    const filePath = `linguaray-history.${format}`;
    try {
      const written = await historyExport(filePath, format, {
        query: query().trim() || null,
        favorites_only: favoritesOnly(),
      });
      setNotice(t.exportDone.replace("{path}", written));
    } catch {
      setNotice(t.exportFailed);
    }
  };

  return (
    <section class="history-view" aria-label={t.title}>
      <header class="history-view__header">
        <h1>{t.title}</h1>
        <div class="history-view__toolbar">
          <TextField
            label={t.search}
            value={query()}
            placeholder={t.searchPlaceholder}
            onInput={(e) => setQuery(e.currentTarget.value)}
          />
          <Button variant="secondary" onClick={() => void load(false)}>
            {t.search}
          </Button>
          <label class="history-view__fav-filter">
            <input
              type="checkbox"
              checked={favoritesOnly()}
              onChange={(e) => {
                setFavoritesOnly(e.currentTarget.checked);
                void load(false);
              }}
            />
            {t.favoritesOnly}
          </label>
          <Button variant="ghost" onClick={() => void exportFile("csv")}>
            {t.exportCsv}
          </Button>
          <Button variant="ghost" onClick={() => void exportFile("json")}>
            {t.exportJson}
          </Button>
        </div>
      </header>

      <Show when={notice()}>
        <p class="history-view__notice" role="status">
          {notice()}
        </p>
      </Show>

      <Show when={state() === "loading"}>
        <p class="history-view__loading">{t.search}…</p>
      </Show>
      <Show when={state() === "disabled"}>
        <EmptyState icon={<Search size={32} />} title={t.disabled.title} description={t.disabled.hint} />
      </Show>
      <Show when={state() === "empty"}>
        <EmptyState icon={<Search size={32} />} title={t.empty.title} />
      </Show>
      <Show when={state() === "search-empty"}>
        <EmptyState icon={<Search size={32} />} title={t.noMatches.title} />
      </Show>
      <Show when={state() === "populated"}>
        <ul class="history-view__list">
          <For each={items()}>
            {(session) => (
              <li class="history-view__item" data-corrupt={session.corrupt}>
                <div class="history-view__item-body">
                  <p class="history-view__source">{session.source_text ?? ""}</p>
                  <p class="history-view__meta">
                    {session.target_language} · {session.trigger_source}
                  </p>
                  <Show when={session.corrupt}>
                    <span class="history-view__corrupt-badge" role="alert">
                      {t.corrupt.label}
                    </span>
                  </Show>
                </div>
                <div class="history-view__actions">
                  <Button
                    variant="ghost"
                    aria-label={session.is_favorite ? t.unfavorite : t.favorite}
                    onClick={() => void toggleFavorite(session)}
                  >
                    <Star size={16} fill={session.is_favorite ? "currentColor" : "none"} />
                  </Button>
                  <Button variant="ghost" onClick={() => void remove(session)}>
                    {t.delete}
                  </Button>
                </div>
              </li>
            )}
          </For>
        </ul>
        <Show when={!complete()}>
          <Button variant="secondary" onClick={() => void load(true)}>
            {t.loadMore}
          </Button>
        </Show>
      </Show>
    </section>
  );
};

export default HistoryView;
