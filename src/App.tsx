/**
 * R3a App mount — a thin wrapper that mounts SettingsShell hosting the
 * Provider Center (Surface 05) by default, with Keystore Recovery (Surface 06)
 * reachable via nav. Shortcuts and Privacy are R3b placeholders.
 *
 * The legacy monolithic settings/translate window (translate_clipboard, the
 * <select>/<textarea> key input, inline confirm()) is fully removed. Live
 * translation now lives in the Popup/InputPanel surfaces (R2b).
 */
import { createSignal, type Component } from "solid-js";
import SettingsShell, { type SettingsSection } from "./features/settings/SettingsShell";
import ProviderCenter from "./features/settings/ProviderCenter";
import KeystoreRecovery from "./features/settings/KeystoreRecovery";
import { SETTINGS_COPY } from "./features/settings/copy";
import { detectLocale } from "./i18n";

const App: Component = () => {
  const locale = detectLocale();
  const t = SETTINGS_COPY[locale];
  const [section, setSection] = createSignal<SettingsSection>("provider-center");

  return (
    <SettingsShell initialSection="provider-center" onNavigate={setSection}>
      {section() === "provider-center" ? (
        <ProviderCenter />
      ) : section() === "keystore-recovery" ? (
        <KeystoreRecovery />
      ) : (
        <section
          class="app__placeholder"
          aria-label={t.nav.placeholderHint}
        >
          <p>{t.nav.placeholderHint}</p>
        </section>
      )}
    </SettingsShell>
  );
};

export default App;
