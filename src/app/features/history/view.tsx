import {
  ActionIcon,
  Alert,
  Badge,
  Button,
  Checkbox,
  Group,
  Loader,
  Paper,
  Stack,
  Text,
  TextInput,
  Title,
} from "@mantine/core";
import { Search, Star } from "lucide-react";
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

  return (
    <Stack gap="md" aria-label={t.title} data-testid="history-page">
      <Title order={3}>{t.title}</Title>

      <Group gap="sm" wrap="wrap" align="flex-end">
        <TextInput
          placeholder={t.searchPlaceholder}
          aria-label={t.search}
          value={props.query}
          w={220}
          onChange={(e) => props.onQueryChange(e.currentTarget.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") props.onSearch();
          }}
        />
        <Button variant="light" loading={props.busy && props.state === "loading"} onClick={props.onSearch}>
          {t.search}
        </Button>
        <Checkbox
          label={t.favoritesOnly}
          checked={props.favoritesOnly}
          onChange={(e) => props.onFavoritesOnlyChange(e.currentTarget.checked)}
        />
        <Button variant="subtle" size="xs" onClick={() => props.onExport("csv")}>
          {t.exportCsv}
        </Button>
        <Button variant="subtle" size="xs" onClick={() => props.onExport("json")}>
          {t.exportJson}
        </Button>
      </Group>

      {props.notice && (
        <Alert data-testid="history-notice" role="status">
          {props.notice}
        </Alert>
      )}

      {props.state === "loading" && (
        <Group gap="sm" data-testid="history-loading">
          <Loader size="sm" />
          <Text size="sm" c="dimmed">
            {t.search}…
          </Text>
        </Group>
      )}

      {(props.state === "disabled" ||
        props.state === "empty" ||
        props.state === "search-empty") && (
        <Stack align="center" gap="xs" py="xl" data-testid={`history-${props.state}`}>
          <Search size={28} aria-hidden />
          <Text fw={500}>
            {props.state === "disabled"
              ? t.disabledTitle
              : props.state === "empty"
                ? t.emptyTitle
                : t.noMatchesTitle}
          </Text>
          {props.state === "disabled" && (
            <Text size="sm" c="dimmed">
              {t.disabledHint}
            </Text>
          )}
        </Stack>
      )}

      {props.state === "populated" && (
        <>
          <Stack gap="xs" data-testid="history-list">
            {props.items.map((session) => (
              <Paper
                key={session.session_uuid}
                withBorder
                p="sm"
                data-corrupt={session.corrupt ? "true" : undefined}
              >
                <Group justify="space-between" wrap="nowrap" align="flex-start">
                  <div style={{ minWidth: 0 }}>
                    <Text size="sm" lineClamp={2}>
                      {session.source_text ?? ""}
                    </Text>
                    <Text size="xs" c="dimmed">
                      {session.target_language} · {session.trigger_source}
                    </Text>
                    {session.corrupt && (
                      <Badge color="warning" variant="light" mt={4}>
                        {t.corruptLabel}
                      </Badge>
                    )}
                  </div>
                  <Group gap="xs" wrap="nowrap">
                    <ActionIcon
                      variant={session.is_favorite ? "filled" : "light"}
                      color={session.is_favorite ? "warning" : "gray"}
                      aria-label={session.is_favorite ? t.unfavorite : t.favorite}
                      onClick={() => props.onToggleFavorite(session)}
                    >
                      <Star
                        size={16}
                        fill={session.is_favorite ? "currentColor" : "none"}
                        aria-hidden
                      />
                    </ActionIcon>
                    <Button variant="subtle" color="danger" size="xs" onClick={() => props.onRemove(session)}>
                      {t.delete}
                    </Button>
                  </Group>
                </Group>
              </Paper>
            ))}
          </Stack>
          {props.hasMore && (
            <Button variant="light" onClick={props.onLoadMore}>
              {t.loadMore}
            </Button>
          )}
        </>
      )}
    </Stack>
  );
}

export default HistoryView;
