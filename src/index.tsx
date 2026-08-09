/* @refresh reload */
import { render } from "solid-js/web";
import "@linguaray/ui/styles";
import { initTheme } from "./theme";
import App from "./App";
initTheme();
render(() => <App />, document.getElementById("root") as HTMLElement);
