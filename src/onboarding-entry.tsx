import { render } from "solid-js/web";
import Onboarding from "./Onboarding";

const root = document.getElementById("root");
if (root) render(() => <Onboarding />, root);
