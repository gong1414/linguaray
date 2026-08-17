import { Alert, Button, Modal, Spin, Typography } from "antd";
import { CloseOutlined } from "@ant-design/icons";
import { detectLocale } from "../../app/i18n";
import { useUiStyles } from "../../ui/styles";
import { PROVIDER_COPY } from "./copy";
import { ProviderList } from "./parts/ProviderList";
import ProviderDetail, { DetailEmpty } from "./parts/ProviderDetail";
import type { ProviderController } from "./controller";
import type { ConsentRecipient, ToastEntry } from "./model";

/** Pure presentational Provider Center (controller props in, callbacks out). */
export function ProviderCenterView({ c }: { c: ProviderController }) {
  const t = PROVIDER_COPY[detectLocale()];
  const styles = useUiStyles();
  const locked = c.exclusiveBusy;
  const toasts: ToastEntry[] = c.toasts;
  const recipients: ConsentRecipient[] = c.consentRecipients;

  return (
    <div className={styles.page} role="region" aria-label={t.providerListLabel}>
      {c.loadError ? <Alert type="error" showIcon title={t.loadFailed} action={<Button disabled={locked} onClick={c.onReloadFromError}>{t.reload}</Button>} /> : null}
      {c.selectionError ? <Alert type="error" showIcon title={t.loadFailed} action={<Button disabled={locked} onClick={c.onRetrySelectionLoad}>{t.retry}</Button>} data-testid="selection-error" /> : null}
      <div className={styles.twoColumn}>
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
          ) : <DetailEmpty t={t} />}
        </section>
      </div>

      <Modal
        open={c.deleteConfirmUuid !== null}
        title={t.deleteConfirmTitle}
        onCancel={c.onCancelDelete}
        data-testid="delete-confirm"
        footer={[
          <Button key="cancel" onClick={c.onCancelDelete}>{t.cancel}</Button>,
          <Button key="delete" type="primary" danger icon={c.deletingUuid !== null ? <Spin size="small" /> : undefined} disabled={c.deletingUuid !== null} onClick={c.onConfirmDelete}>{t.delete}</Button>,
        ]}
      >
        <Typography.Paragraph>{t.deleteConfirmMsg}</Typography.Paragraph>
      </Modal>

      {c.deleteError ? <Alert type="error" showIcon title={t.saveFailed} action={<div className={styles.row}><Button disabled={locked} onClick={c.onRetryDelete}>{t.retry}</Button><Button type="text" onClick={c.onDismissDeleteError}>{t.cancel}</Button></div>} data-testid="delete-error" /> : null}

      <Modal
        open={c.consentOpen}
        title={t.consent.title}
        onCancel={c.onCancelConsent}
        data-testid="consent-modal"
        footer={[<Button key="cancel" onClick={c.onCancelConsent}>{t.consent.cancel}</Button>, <Button key="confirm" type="primary" onClick={c.onConfirmConsent}>{t.consent.confirm}</Button>]}
      >
        <div className={styles.stack}>
          <Typography.Text>{t.consent.message}</Typography.Text>
          <ul>{recipients.map((recipient) => <li key={recipient.name}><Typography.Text strong>{recipient.name}</Typography.Text> <Typography.Text type="secondary">{recipient.localLabel}</Typography.Text></li>)}</ul>
        </div>
      </Modal>

      {toasts.length > 0 ? (
        <div className={styles.stack} aria-live="polite" data-testid="provider-toasts">
          {toasts.map((toast) => <Alert key={toast.id} type={toast.variant === "success" ? "success" : toast.variant === "warning" ? "warning" : toast.variant === "destructive" ? "error" : "info"} showIcon title={toast.message} action={<Button type="text" size="small" icon={<CloseOutlined aria-hidden />} aria-label={t.toastDismiss} onClick={() => c.onDismissToast(toast.id)} />} />)}
        </div>
      ) : null}
    </div>
  );
}

export default ProviderCenterView;
