import { createRoot } from "react-dom/client";
import "@mantine/core/styles.css";
import { AppProviders } from "../app/providers";
import { InputPanelView } from "../features/translation/InputPanelView";
import { useInputController } from "../features/translation/inputController";

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
