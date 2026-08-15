import {
  Alert,
  Badge,
  Button,
  Group,
  Kbd,
  Modal,
  Paper,
  Stack,
  Text,
  Title,
} from "@mantine/core";
import { detectLocale } from "../../i18n";
import { SHORTCUTS_COPY } from "./copy";
import { SHORTCUT_ACTIONS, DEFAULT_SHORTCUT_MAP, type ShortcutAction, type ShortcutSnapshot } from "./model";
import type { ShortcutsController } from "./controller";

const ICON_HINT: Record<ShortcutAction, string> = {
  translate_selection: "⌨",
  translate_input: "⌨",
  translate_clipboard: "⌨",
  ocr_translate: "⌨",
};

function entryFor(snapshot: ShortcutSnapshot, action: ShortcutAction) {
  return (
    snapshot.entries.find((entry) => entry.action === action) ?? {
      action,
      combo: DEFAULT_SHORTCUT_MAP[action],
      available: false,
      registration_state: "unavailable" as const,
      registration_error: null,
    }
  );
}

/** Pure presentational shortcuts page (Mantine Kbd chips + recorder button). */
export function ShortcutsView({ c }: { c: ShortcutsController }) {
  const t = SHORTCUTS_COPY[detectLocale()];

  return (
    <Stack gap="md" aria-label={t.title} data-testid="shortcuts-page">
      <Title order={3}>{t.title}</Title>

      {c.loadError && (
        <Alert color="red" title={t.loadFailed} data-testid="shortcuts-load-error">
          <Button size="xs" variant="light" color="red" onClick={c.retryLoad}>
            {t.retry}
          </Button>
        </Alert>
      )}

      {!c.loadError && !c.snapshot && (
        <Text size="sm" c="dimmed" role="status">
          {t.loading}
        </Text>
      )}

      {c.snapshot &&
        SHORTCUT_ACTIONS.map((action) => {
          const entry = entryFor(c.snapshot!, action);
          const isRecording = c.recordingAction === action;
          const conflict = c.conflict?.action === action ? c.conflict : null;
          const registrationFailed =
            c.localFailures[action] || entry.registration_state === "registration_failed";
          const changeLabel = t.changeLabel.replace("{action}", t.actions[action]);

          return (
            <Paper key={action} withBorder p="sm" data-action={action}>
              <Group justify="space-between" wrap="nowrap">
                <Group gap="xs" wrap="nowrap">
                  <span aria-hidden style={{ fontSize: 16 }}>{ICON_HINT[action]}</span>
                  <Stack gap={0}>
                    <Text size="sm" fw={500}>{t.actions[action]}</Text>
                    {!entry.available && (
                      <Text size="xs" c="dimmed">{t.unavailable}</Text>
                    )}
                  </Stack>
                </Group>
                <Group gap="xs" wrap="nowrap">
                  {isRecording ? (
                    <>
                      <Button
                        variant="light"
                        size="xs"
                        aria-label={t.recordingPrompt}
                        data-recorder-action={action}
                        onKeyDown={c.onRecorderKeyDown}
                      >
                        <Kbd>{c.recordedCombo || "…"}</Kbd>
                      </Button>
                      <Button variant="subtle" size="compact-sm" onClick={c.cancelRecording}>
                        {t.cancel}
                      </Button>
                    </>
                  ) : (
                    <>
                      <Badge variant="light" size="lg" data-testid={`shortcut-chip-${action}`}>
                        <Kbd style={{ fontFamily: "var(--mantine-font-family-monospace)" }}>
                          {entry.combo}
                        </Kbd>
                      </Badge>
                      <Button
                        variant="subtle"
                        size="compact-sm"
                        aria-label={changeLabel}
                        data-change-action={action}
                        data-testid={`shortcuts-change-${action}`}
                        disabled={!entry.available || c.busy !== null}
                        onClick={() => c.change(action)}
                      >
                        {t.change}
                      </Button>
                    </>
                  )}
                </Group>
              </Group>

              {conflict && (
                <Group gap="xs" mt="xs" data-testid={`shortcut-conflict-${action}`}>
                  <Text size="sm" c="red">
                    {t.conflictMessage.replace("{action}", t.actions[conflict.otherAction])}
                  </Text>
                  <Button
                    size="xs"
                    variant="light"
                    color="red"
                    loading={c.busy === "save"}
                    onClick={c.overrideConflict}
                  >
                    {t.override}
                  </Button>
                  <Button size="xs" variant="subtle" onClick={c.cancelRecording}>
                    {t.cancel}
                  </Button>
                </Group>
              )}

              {registrationFailed && (
                <Text size="xs" c="orange.7" mt={4} data-testid={`shortcut-regfail-${action}`}>
                  {t.registrationFailed}
                </Text>
              )}
            </Paper>
          );
        })}

      {c.operationError && (
        <Alert color="red" data-testid="shortcuts-operation-error">
          {c.operationError === "reset" ? t.resetFailed : t.saveFailed}
        </Alert>
      )}

      {c.snapshot && (
        <Group justify="flex-end">
          <Button
            variant="light"
            disabled={!c.differsFromDefaults || c.busy !== null}
            onClick={c.openReset}
            data-testid="shortcuts-reset-trigger"
          >
            {t.resetDefaults}
          </Button>
        </Group>
      )}

      <Modal
        opened={c.resetOpen}
        onClose={c.closeReset}
        title={t.resetConfirmTitle}
        centered
        data-testid="shortcuts-reset-modal"
      >
        <Text size="sm">{t.resetConfirmMessage}</Text>
        <Group justify="flex-end" mt="md">
          <Button variant="subtle" onClick={c.closeReset}>{t.cancel}</Button>
          <Button color="danger" onClick={c.reset} data-testid="shortcuts-reset-confirm">
            {t.useDefaults}
          </Button>
        </Group>
      </Modal>
    </Stack>
  );
}

export default ShortcutsView;
