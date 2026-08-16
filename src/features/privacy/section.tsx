import { detectLocale } from "../../app/i18n";
import { usePrivacyController } from "./controller";
import { PrivacyView } from "./view";

/** Self-composing settings section — the window file only routes. */
export function PrivacySection() {
  const c = usePrivacyController();
  return (
    <PrivacyView
      locale={detectLocale()}
      status={c.status}
      loading={c.loading}
      error={c.error}
      busy={c.busy}
      clearOpen={c.clearOpen}
      toasts={c.toasts}
      external={c.external}
      externalBusy={c.externalBusy}
      tokenOnce={c.tokenOnce}
      tokenCopied={c.tokenCopied}
      onRetry={c.retry}
      onEnabledChange={c.setEnabled}
      onRetentionChange={c.setRetention}
      onOpenClear={c.openClear}
      onCloseClear={c.closeClear}
      onConfirmClear={c.confirmClear}
      onEnableExternal={c.enableExternal}
      onDisableExternal={c.disableExternal}
      onRegenToken={c.regenToken}
      onCopyToken={c.copyToken}
      onDismissToast={c.dismissToast}
    />
  );
}
