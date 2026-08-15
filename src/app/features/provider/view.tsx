import { Alert, Button, Group, Modal, Stack, Text } from "@mantine/core";
import { detectLocale } from "../../i18n";
import { PROVIDER_COPY } from "./copy";
import { ProviderList } from "./parts/ProviderList";
import ProviderDetail, { DetailEmpty } from "./parts/ProviderDetail";
import type { ProviderController } from "./controller";
import type { ConsentRecipient, ToastEntry } from "./model";
import classes from "./provider.module.css";

/** Pure presentational Provider Center (controller props in, callbacks out). */
export function ProviderCenterView({ c }: { c: ProviderController }) {
  const t = PROVIDER_COPY[detectLocale()];
  const locked = c.exclusiveBusy;

  const toasts: ToastEntry[] = c.toasts;
  const recipients: ConsentRecipient[] = c.consentRecipients;

  return (
    <div className={classes.body} role="region" aria-label={t.providerListLabel}>
      {c.loadError && (
        <Alert color="red" mb="sm" title={t.loadFailed}>
          <Button size="xs" variant="light" color="red" disabled={locked} onClick={c.onReloadFromError}>
            {t.reload}
          </Button>
        </Alert>
      )}
      {c.selectionError && (
        <Alert color="red" mb="sm" data-testid="selection-error">
          <Group gap="sm">
            <Text size="sm">{t.loadFailed}</Text>
            <Button size="xs" variant="light" color="red" disabled={locked} onClick={c.onRetrySelectionLoad}>
              {t.retry}
            </Button>
          </Group>
        </Alert>
      )}

      <div className={classes.layout}>
        <ProviderList
          t={t}
          providers={c.providers}
          selectedUuid={c.selectedUuid}
          exclusiveBusy={c.exclusiveBusy}
          deletingUuid={c.deletingUuid}
          presets={c.presets}
          roleFor={c.roleFor}
          onToggle={c.onToggle}
          onEdit={c.select}
          onDelete={c.onDelete}
          onSetPrimary={c.onSetPrimary}
          onAddParallel={c.onAddParallel}
          onRemoveParallel={c.onRemoveParallel}
          onSetFallback={c.onSetFallback}
          onDuplicate={c.onDuplicate}
          onMoveUp={c.onMoveUp}
          onMoveDown={c.onMoveDown}
          onAddPreset={c.onAddPreset}
        />
        <section aria-label={t.detailLabel}>
          {c.detail ? (
            <ProviderDetail
              t={t}
              detail={c.detail}
              reloading={c.reloadingUuid === c.detail.provider.uuid}
              exclusiveBusy={c.exclusiveBusy}
              balanceText={c.balanceByUuid[c.detail.provider.uuid]}
              onNameInput={c.onNameInput}
              onEndpointInput={c.onEndpointInput}
              onModelInput={c.onModelInput}
              onModelChange={c.onModelChange}
              onKeyInput={c.onKeyInput}
              onSaveProfile={c.onSaveProfile}
              onToggleCustomAnthropic={c.onToggleCustomAnthropic}
              onSaveKey={c.onSaveKey}
              onFetchModels={c.onFetchModels}
              onTestConnection={c.onTestConnection}
              onFetchBalance={c.onFetchBalance}
              onResolveSaveConflict={c.onResolveSaveConflict}
            />
          ) : (
            <DetailEmpty t={t} />
          )}
        </section>
      </div>

      {/* Delete confirm */}
      <Modal
        opened={c.deleteConfirmUuid !== null}
        onClose={c.onCancelDelete}
        title={t.deleteConfirmTitle}
        centered
        data-testid="delete-confirm"
      >
        <Text size="sm">{t.deleteConfirmMsg}</Text>
        <Group justify="flex-end" mt="md">
          <Button variant="subtle" onClick={c.onCancelDelete}>{t.cancel}</Button>
          <Button color="danger" loading={c.deletingUuid !== null} onClick={c.onConfirmDelete}>
            {t.delete}
          </Button>
        </Group>
      </Modal>

      {/* Delete-error retry banner (outside the modal so clicks aren't swallowed) */}
      {c.deleteError && (
        <Alert color="red" mt="sm" data-testid="delete-error" title={t.saveFailed}>
          <Group gap="sm">
            <Button size="xs" variant="light" color="red" disabled={locked} onClick={c.onRetryDelete}>
              {t.retry}
            </Button>
            <Button size="xs" variant="subtle" onClick={c.onDismissDeleteError}>{t.cancel}</Button>
          </Group>
        </Alert>
      )}

      {/* Consent confirm */}
      <Modal
        opened={c.consentOpen}
        onClose={c.onCancelConsent}
        title={t.consent.title}
        centered
        data-testid="consent-modal"
      >
        <Stack gap="sm">
          <Text size="sm">{t.consent.message}</Text>
          <ul style={{ margin: 0, paddingLeft: 18 }}>
            {recipients.map((r) => (
              <li key={r.name}>
                <Text span fw={600}>{r.name}</Text>{" "}
                <Text span size="sm" c="dimmed">{r.localLabel}</Text>
              </li>
            ))}
          </ul>
          <Group justify="flex-end">
            <Button variant="subtle" onClick={c.onCancelConsent}>{t.consent.cancel}</Button>
            <Button onClick={c.onConfirmConsent}>{t.consent.confirm}</Button>
          </Group>
        </Stack>
      </Modal>

      {/* Toasts */}
      {toasts.length > 0 && (
        <Stack gap="xs" mt="sm" aria-live="polite" data-testid="provider-toasts">
          {toasts.map((toast) => (
            <Alert
              key={toast.id}
              color={
                toast.variant === "success" ? "green"
                : toast.variant === "warning" ? "yellow"
                : toast.variant === "destructive" ? "red"
                : "blue"
              }
              withCloseButton
              closeButtonLabel={t.toastDismiss}
              onClose={() => c.onDismissToast(toast.id)}
            >
              {toast.message}
            </Alert>
          ))}
        </Stack>
      )}
    </div>
  );
}

export default ProviderCenterView;
