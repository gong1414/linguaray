import { render } from "solid-js/web";
import "@linguaray/ui/styles";
import { initTheme } from "./theme";
import InputPanel from "./InputPanel";
initTheme();
render(() => <InputPanel />, document.getElementById("root")!);
