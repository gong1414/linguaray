import type { Meta, StoryObj } from "@storybook/react-vite";
import PopupView from "./PopupView";
import type { PopupController } from "./popupController";
import type { TranslationState } from "./types";

const noop = () => {};
const fake = (state: TranslationState, over: Record<string, unknown> = {}): PopupController =>
  ({
    state,
    pinned: false,
    lastSource: "hello world",
    hasSource: true,
    engineLabel: (raw: string) => (raw === "provider/u1" ? "MyOpenAI" : raw),
    pin: noop,
    unpin: noop,
    dismiss: noop,
    retrySelection: noop,
    copyText: async () => {},
    speak: async () => {},
    stopSpeaking: async () => {},
    favoriteText: async () => {},
    openSettings: async () => {},
    ...over,
  }) as unknown as PopupController;

const single: TranslationState = { kind: "single-success", text: "你好，世界", engine: "provider/u1" };

const meta: Meta<typeof PopupView> = { title: "Windows/Translation Popup", component: PopupView };
export default meta;

export const Loading: StoryObj<typeof PopupView> = { args: { c: fake({ kind: "loading" }, { lastSource: "", hasSource: false }) } };
export const LoadingWithRetry: StoryObj<typeof PopupView> = { args: { c: fake({ kind: "loading" }) } };
export const SingleSuccess: StoryObj<typeof PopupView> = { args: { c: fake(single) } };
export const Pinned: StoryObj<typeof PopupView> = { args: { c: fake(single, { pinned: true }) } };
export const MultiSuccess: StoryObj<typeof PopupView> = {
  args: {
    c: fake({
      kind: "multi-success",
      results: [
        { uuid: "u1", engine: "provider/u1", text: "你好", ok: true },
        { uuid: "u2", engine: "provider/u2", text: "您好", ok: true },
      ],
    }),
  },
};
export const Partial: StoryObj<typeof PopupView> = {
  args: {
    c: fake({
      kind: "partial",
      results: [
        { uuid: "u1", engine: "provider/u1", text: "你好", ok: true },
        { uuid: "u2", engine: "provider/u2", errorText: "timeout after 30s", ok: false },
      ],
    }),
  },
};
export const NetworkError: StoryObj<typeof PopupView> = {
  args: { c: fake({ kind: "error", sub: "network", message: "network error: timeout" }) },
};
export const ConfigKeyError: StoryObj<typeof PopupView> = {
  args: { c: fake({ kind: "error", sub: "config-key", message: "missing key" }) },
};
export const KeystoreCorrupt: StoryObj<typeof PopupView> = {
  args: { c: fake({ kind: "keystore-corrupt", message: "unreadable" }) },
};
export const LongChinese: StoryObj<typeof PopupView> = {
  args: {
    c: fake({ kind: "single-success", text: "这是一段超长的中文翻译结果，用于验证无边框翻译浮窗在长文本下换行与滚动，绝不撑破窗口边界，同时保留混合中英 (CJK + Latin) 的可读性。", engine: "provider/u1" }),
  },
};
export const Dark: StoryObj<typeof PopupView> = {
  parameters: { colorScheme: "dark", backgrounds: { default: "dark" } },
  args: { c: fake(single) },
};
