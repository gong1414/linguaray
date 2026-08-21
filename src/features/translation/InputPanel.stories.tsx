import type { Meta, StoryObj } from "@storybook/react-vite";
import InputPanelView from "./InputPanelView";
import type { InputController } from "./inputController";
import type { TranslationState } from "./types";

const noop = () => {};
const fake = (state: TranslationState, over: Record<string, unknown> = {}): InputController =>
  ({
    text: "hello world",
    state,
    idle: true,
    hasResult: true,
    favoritedKey: null,
    setText: noop,
    textareaRef: { current: null },
    translate: noop,
    clear: noop,
    favorite: noop,
    copyText: async () => {},
    ...over,
  }) as unknown as InputController;

const meta: Meta<typeof InputPanelView> = { title: "Windows/Input", component: InputPanelView };
export default meta;

const single: TranslationState = { kind: "single-success", text: "你好，世界", engine: "provider/u1" };
const multi: TranslationState = {
  kind: "multi-success",
  results: [
    { uuid: "u1", engine: "provider/u1", text: "你好", ok: true },
    { uuid: "u2", engine: "provider/u2", text: "您好", ok: true },
  ],
};
const partial: TranslationState = {
  kind: "partial",
  results: [
    { uuid: "u1", engine: "provider/u1", text: "你好", ok: true },
    { uuid: "u2", engine: "provider/u2", errorText: "timeout", ok: false },
  ],
};

export const Idle: StoryObj<typeof InputPanelView> = {
  args: { c: fake({ kind: "loading" }, { hasResult: false, text: "" }) },
};
export const Loading: StoryObj<typeof InputPanelView> = {
  args: { c: fake({ kind: "loading" }, { idle: false }) },
};
export const SingleSuccess: StoryObj<typeof InputPanelView> = { args: { c: fake(single) } };
export const MultiSuccess: StoryObj<typeof InputPanelView> = { args: { c: fake(multi) } };
export const Partial: StoryObj<typeof InputPanelView> = { args: { c: fake(partial) } };
export const ErrorState: StoryObj<typeof InputPanelView> = {
  args: { c: fake({ kind: "error", sub: "network", message: "timeout" }) },
};
export const Offline: StoryObj<typeof InputPanelView> = {
  args: { c: fake({ kind: "offline", message: "offline" }) },
};
export const LongChinese: StoryObj<typeof InputPanelView> = {
  args: {
    c: fake({ kind: "single-success", text: "这是一段超长的中文翻译结果，用于验证输入窗口在长文本下正确换行、滚动且不撑破 420×280 的窗口边界。(mixed CJK + Latin)", engine: "provider/u1" }),
  },
};
export const Dark: StoryObj<typeof InputPanelView> = {
  parameters: { colorScheme: "dark", backgrounds: { default: "dark" } },
  args: { c: fake(single) },
};
