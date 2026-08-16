import { createRoot } from "react-dom/client";
import { AppProviders } from "../app/providers";
import { prepareWindowDocument } from "../app/windowDocument";
import { PopupView } from "../features/translation/PopupView";
import { usePopupController } from "../features/translation/popupController";

prepareWindowDocument({ transparent: true });

function PopupWindow() {
  const c = usePopupController();
  return <PopupView c={c} />;
}

const container = document.getElementById("root");
if (container) {
  createRoot(container).render(
    <AppProviders transparent>
      <PopupWindow />
    </AppProviders>,
  );
}
