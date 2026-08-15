import {
  Alert,
  Badge,
  Button,
  Code,
  Group,
  Loader,
  Modal,
  Paper,
  Select,
  Stack,
  Switch,
  Text,
  Title,
} from "@mantine/core";
import { PRIVACY_COPY } from "./copy";
import type { PrivacyToast } from "./controller";
import type { ExternalApiStatus, HistoryPrivacyStatus, HistoryRetentionDays, PrivacyBusy } from "./model";

export type PrivacyViewProps = {
  locale: "zh" | "en";
  status: HistoryPrivacyStatus | null;
  loading: boolean;
  error: string | null;
  busy: PrivacyBusy;
  clearOpen: boolean;
  toasts: PrivacyToast[];
  external: ExternalApiStatus | null;
  externalBusy: boolean;
  tokenOnce: string | null;
  tokenCopied: boolean;
  onRetry: () => void;
  onEnabledChange: (enabled: boolean) => void;
  onRetentionChange: (days: HistoryRetentionDays) => void;
  onOpenClear: () => void;
  onCloseClear: () => void;
  onConfirmClear: () => void;
  onEnableExternal: () => void;
  onDisableExternal: () => void;
  onRegenToken: () => void;
  onCopyToken: () => void;
  onDismissToast: (id: number) => void;
};

/**
 * Pure presentational Privacy & Data page. The External API section is part
 * of the SAME formal page structure (migration spec §六.Privacy) — no bare
 * controls bolted on outside the reviewed composition.
 */
export function PrivacyView(props: PrivacyViewProps) {
  const t = PRIVACY_COPY[props.locale];
  const countCopy = t.records.replace("{count}", String(props.status?.record_count ?? 0));
  const externalLine =
    props.external?.state === "enabled"
      ? t.externalOn.replace("{port}", String(props.external.port ?? ""))
      : t.externalOff;

  return (
    <Stack gap="md" aria-label={t.title} aria-busy={props.loading || undefined} data-testid="privacy-page">
      <Title order={3}>{t.title}</Title>

      {props.error && (
        <Alert color="red" title={t.loadFailed} data-testid="privacy-error">
          <Group justify="space-between" wrap="nowrap" gap="sm">
            <Text size="sm" style={{ flex: 1 }}>
              {props.error}
            </Text>
            <Button variant="light" color="red" size="xs" onClick={props.onRetry}>
              {t.retry}
            </Button>
          </Group>
        </Alert>
      )}

      {props.loading && (
        <Group gap="sm" data-testid="privacy-loading">
          <Loader size="sm" />
          <Text size="sm" c="dimmed">
            {t.loading}
          </Text>
        </Group>
      )}

      {!props.error && !props.loading && props.status && (
        <>
          <Paper withBorder p="md" data-testid="history-panel">
            <Group justify="space-between" align="flex-start" wrap="nowrap">
              <div>
                <Title order={4}>{t.historyTitle}</Title>
                <Text size="sm" c="dimmed">
                  {props.status.enabled ? t.historyEnabledNotice : t.historyDisabledNotice}
                </Text>
              </div>
              <Switch
                label={t.historyEnable}
                checked={props.status.enabled}
                disabled={props.busy !== null}
                onChange={(e) => props.onEnabledChange(e.currentTarget.checked)}
              />
            </Group>
            <Group justify="space-between" mt="md" wrap="nowrap">
              <Select
                label={t.retention}
                w={160}
                value={String(props.status.retention_days)}
                data={[
                  { value: "30", label: t.retention30 },
                  { value: "90", label: t.retention90 },
                ]}
                disabled={!props.status.enabled || props.busy !== null}
                onChange={(v) => props.onRetentionChange(Number(v) as HistoryRetentionDays)}
              />
              <Group gap="sm" wrap="nowrap">
                <Badge variant="light" color={props.status.record_count > 0 ? "brand" : "gray"}>
                  {countCopy}
                </Badge>
                <Button
                  color="danger"
                  disabled={props.status.record_count === 0 || props.busy !== null}
                  loading={props.busy === "clear"}
                  onClick={props.onOpenClear}
                >
                  {t.clearAll}
                </Button>
              </Group>
            </Group>
          </Paper>

          <Paper withBorder p="md" data-testid="external-panel">
            <Group justify="space-between" align="flex-start" wrap="nowrap">
              <div>
                <Title order={4}>{t.externalTitle}</Title>
                <Text size="sm" c="dimmed">
                  {t.externalHint}
                </Text>
              </div>
              <Badge variant="light" color={props.external?.state === "enabled" ? "success" : "gray"}>
                {externalLine}
              </Badge>
            </Group>
            <Group mt="md">
              <Button
                size="xs"
                loading={props.externalBusy && props.external?.state !== "enabled"}
                disabled={props.external?.state === "enabled" || props.externalBusy}
                onClick={props.onEnableExternal}
              >
                {t.externalEnable}
              </Button>
              <Button
                size="xs"
                variant="light"
                disabled={props.external?.state !== "enabled" || props.externalBusy}
                loading={props.externalBusy && props.external?.state === "enabled"}
                onClick={props.onDisableExternal}
              >
                {t.externalDisable}
              </Button>
              <Button
                size="xs"
                variant="light"
                disabled={props.external?.state !== "enabled" || props.externalBusy}
                onClick={props.onRegenToken}
              >
                {t.externalRegen}
              </Button>
            </Group>
            {props.tokenOnce && (
              <Alert color="warning" mt="md" title={t.externalTokenOnce} data-testid="external-token">
                <Stack gap="xs">
                  <Text size="sm">{t.externalTokenHint}</Text>
                  <Group gap="xs">
                    <Code block>{props.tokenOnce}</Code>
                    <Button size="xs" variant="light" onClick={props.onCopyToken}>
                      {props.tokenCopied ? t.copied : t.copyToken}
                    </Button>
                  </Group>
                </Stack>
              </Alert>
            )}
          </Paper>
        </>
      )}

      <Modal
        opened={props.clearOpen}
        onClose={props.onCloseClear}
        title={t.clearConfirmTitle}
        centered
        data-testid="privacy-clear-modal"
      >
        <Text size="sm">{t.clearConfirmMessage}</Text>
        <Group justify="flex-end" mt="md">
          <Button variant="subtle" onClick={props.onCloseClear}>
            {t.cancel}
          </Button>
          <Button color="danger" onClick={props.onConfirmClear}>
            {t.clearAll}
          </Button>
        </Group>
      </Modal>

      {props.toasts.length > 0 && (
        <Stack gap="xs" aria-live="polite" data-testid="privacy-toasts">
          {props.toasts.map((entry) => (
            <Alert
              key={entry.id}
              color={entry.variant === "success" ? "green" : "red"}
              withCloseButton
              closeButtonLabel={t.dismissToast}
              onClose={() => props.onDismissToast(entry.id)}
            >
              {entry.message}
            </Alert>
          ))}
        </Stack>
      )}
    </Stack>
  );
}

export default PrivacyView;
