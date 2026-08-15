import { createRoot } from "react-dom/client";
import "@mantine/core/styles.css";
import { AppProviders } from "../app/providers";
import { PopupView } from "../features/translation/PopupView";
import { usePopupController } from "../features/translation/popupController";

function PopupWindow() {
  const c = usePopupController();
  return <PopupView c={c} />;
}

const container = document.getElementById("root");
if (container) {
  createRoot(container).render(
    <AppProviders>
      <PopupWindow />
    </AppProviders>,
  );
}
