import { createRoot } from "react-dom/client";
import { AppProviders } from "../app/providers";
import { prepareWindowDocument } from "../app/windowDocument";
import { OcrOverlayView } from "../features/ocr/view";
import { useOcrController } from "../features/ocr/controller";

prepareWindowDocument({ transparent: true });

function OcrOverlayWindow() {
  const c = useOcrController();
  return <OcrOverlayView c={c} />;
}

const container = document.getElementById("root");
if (container) {
  createRoot(container).render(
    <AppProviders transparent>
      <OcrOverlayWindow />
    </AppProviders>,
  );
}
