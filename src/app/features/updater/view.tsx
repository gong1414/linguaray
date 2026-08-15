import {
  Alert,
  Button,
  Checkbox,
  Divider,
  Group,
  Paper,
  Progress,
  Stack,
  Text,
  Title,
} from "@mantine/core";
import { detectLocale } from "../../i18n";
import { UPDATER_COPY } from "./copy";
import type { UpdaterController } from "./controller";
import type { AvailableUpdate } from "./model";

/** Pure presentational updater panel driven by the phase machine. */
export function UpdaterPanelView({ c }: { c: UpdaterController }) {
  const t = UPDATER_COPY[detectLocale()];
  const phase = c.phase;

  const availableUpdate: AvailableUpdate | null =
    phase.kind === "available" ||
    phase.kind === "downloading" ||
    phase.kind === "installing" ||
    phase.kind === "readyToRelaunch"
      ? phase.update
      : null;
  const knownVersion =
    phase.kind === "upToDate" ? phase.version : availableUpdate?.current ?? null;
  const percent = phase.kind === "downloading" ? phase.percent : null;
  const busy = phase.kind === "downloading" || phase.kind === "installing";

  return (
    <Stack gap="md" data-testid="updater-panel" aria-label={t.title}>
      <Group justify="space-between" wrap="wrap">
        <Title order={3}>{t.title}</Title>
        {knownVersion && (
          <Text size="sm" c="dimmed" data-testid="updater-current-version">
            {t.currentVersion}: {knownVersion}
          </Text>
        )}
      </Group>

      <Paper withBorder p="md" data-testid="updater-status">
        {phase.kind === "checking" && <Text size="sm" c="dimmed">{t.status.checking}</Text>}
        {phase.kind === "upToDate" && <Text size="sm">{t.status.upToDate}</Text>}
        {phase.kind === "error" && (
          <Alert color="red" data-testid="updater-error" title={t.status.errorPrefix}>
            {phase.message}
          </Alert>
        )}

        {availableUpdate && (
          <Stack gap="xs">
            <Text fw={600} size="sm" data-testid="updater-next">
              {t.status.available} {availableUpdate.next}
            </Text>
            {availableUpdate.notes && (
              <details>
                <summary style={{ fontSize: "var(--mantine-font-size-sm)" }}>{t.releaseNotes}</summary>
                <Text component="pre" size="xs" c="dimmed" style={{ whiteSpace: "pre-wrap" }}>
                  {availableUpdate.notes}
                </Text>
              </details>
            )}
          </Stack>
        )}

        {phase.kind === "downloading" && (
          <Stack gap={4} mt="sm" data-testid="updater-progress">
            <Text size="sm">
              {percent !== null ? `${t.progress.downloading} ${percent}%` : t.progress.unknownSize}
            </Text>
            {percent !== null ? (
              <Progress value={percent} size="sm" />
            ) : (
              <Progress value={100} size="sm" animated />
            )}
          </Stack>
        )}
        {phase.kind === "installing" && (
          <Text size="sm" mt="sm" data-testid="updater-installing">
            {t.progress.installing}
          </Text>
        )}
        {phase.kind === "readyToRelaunch" && (
          <Group gap="sm" mt="sm">
            <Text size="sm">{t.progress.installedHint}</Text>
            <Button size="xs" data-testid="updater-relaunch" onClick={c.relaunch}>
              {t.action.relaunch}
            </Button>
          </Group>
        )}

        <Group gap="sm" mt="md">
          <Button variant="light" size="xs" disabled={busy} onClick={c.check} data-testid="updater-check-again">
            {t.action.checkAgain}
          </Button>
          {phase.kind === "available" && (
            <Button size="xs" onClick={c.install} data-testid="updater-download">
              {t.action.downloadInstall}
            </Button>
          )}
        </Group>
      </Paper>

      <Divider />

      <Checkbox
        checked={c.autoCheck}
        label={
          <div>
            <Text span fw={600} size="sm">{t.autoCheckLabel}</Text>
            <br />
            <Text span size="xs" c="dimmed">{t.autoCheckHint}</Text>
          </div>
        }
        data-testid="updater-autocheck"
        onChange={(e) => c.toggleAutoCheck(e.currentTarget.checked)}
      />
      {c.autoCheckError && (
        <Alert color="red" role="alert" data-testid="updater-autocheck-error">
          {c.autoCheckError}
        </Alert>
      )}
    </Stack>
  );
}

export default UpdaterPanelView;
