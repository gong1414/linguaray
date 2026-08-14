import { render } from "solid-js/web";
import OcrOverlay from "./OcrOverlay";

const root = document.getElementById("root");
if (root) render(() => <OcrOverlay />, root);
