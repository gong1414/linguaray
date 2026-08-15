import {
  ActionIcon,
  Alert,
  Badge,
  Button,
  Group,
  Paper,
  Stack,
  Textarea,
  Text,
  Title,
} from "@mantine/core";
import { Star } from "lucide-react";
import { t } from "../../i18n";
import { engineLabel } from "./providerNames";
import type { InputController } from "./inputController";
import type { TranslationState } from "./types";

/** Map the state union onto the localized headline (mirrors the Solid view). */
export function errorMessageFor(state: TranslationState): string | null {
  if (state.kind === "error") {
    return state.sub === "network"
      ? t("selection.error.network")
      : state.sub === "config-key"
        ? t("selection.error.config.key")
        : state.sub === "config-401"
          ? t("selection.error.config.auth")
          : state.message;
  }
  if (state.kind === "offline") return t("input.error.offline");
  if (state.kind === "no-permission") return t("selection.error.noPermission");
  if (state.kind === "keystore-corrupt") return t("selection.error.keystore");
  return null;
}

/** Pure presentational input window (props/callbacks only). */
export function InputPanelView({ c }: { c: InputController }) {
  const single = c.state.kind === "single-success" ? c.state : null;
  const multi =
    c.state.kind === "multi-success" || c.state.kind === "partial" ? c.state.results : null;
  const errorMessage = errorMessageFor(c.state);
  // Clear stays enabled with typed-but-untranslated text (legacy P1-5 rule).
  const showClear = c.hasResult || c.text.trim().length > 0;

  const favoriteButton = (key: string, translation: string) => {
    const favorited = c.favoritedKey === key;
    return (
      <ActionIcon
        variant={favorited ? "filled" : "light"}
        color={favorited ? "warning" : "gray"}
        size="sm"
        aria-label={favorited ? t("selection.action.favorited") : t("selection.action.favorite")}
        onClick={() => void c.favorite(c.text, translation, key)}
      >
        <Star size={14} fill={favorited ? "currentColor" : "none"} aria-hidden />
      </ActionIcon>
    );
  };

  return (
    <Stack gap="sm" p="md" data-testid="input-panel">
      <Title order={4}>{t("input.title")}</Title>
      <Textarea
        ref={c.textareaRef}
        aria-label={t("input.title")}
        placeholder={t("input.placeholder")}
        rows={4}
        value={c.text}
        disabled={!c.idle}
        onChange={(e) => c.setText(e.currentTarget.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter" && !e.shiftKey) {
            e.preventDefault();
            void c.translate();
          }
        }}
      />
      <Group gap="sm">
        <Button variant="light" onClick={c.clear} disabled={!showClear}>
          {t("input.action.clear")}
        </Button>
        <Button
          loading={!c.idle}
          loaderProps={{ "aria-label": t("selection.loading") }}
          onClick={() => void c.translate()}
          disabled={!c.text.trim()}
        >
          {t("input.action.translate")}
        </Button>
      </Group>

      {single && (
        <Paper withBorder p="sm" data-testid="input-result">
          <Group justify="space-between" wrap="nowrap" align="flex-start">
            <Badge variant="light" color="brand">{engineLabel(single.engine)}</Badge>
            {favoriteButton("__single__", single.text)}
          </Group>
          <Text size="sm" mt="xs" style={{ whiteSpace: "pre-wrap" }}>{single.text}</Text>
        </Paper>
      )}

      {multi && (
        <Stack gap="xs" data-multi="true">
          {multi.map((r) => (
            <Paper key={r.uuid} withBorder p="sm" data-testid="input-result">
              <Group justify="space-between" wrap="nowrap" align="flex-start">
                <Badge variant="light" color={r.ok ? "brand" : "gray"}>{engineLabel(r.engine)}</Badge>
                {r.ok && r.text && favoriteButton(r.uuid, r.text)}
              </Group>
              {r.ok ? (
                <Text size="sm" mt="xs" style={{ whiteSpace: "pre-wrap" }}>{r.text}</Text>
              ) : (
                <Text size="sm" c="red" mt="xs">{r.errorText}</Text>
              )}
            </Paper>
          ))}
        </Stack>
      )}

      {errorMessage && (
        <Alert color="red" icon={<span aria-hidden>⚠</span>} data-testid="input-error">
          {errorMessage}
        </Alert>
      )}
    </Stack>
  );
}

export default InputPanelView;
