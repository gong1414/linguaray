import { createRoot } from "react-dom/client";
import { AppProviders } from "../app/providers";
import { prepareWindowDocument } from "../app/windowDocument";
import { InputPanelView } from "../features/translation/InputPanelView";
import { useInputController } from "../features/translation/inputController";

prepareWindowDocument();

function InputWindow() {
  const c = useInputController();
  return <InputPanelView c={c} />;
}

const container = document.getElementById("root");
if (container) {
  createRoot(container).render(
    <AppProviders>
      <InputWindow />
    </AppProviders>,
  );
}
