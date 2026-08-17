import type { Preview } from "@storybook/react";
import { AppProviders } from "../src/app/providers";

const preview: Preview = {
  decorators: [
    (Story, context) => (
      <AppProviders forceColorScheme={context.parameters.colorScheme === "dark" ? "dark" : undefined}>
        <Story />
      </AppProviders>
    ),
  ],
  parameters: {
    // Window stories must occupy the same canvas as their Tauri webview.
    // Individual component-only stories may opt back into `centered`.
    layout: "fullscreen",
    a11y: {
      test: "error",
    },
    controls: {
      matchers: {
        color: /(background|color)$/i,
        date: /Date$/i,
      },
    },
  },
};

export default preview;
