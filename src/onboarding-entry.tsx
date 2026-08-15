import { render } from "solid-js/web";
import "@linguaray/ui/styles";
import { initTheme } from "./theme";
import Onboarding from "./Onboarding";

/**
 * Entry for onboarding.html. Loading the design-system stylesheet +
 * initTheme() here is load-bearing (R6): without them the window renders
 * with browser-default fonts/spacing and no tokens. `mountOnboarding` exists
 * so the regression test can execute exactly this wiring under jsdom.
 */
export function mountOnboarding(root: HTMLElement): void {
  initTheme();
  render(() => <Onboarding />, root);
}

initTheme();
const root = document.getElementById("root");
if (root) mountOnboarding(root);
