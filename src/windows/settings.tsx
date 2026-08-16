import { createRoot } from "react-dom/client";
import { useEffect, useState } from "react";
import { AppProviders } from "../app/providers";
import { prepareWindowDocument } from "../app/windowDocument";
import { detectLocale } from "../app/i18n";
import { SettingsShellView } from "../features/shell/view";
import { useShellController } from "../features/shell/controller";
import { onWindowNavigation, runTrayAction } from "../features/shell/window-ipc";
import type { SettingsSection } from "../features/shell/model";
import { ProviderSection } from "../features/provider/section";
import { KeystoreSection } from "../features/keystore/section";
import { ShortcutsSection } from "../features/shortcuts/section";
import { PrivacySection } from "../features/privacy/section";
import { HistorySection } from "../features/history/section";
import { VocabularySection } from "../features/vocabulary/section";
import { DictionarySection } from "../features/dictionary/section";
import { UpdaterSection } from "../features/updater/section";

prepareWindowDocument();

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
