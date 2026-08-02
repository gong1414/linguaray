/* @refresh reload */
import { render } from "solid-js/web";
import App from "./App";

// Bring in the design-system tokens/reset/base exactly once.
import "@linguaray/ui/styles";

const root = document.getElementById("root");
if (!root) {
  throw new Error("UI Lab root element #root not found");
}

render(() => <App />, root);
