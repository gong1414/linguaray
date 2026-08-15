import { createRoot } from "react-dom/client";
import { useEffect, useState } from "react";
import "@mantine/core/styles.css";
import { AppProviders } from "../app/providers";
import { detectLocale } from "../i18n";
import { SettingsShellView } from "../features/shell/view";
import { useShellController } from "../features/shell/controller";
import { onWindowNavigation, runTrayAction } from "../features/shell/window-ipc";
import type { SettingsSection } from "../features/shell/model";
import { ProviderCenterView } from "../features/provider/view";
import { useProviderController } from "../features/provider/controller";
import { KeystoreRecoveryView } from "../features/keystore/view";
import { useKeystoreController } from "../features/keystore/controller";
import { ShortcutsView } from "../features/shortcuts/view";
import { useShortcutsController } from "../features/shortcuts/controller";
import { PrivacyView } from "../features/privacy/view";
import { usePrivacyController } from "../features/privacy/controller";
import { HistoryView } from "../features/history/view";
import { useHistoryController } from "../features/history/controller";
import { VocabularyView } from "../features/vocabulary/view";
import { useVocabularyController } from "../features/vocabulary/controller";
import { DictionaryView } from "../features/dictionary/view";
import { useDictionaryController } from "../features/dictionary/controller";
import { UpdaterPanelView } from "../features/updater/view";
import { useUpdaterController } from "../features/updater/controller";

/** One controller-live component per section (hooks stay unconditional). */
function ProviderSection() {
  const c = useProviderController();
  return <ProviderCenterView c={c} />;
}

function KeystoreSection() {
  const c = useKeystoreController();
  return <KeystoreRecoveryView c={c} />;
}

function ShortcutsSection() {
  const c = useShortcutsController();
  return <ShortcutsView c={c} />;
}

function PrivacySection() {
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

function HistorySection() {
  const c = useHistoryController();
  return (
    <HistoryView
      locale={detectLocale()}
      state={c.state}
      items={c.items}
      query={c.query}
      favoritesOnly={c.favoritesOnly}
      hasMore={c.hasMore}
      notice={c.notice}
      busy={c.busy}
      onQueryChange={c.setQuery}
      onSearch={c.search}
      onFavoritesOnlyChange={c.setFavoritesOnly}
      onLoadMore={c.loadMore}
      onToggleFavorite={c.toggleFavorite}
      onRemove={c.remove}
      onExport={c.exportFile}
    />
  );
}

function VocabularySection() {
  const c = useVocabularyController();
  return <VocabularyView c={c} />;
}

function DictionarySection() {
  const c = useDictionaryController();
  return <DictionaryView c={c} />;
}

function UpdaterSection() {
  const c = useUpdaterController();
  return <UpdaterPanelView c={c} />;
}

function sectionFor(active: SettingsSection) {
  switch (active) {
    case "provider-center":
      return <ProviderSection />;
    case "keystore-recovery":
      return <KeystoreSection />;
    case "shortcuts":
      return <ShortcutsSection />;
    case "privacy":
      return <PrivacySection />;
    case "history":
      return <HistorySection />;
    case "vocabulary":
      return <VocabularySection />;
    case "dictionary":
      return <DictionarySection />;
    case "updater":
      return <UpdaterSection />;
  }
}

/** Settings (main) window: shell + tray/navigation routing + sections. */
export function SettingsWindow() {
  const shell = useShellController();
  const [activePage, setActivePage] = useState<SettingsSection>("provider-center");

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let done = false;
    onWindowNavigation(
      (action) => {
        const target = runTrayAction(action);
        if (target) setActivePage(target);
      },
      (section) => setActivePage(section),
    )
      .then((u) => {
        if (done) u();
        else unlisten = u;
      })
      .catch(() => {});
    return () => {
      done = true;
      unlisten?.();
    };
  }, []);

  return (
    <SettingsShellView
      locale={detectLocale()}
      active={activePage}
      a11yGranted={shell.a11yGranted}
      onNavigate={setActivePage}
      onRecheckA11y={shell.recheckA11y}
      onOpenA11ySettings={shell.openSystemSettings}
    >
      {sectionFor(activePage)}
    </SettingsShellView>
  );
}

const container = document.getElementById("root");
if (container) {
  createRoot(container).render(
    <AppProviders>
      <SettingsWindow />
    </AppProviders>,
  );
}
