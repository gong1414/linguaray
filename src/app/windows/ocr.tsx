import { createRoot } from "react-dom/client";
import "@mantine/core/styles.css";
import { AppProviders } from "../app/providers";
import { OcrOverlayView } from "../features/ocr/view";
import { useOcrController } from "../features/ocr/controller";

function OcrOverlayWindow() {
  const c = useOcrController();
  return <OcrOverlayView c={c} />;
}

const container = document.getElementById("root");
if (container) {
  createRoot(container).render(
    <AppProviders>
      <OcrOverlayWindow />
    </AppProviders>,
  );
}
