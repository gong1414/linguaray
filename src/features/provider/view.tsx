import {
  Button,
  Dialog,
  DialogActions,
  DialogBody,
  DialogContent,
  DialogSurface,
  DialogTitle,
  MessageBar,
  MessageBarActions,
  MessageBarBody,
  MessageBarTitle,
  Spinner,
  Text,
} from "@fluentui/react-components";
import { DismissRegular } from "@fluentui/react-icons";
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
      {c.loadError && (
        <MessageBar intent="error">
          <MessageBarBody><MessageBarTitle>{t.loadFailed}</MessageBarTitle></MessageBarBody>
          <MessageBarActions><Button size="small" appearance="secondary" disabled={locked} onClick={c.onReloadFromError}>{t.reload}</Button></MessageBarActions>
        </MessageBar>
      )}
      {c.selectionError && (
        <MessageBar intent="error" data-testid="selection-error">
          <MessageBarBody>{t.loadFailed}</MessageBarBody>
          <MessageBarActions><Button size="small" appearance="secondary" disabled={locked} onClick={c.onRetrySelectionLoad}>{t.retry}</Button></MessageBarActions>
        </MessageBar>
      )}

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

      <Dialog open={c.deleteConfirmUuid !== null} onOpenChange={(_, data) => !data.open && c.onCancelDelete()}>
        <DialogSurface data-testid="delete-confirm">
          <DialogBody>
            <DialogTitle>{t.deleteConfirmTitle}</DialogTitle>
            <DialogContent><Text>{t.deleteConfirmMsg}</Text></DialogContent>
            <DialogActions>
              <Button appearance="secondary" onClick={c.onCancelDelete}>{t.cancel}</Button>
              <Button appearance="primary" icon={c.deletingUuid !== null ? <Spinner size="tiny" /> : undefined} disabled={c.deletingUuid !== null} onClick={c.onConfirmDelete}>{t.delete}</Button>
            </DialogActions>
          </DialogBody>
        </DialogSurface>
      </Dialog>

      {c.deleteError && (
        <MessageBar intent="error" data-testid="delete-error">
          <MessageBarBody><MessageBarTitle>{t.saveFailed}</MessageBarTitle></MessageBarBody>
          <MessageBarActions>
            <Button size="small" appearance="secondary" disabled={locked} onClick={c.onRetryDelete}>{t.retry}</Button>
            <Button size="small" appearance="subtle" onClick={c.onDismissDeleteError}>{t.cancel}</Button>
          </MessageBarActions>
        </MessageBar>
      )}

      <Dialog open={c.consentOpen} onOpenChange={(_, data) => !data.open && c.onCancelConsent()}>
        <DialogSurface data-testid="consent-modal">
          <DialogBody>
            <DialogTitle>{t.consent.title}</DialogTitle>
            <DialogContent>
              <div className={styles.stack}>
                <Text>{t.consent.message}</Text>
                <ul>
                  {recipients.map((recipient) => <li key={recipient.name}><Text weight="semibold">{recipient.name}</Text> <Text size={300} className={styles.muted}>{recipient.localLabel}</Text></li>)}
                </ul>
              </div>
            </DialogContent>
            <DialogActions>
              <Button appearance="secondary" onClick={c.onCancelConsent}>{t.consent.cancel}</Button>
              <Button appearance="primary" onClick={c.onConfirmConsent}>{t.consent.confirm}</Button>
            </DialogActions>
          </DialogBody>
        </DialogSurface>
      </Dialog>

      {toasts.length > 0 && (
        <div className={styles.stack} aria-live="polite" data-testid="provider-toasts">
          {toasts.map((toast) => (
            <MessageBar key={toast.id} intent={toast.variant === "success" ? "success" : toast.variant === "warning" ? "warning" : toast.variant === "destructive" ? "error" : "info"}>
              <MessageBarBody>{toast.message}</MessageBarBody>
              <MessageBarActions containerAction={<Button appearance="transparent" icon={<DismissRegular />} aria-label={t.toastDismiss} onClick={() => c.onDismissToast(toast.id)} />} />
            </MessageBar>
          ))}
        </div>
      )}
    </div>
  );
}

export default ProviderCenterView;
