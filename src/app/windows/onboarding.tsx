import { createRoot } from "react-dom/client";
import "@mantine/core/styles.css";
import { AppProviders } from "../app/providers";
import { OnboardingView } from "../features/onboarding/view";
import { useOnboardingController } from "../features/onboarding/controller";
import { detectLocale } from "../i18n";

function OnboardingWindow() {
  const c = useOnboardingController();
  return (
    <OnboardingView
      locale={detectLocale()}
      step={c.step}
      a11y={c.a11y}
      screenCapture={c.screenCapture}
      providerCount={c.providerCount}
      historyBusy={c.historyBusy}
      shortcuts={c.shortcuts}
      advancing={c.advancing}
      error={c.error}
      onOpenA11ySettings={c.openA11ySettings}
      onOpenScreenCaptureSettings={c.openScreenCaptureSettings}
      onRecheckPermissions={c.recheckPermissions}
      onOpenProviderSettings={c.openProviderSettings}
      onOpenShortcutsSettings={c.openShortcutsSettings}
      onEnableHistory={c.enableHistory}
      onAdvance={c.advance}
      onFinish={c.finish}
    />
  );
}

const container = document.getElementById("root");
if (container) {
  createRoot(container).render(
    <AppProviders>
      <OnboardingWindow />
    </AppProviders>,
  );
}
