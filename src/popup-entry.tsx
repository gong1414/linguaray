import { render } from "solid-js/web";
import "@linguaray/ui/styles";
import { initTheme } from "./theme";
import Popup from "./Popup";
initTheme();
render(() => <Popup />, document.getElementById("root")!);
