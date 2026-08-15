import { Alert, Button, Group, Modal, Stack, Text } from "@mantine/core";
import { detectLocale } from "../../i18n";
import { KEYSTORE_COPY } from "./copy";
import type { KeystoreController } from "./controller";

/**
 * Pure presentational Keystore Recovery surface: corrupt/archived banner +
 * destructive reset Confirm (Cancel-focused) + toast stack.
 */
export function KeystoreRecoveryView({ c }: { c: KeystoreController }) {
  const t = KEYSTORE_COPY[detectLocale()];

  return (
    <section aria-label={t.title} data-testid="keystore-recovery">
      {c.state === "corrupt" && (
        <Alert color="red" title={t.title} data-testid="keystore-corrupt">
          <Text size="sm" mb="xs">{t.description.replace("{reason}", c.reason)}</Text>
          <Group gap="xs">
            <Button size="xs" loading={c.busy === "archive"} onClick={c.archive}>
              {t.archive}
            </Button>
            <Button
              color="danger"
              size="xs"
              onClick={c.openReset}
              data-testid="keystore-reset-trigger"
            >
              {t.reset}
            </Button>
          </Group>
        </Alert>
      )}

      {c.state === "archived" && (
        <Alert color="blue" title={t.archivedTitle} data-testid="keystore-archived">
          {t.archivedPrompt}
        </Alert>
      )}

      <Modal
        opened={c.resetOpen}
        onClose={c.closeReset}
        title={t.resetConfirmTitle}
        centered
        data-testid="keystore-reset-modal"
      >
        <Text size="sm">{t.resetConfirmMessage}</Text>
        <Group justify="flex-end" mt="md">
          <Button variant="subtle" onClick={c.closeReset}>
            {t.resetConfirmCancelLabel}
          </Button>
          <Button
            color="danger"
            size="xs"
            loading={c.busy === "reset"}
            onClick={c.reset}
            data-testid="keystore-reset-confirm"
          >
            {t.resetConfirmConfirmLabel}
          </Button>
        </Group>
      </Modal>

      {c.toasts.length > 0 && (
        <Stack gap="xs" mt="sm" aria-live="polite" data-testid="keystore-toasts">
          {c.toasts.map((entry) => (
            <Alert
              key={entry.id}
              color={
                entry.variant === "destructive" ? "red"
                : entry.variant === "warning" ? "yellow"
                : entry.variant === "success" ? "green"
                : "blue"
              }
              withCloseButton
              closeButtonLabel="Dismiss"
              onClose={() => c.dismissToast(entry.id)}
            >
              {entry.message}
            </Alert>
          ))}
        </Stack>
      )}
    </section>
  );
}

export default KeystoreRecoveryView;
