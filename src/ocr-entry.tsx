import { render } from "solid-js/web";
import "@linguaray/ui/styles";
import { initTheme } from "./theme";
import OcrOverlay from "./OcrOverlay";

initTheme();
const root = document.getElementById("root");
if (root) render(() => <OcrOverlay />, root);
