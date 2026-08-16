import type { Meta, StoryObj } from "@storybook/react-vite";
import { FluentProvider, webDarkTheme } from "@fluentui/react-components";
import OcrOverlayView from "./view";
import type { OcrController } from "./controller";

const noop = () => {};
const fake = (over: Record<string, unknown>): OcrController =>
  ({
    rect: null,
    notice: null,
    busy: false,
    startDrag: noop,
    moveDrag: noop,
    finishDrag: noop,
    runRegion: noop,
    runFile: noop,
    runClipboard: noop,
    cancel: noop,
    ...over,
  }) as unknown as OcrController;

const meta: Meta<typeof OcrOverlayView> = {
  title: "Windows/OCR Overlay",
  component: OcrOverlayView,
  parameters: { layout: "fullscreen" },
};
export default meta;

export const Idle: StoryObj<typeof OcrOverlayView> = { args: { c: fake({}) } };
export const Dragging: StoryObj<typeof OcrOverlayView> = {
  args: { c: fake({ rect: { x: 120, y: 80, w: 320, h: 180 } }) },
};
export const Recognizing: StoryObj<typeof OcrOverlayView> = { args: { c: fake({ busy: true }) } };
export const ErrorState: StoryObj<typeof OcrOverlayView> = {
  args: { c: fake({ notice: "engine unavailable" }) },
};
export const Dark: StoryObj<typeof OcrOverlayView> = {
  parameters: { backgrounds: { default: "dark" } },
  decorators: [
    (Story) => (
      <FluentProvider theme={webDarkTheme}>
        <Story />
      </FluentProvider>
    ),
  ],
  args: { c: fake({ rect: { x: 60, y: 40, w: 260, h: 140 } }) },
};
