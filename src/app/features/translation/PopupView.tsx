import { useEffect, useRef, useState } from "react";
import {
  ActionIcon,
  Badge,
  Button,
  Group,
  Paper,
  Stack,
  Text,
  Tooltip,
} from "@mantine/core";
import { Copy, Pin, PinOff, Star, Volume2 } from "lucide-react";
import { t } from "../../i18n";
import { writeText } from "../../../bridge/clipboard";
import { addVocabulary } from "./input-ipc";
import { openSettingsWindow, ttsSpeak, ttsStop } from "./popup-ipc";
import type { PopupController } from "./popupController";
import type { TranslationState } from "./types";

/** Map a state kind onto the localized headline (aria-label + error shell). */
export function headlineFor(s: TranslationState): string {
  switch (s.kind) {
    case "loading":
      return t("selection.loading");
    case "single-success":
    case "multi-success":
    case "partial":
      return t("selection.multi.title");
    case "error":
      switch (s.sub) {
        case "network":
          return t("selection.error.network");
        case "config-key":
          return t("selection.error.config.key");
        case "config-401":
          return t("selection.error.config.auth");
        default:
          return s.message;
      }
    case "offline":
      return t("selection.error.offline");
    case "no-selection":
      return t("selection.error.noSelection");
    case "no-permission":
      return t("selection.error.noPermission");
    case "keystore-corrupt":
      return t("selection.error.keystore");
  }
}

const COPIED_FEEDBACK_MS = 1200;

/** Card actions shared by the single and multi result shells. */
function CardActions({
  c,
  text,
  copied,
  favorited,
  speaking,
  onCopied,
  onFavorited,
  onSpeaking,
}: {
  c: PopupController;
  text: string;
  copied: boolean;
  favorited: boolean;
  speaking: boolean;
  onCopied: () => void;
  onFavorited: () => void;
  onSpeaking: (on: boolean) => void;
}) {
  return (
    <Group gap={4} wrap="nowrap">
      <Tooltip label={copied ? t("selection.action.copied") : t("selection.action.copy")}>
        <ActionIcon
          variant={copied ? "filled" : "light"}
          size="sm"
          aria-label={copied ? t("selection.action.copied") : t("selection.action.copy")}
          onClick={() => {
            void writeText(text)
              .then(onCopied)
              .catch(() => {});
          }}
        >
          {copied ? <span style={{ fontSize: 10 }}>{t("selection.action.copied")}</span> : <Copy size={14} aria-hidden />}
        </ActionIcon>
      </Tooltip>
      <Tooltip label={speaking ? t("selection.action.stop") : t("selection.action.speak")}>
        <ActionIcon
          variant={speaking ? "filled" : "light"}
          color={speaking ? "brand" : "gray"}
          size="sm"
          aria-label={speaking ? t("selection.action.stop") : t("selection.action.speak")}
          onClick={() => {
            if (speaking) {
              void ttsStop().finally(() => onSpeaking(false));
              return;
            }
            void ttsSpeak(text)
              .then(() => onSpeaking(true))
              .catch(() => onSpeaking(false));
          }}
        >
          <Volume2 size={14} aria-hidden />
        </ActionIcon>
      </Tooltip>
      <Tooltip label={c.pinned ? t("selection.action.unpin") : t("selection.action.pin")}>
        <ActionIcon
          variant={c.pinned ? "filled" : "light"}
          color={c.pinned ? "brand" : "gray"}
          size="sm"
          aria-label={c.pinned ? t("selection.action.unpin") : t("selection.action.pin")}
          onClick={() => (c.pinned ? c.unpin() : c.pin())}
        >
          {c.pinned ? <PinOff size={14} aria-hidden /> : <Pin size={14} aria-hidden />}
        </ActionIcon>
      </Tooltip>
      <Tooltip label={favorited ? t("selection.action.favorited") : t("selection.action.favorite")}>
        <ActionIcon
          variant={favorited ? "filled" : "light"}
          color={favorited ? "warning" : "gray"}
          size="sm"
          aria-label={favorited ? t("selection.action.favorited") : t("selection.action.favorite")}
          onClick={() => {
            const source = c.lastSource || text;
            void addVocabulary(source, text, "zh")
              .then(onFavorited)
              .catch(() => {});
          }}
        >
          <Star size={14} fill={favorited ? "currentColor" : "none"} aria-hidden />
        </ActionIcon>
      </Tooltip>
    </Group>
  );
}

/** Pure presentational selection popup. */
export function PopupView({ c }: { c: PopupController }) {
  const single = c.state.kind === "single-success" ? c.state : null;
  const multi =
    c.state.kind === "multi-success" || c.state.kind === "partial" ? c.state.results : null;
  const errorState = c.state.kind === "error" ? c.state : null;
  const isErrorShell =
    c.state.kind === "error" ||
    c.state.kind === "offline" ||
    c.state.kind === "no-selection" ||
    c.state.kind === "no-permission";

  const [copiedKey, setCopiedKey] = useState<string | null>(null);
  const [favoritedKey, setFavoritedKey] = useState<string | null>(null);
  const [speakingKey, setSpeakingKey] = useState<string | null>(null);
  const copiedTimer = useRef<ReturnType<typeof setTimeout> | undefined>(undefined);
  useEffect(() => () => copiedTimer.current && clearTimeout(copiedTimer.current), []);

  const markCopied = (uuid: string) => {
    setCopiedKey(uuid);
    if (copiedTimer.current) clearTimeout(copiedTimer.current);
    copiedTimer.current = setTimeout(() => setCopiedKey(null), COPIED_FEEDBACK_MS);
  };

  return (
    <section
      aria-label={headlineFor(c.state)}
      aria-busy={c.state.kind === "loading" || undefined}
      data-testid="popup-shell"
      tabIndex={-1}
      onKeyDown={(e) => {
        if (e.key === "Escape") {
          e.preventDefault();
          c.dismiss();
        }
      }}
      style={{ outline: "none" }}
    >
      {c.state.kind === "loading" && (
        <Group gap="xs" px="md" py="sm" data-testid="popup-loading">
          <span aria-hidden>…</span>
          <Text size="sm" c="dimmed">{t("selection.loading")}</Text>
          {c.hasSource && (
            <Button variant="subtle" size="compact-sm" onClick={c.retrySelection}>
              {t("selection.action.retry")}
            </Button>
          )}
        </Group>
      )}

      {single && (
        <Paper p="sm" withBorder data-testid="popup-card">
          <Group justify="space-between" wrap="nowrap" align="flex-start">
            <Badge variant="light" color="brand">{c.engineLabel(single.engine)}</Badge>
            <CardActions
              c={c}
              text={single.text}
              copied={copiedKey === "__single__"}
              favorited={favoritedKey === "__single__"}
              speaking={speakingKey === "__single__"}
              onCopied={() => markCopied("__single__")}
              onFavorited={() => setFavoritedKey("__single__")}
              onSpeaking={(on) => setSpeakingKey(on ? "__single__" : null)}
            />
          </Group>
          <Text size="sm" mt="xs" style={{ whiteSpace: "pre-wrap" }}>{single.text}</Text>
        </Paper>
      )}

      {multi && (
        <Stack gap="xs" p="sm" data-multi="true">
          {multi.map((r) => (
            <Paper key={r.uuid} withBorder p="sm" data-testid="popup-card">
              <Group justify="space-between" wrap="nowrap" align="flex-start">
                <Badge variant="light" color={r.ok ? "brand" : "gray"}>{c.engineLabel(r.engine)}</Badge>
                {r.ok && r.text && (
                  <CardActions
                    c={c}
                    text={r.text}
                    copied={copiedKey === r.uuid}
                    favorited={favoritedKey === r.uuid}
                    speaking={speakingKey === r.uuid}
                    onCopied={() => markCopied(r.uuid)}
                    onFavorited={() => setFavoritedKey(r.uuid)}
                    onSpeaking={(on) => setSpeakingKey(on ? r.uuid : null)}
                  />
                )}
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

      {(single || multi) && c.hasSource && (
        <Group justify="flex-end" px="sm">
          <Button variant="subtle" size="compact-sm" onClick={c.retrySelection}>
            {t("selection.action.retry")}
          </Button>
        </Group>
      )}

      {isErrorShell && (
        <Stack align="center" gap="xs" py="md" px="md" role="alert" data-testid="popup-error">
          <span aria-hidden>⚠</span>
          <Text size="sm" fw={500}>{headlineFor(c.state)}</Text>
          {errorState?.sub === "network" && c.hasSource && (
            <Button variant="light" size="xs" onClick={c.retrySelection}>
              {t("selection.action.retry")}
            </Button>
          )}
          {(errorState?.sub === "config-key" || errorState?.sub === "config-401") && (
            <Button variant="subtle" size="xs" onClick={() => void openSettingsWindow("provider-center")}>
              {t("selection.action.openSettings")}
            </Button>
          )}
        </Stack>
      )}

      {c.state.kind === "keystore-corrupt" && (
        <Stack align="center" gap="xs" py="md" px="md" role="alert" data-testid="popup-keystore">
          <span aria-hidden>⚠</span>
          <Text size="sm" fw={500}>{t("selection.error.keystore")}</Text>
          <Button variant="subtle" size="xs" onClick={() => void openSettingsWindow("keystore-recovery")}>
            {t("selection.action.recovery")}
          </Button>
        </Stack>
      )}
    </section>
  );
}

export default PopupView;
