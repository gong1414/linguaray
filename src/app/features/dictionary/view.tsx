import {
  Alert,
  Badge,
  Button,
  Divider,
  Group,
  Paper,
  Stack,
  Text,
  TextInput,
  Title,
} from "@mantine/core";
import { BookOpen } from "lucide-react";
import { detectLocale } from "../../i18n";
import { DICTIONARY_COPY } from "./copy";
import type { DictionaryController } from "./controller";

/** Pure presentational offline Dictionary page. */
export function DictionaryView({ c }: { c: DictionaryController }) {
  const t = DICTIONARY_COPY[detectLocale()];

  return (
    <Stack gap="md" aria-label={t.title} data-testid="dictionary-page">
      <Title order={3}>{t.title}</Title>

      <Group gap="sm" align="flex-end" wrap="nowrap">
        <TextInput
          label={t.word}
          value={c.word}
          aria-label={t.word}
          style={{ flex: 1 }}
          onChange={(e) => c.setWord(e.currentTarget.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") c.lookup();
          }}
        />
        <Button onClick={c.lookup} disabled={!c.word.trim()}>
          {t.lookup}
        </Button>
      </Group>

      {c.result && (
        <Paper withBorder p="sm" data-testid="dictionary-result">
          <Text size="sm">{c.result.definition}</Text>
          <Text size="xs" c="dimmed" mt={4}>
            {t.source.replace("{source}", c.result.source)}
          </Text>
        </Paper>
      )}
      {c.miss && (
        <Text size="sm" c="dimmed" data-testid="dictionary-miss">
          {t.noResult}
        </Text>
      )}

      <Divider />

      <Stack gap="xs">
        <Text fw={500} size="sm">{t.install}</Text>
        <Group gap="sm" align="flex-end" wrap="wrap">
          <TextInput
            label={t.sourceDir}
            value={c.sourceDir}
            aria-label={t.sourceDir}
            style={{ flex: "1 1 14rem" }}
            onChange={(e) => c.setSourceDir(e.currentTarget.value)}
          />
          <TextInput
            label={t.packageId}
            value={c.packageId}
            aria-label={t.packageId}
            w={140}
            onChange={(e) => c.setPackageId(e.currentTarget.value)}
          />
          <TextInput
            label={t.packageName}
            value={c.packageName}
            aria-label={t.packageName}
            w={140}
            onChange={(e) => c.setPackageName(e.currentTarget.value)}
          />
          <TextInput
            label={t.version}
            value={c.version}
            aria-label={t.version}
            w={90}
            onChange={(e) => c.setVersion(e.currentTarget.value)}
          />
          <Button
            size="sm"
            variant="light"
            loading={c.installing}
            disabled={!c.sourceDir.trim() || !c.packageId.trim()}
            onClick={c.install}
          >
            {t.install}
          </Button>
        </Group>
      </Stack>

      {c.packages.length === 0 ? (
        <Stack align="center" gap={4} py="lg" data-testid="dictionary-no-packages">
          <BookOpen size={26} aria-hidden />
          <Text size="sm" c="dimmed">{t.noPackages}</Text>
        </Stack>
      ) : (
        <Group gap="xs">
          {c.packages.map((p) => (
            <Badge key={p.package_id} variant="light">{p.name}</Badge>
          ))}
        </Group>
      )}

      {c.notice && (
        <Alert role="status" data-testid="dictionary-notice">{c.notice}</Alert>
      )}
      {c.error && (
        <Alert color="red" role="alert" data-testid="dictionary-error">{c.error}</Alert>
      )}
    </Stack>
  );
}

export default DictionaryView;
