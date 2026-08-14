/**
 * R3a App mount + R2/R3a audit Task A4: hosts the tray-action + navigate
 * listeners. The tray (Surface 04) emits `tray-action`; `open_settings_window`
 * emits `navigate`. The shell's activePage is a CONTROLLED signal (P1-5) so the
 * tray / popup CTAs can drive navigation.
 *
 * Surface 04 scope (rev-10): normal icon, provider name status,
 * translate-selection/clipboard/switch-provider/settings/quit are live. OCR +
 * History are disabled with "Coming later". Update badge, active-translation
 * pulse, and Balance are not implemented (see Surface status table).
 */
import { createSignal, onCleanup, onMount, type Component } from "solid-js";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import SettingsShell, { type SettingsSection } from "./features/settings/SettingsShell";
import ProviderCenter from "./features/settings/ProviderCenter";
import KeystoreRecovery from "./features/settings/KeystoreRecovery";
import Shortcuts from "./features/settings/Shortcuts";
import PrivacyData from "./features/settings/PrivacyData";
import HistoryView from "./features/settings/HistoryView";
import VocabularyView from "./features/settings/VocabularyView";
import DictionaryView from "./features/settings/DictionaryView";
import { SETTINGS_COPY } from "./features/settings/copy";
import { translateSelection, translateClipboard } from "./features/translation/selection-ipc";
import { detectLocale } from "./i18n";

const App: Component = () => {
  const locale = detectLocale();
  const t = SETTINGS_COPY[locale];
  // rev-7-2: activePage uses the EXISTING SettingsSection union (no new type).
  // It is passed as the `activePage` prop so the parent controls the shell.
  const [activePage, setActivePage] = createSignal<SettingsSection>("provider-center");
  const unlisteners: UnlistenFn[] = [];

  onMount(async () => {
    unlisteners.push(
      await listen<string>("tray-action", (e) => {
        const action = e.payload;
        if (action === "translate-clipboard") {
          void translateClipboard();
        } else if (action === "translate-selection") {
          void translateSelection();
        } else if (action === "ocr-capture") {
          // Disabled in the menu (Coming later); no-op here.
        } else if (action === "switch-provider" || action === "settings") {
          setActivePage("provider-center");
        }
      }),
    );
    unlisteners.push(
      await listen<string>("navigate", (e) => {
        const page = e.payload as SettingsSection;
        if (
          page === "provider-center" ||
          page === "keystore-recovery" ||
          page === "shortcuts" ||
          page === "privacy" ||
          page === "history" ||
          page === "vocabulary" ||
          page === "dictionary"
        ) {
          setActivePage(page);
        }
      }),
    );
  });

  onCleanup(() => {
    for (const u of unlisteners) u();
  });

  return (
    <SettingsShell activePage={activePage()} onNavigate={setActivePage}>
      {activePage() === "provider-center" ? (
        <ProviderCenter />
      ) : activePage() === "keystore-recovery" ? (
        <KeystoreRecovery />
      ) : activePage() === "shortcuts" ? (
        <Shortcuts />
      ) : activePage() === "privacy" ? (
        <PrivacyData />
      ) : activePage() === "history" ? (
        <HistoryView />
      ) : activePage() === "vocabulary" ? (
        <VocabularyView />
      ) : activePage() === "dictionary" ? (
        <DictionaryView />
      ) : (
        <section class="app__placeholder" aria-label={t.nav.placeholderHint}>
          <p>{t.nav.placeholderHint}</p>
        </section>
      )}
    </SettingsShell>
  );
};

export default App;
