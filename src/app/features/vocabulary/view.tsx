import {
  Alert,
  Button,
  Group,
  Stack,
  Text,
  TextInput,
  Title,
} from "@mantine/core";
import { BookMarked } from "lucide-react";
import { detectLocale } from "../../i18n";
import { VOCABULARY_COPY } from "./copy";
import type { VocabularyController } from "./controller";

/** Pure presentational Vocabulary page. */
export function VocabularyView({ c }: { c: VocabularyController }) {
  const t = VOCABULARY_COPY[detectLocale()];

  return (
    <Stack gap="md" aria-label={t.title} data-testid="vocabulary-page">
      <Title order={3}>{t.title}</Title>

      <Group gap="sm" align="flex-end" wrap="wrap">
        <TextInput
          label={t.word}
          value={c.word}
          aria-label={t.word}
          w={180}
          onChange={(e) => c.setWord(e.currentTarget.value)}
        />
        <TextInput
          label={t.definition}
          value={c.definition}
          aria-label={t.definition}
          style={{ flex: "1 1 12rem" }}
          onChange={(e) => c.setDefinition(e.currentTarget.value)}
        />
        <Button loading={c.busy} onClick={c.add} disabled={!c.word.trim()}>
          {t.add}
        </Button>
        <Button variant="subtle" size="xs" onClick={() => c.exportFile("csv")}>
          {t.exportCsv}
        </Button>
        <Button variant="subtle" size="xs" onClick={() => c.exportFile("json")}>
          {t.exportJson}
        </Button>
        <Button variant="subtle" size="xs" onClick={() => c.exportFile("anki")}>
          {t.exportAnki}
        </Button>
      </Group>

      {c.notice && (
        <Alert data-testid="vocabulary-notice" role="status">
          {c.notice}
        </Alert>
      )}

      {c.items.length === 0 ? (
        <Stack align="center" gap={4} py="xl" data-testid="vocabulary-empty">
          <BookMarked size={28} aria-hidden />
          <Text fw={500}>{t.empty}</Text>
          <Text size="sm" c="dimmed">{t.hint}</Text>
        </Stack>
      ) : (
        <Stack gap="xs" data-testid="vocabulary-list">
          {c.items.map((item) => (
            <Group key={item.item_uuid} justify="space-between" wrap="nowrap" gap="sm">
              <Group gap="sm" wrap="nowrap" style={{ minWidth: 0 }}>
                <Text fw={600} size="sm">{item.word}</Text>
                <Text size="sm" c="dimmed" style={{ minWidth: 0 }} lineClamp={1}>
                  {item.definition}
                </Text>
              </Group>
              <Button variant="subtle" color="danger" size="xs" onClick={() => c.remove(item.item_uuid)}>
                {t.delete}
              </Button>
            </Group>
          ))}
        </Stack>
      )}
    </Stack>
  );
}

export default VocabularyView;
