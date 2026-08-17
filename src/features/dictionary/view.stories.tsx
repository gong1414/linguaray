import type { Meta, StoryObj } from "@storybook/react-vite";
import DictionaryView from "./view";
import type { DictionaryController } from "./controller";

const noop = () => {};
const set = noop;
const fake = (over: Record<string, unknown>): DictionaryController =>
  ({
    word: "",
    result: null,
    miss: false,
    packages: [{ package_id: "en-zh-1", name: "EN→ZH Core" }],
    error: "",
    notice: "",
    sourceDir: "",
    packageId: "",
    packageName: "",
    version: "1.0",
    installing: false,
    setWord: set,
    setSourceDir: set,
    setPackageId: set,
    setPackageName: set,
    setVersion: set,
    lookup: noop,
    install: noop,
    ...over,
  }) as unknown as DictionaryController;

const meta: Meta<typeof DictionaryView> = { title: "Settings/Dictionary", component: DictionaryView };
export default meta;

export const Default: StoryObj<typeof DictionaryView> = { args: { c: fake({}) } };
export const NoPackages: StoryObj<typeof DictionaryView> = { args: { c: fake({ packages: [] }) } };
export const Result: StoryObj<typeof DictionaryView> = {
  args: { c: fake({ result: { definition: "你好 — a greeting in Mandarin Chinese.", source: "en-zh-1" }, word: "hello" }) },
};
export const Miss: StoryObj<typeof DictionaryView> = { args: { c: fake({ miss: true, word: "zzz" }) } };
export const Installing: StoryObj<typeof DictionaryView> = {
  args: { c: fake({ installing: true, sourceDir: "/data", packageId: "x" }) },
};
export const ErrorState: StoryObj<typeof DictionaryView> = {
  args: { c: fake({ error: "package manifest missing" }) },
};
export const LongChinese: StoryObj<typeof DictionaryView> = {
  args: {
    c: fake({
      locale: undefined,
      word: "机器翻译",
      result: {
        definition: "机器翻译（英语：Machine Translation，常简写为 MT）是计算语言学的其中一个分支，研究如何让电脑自动将文字从一种自然语言翻译成另一种自然语言。",
        source: "zh-百科",
      },
    }),
  },
};
export const Dark: StoryObj<typeof DictionaryView> = {
  parameters: { colorScheme: "dark", backgrounds: { default: "dark" } },
  args: { c: fake({ word: "hello", result: { definition: "你好", source: "en-zh-1" } }) },
};
export const Narrow: StoryObj<typeof DictionaryView> = {
  parameters: { viewport: { defaultViewport: "mobile1" } },
  args: { c: fake({}) },
};
