import type { Meta, StoryObj } from "@storybook/react-vite";
import VocabularyView from "./view";
import type { VocabularyController } from "./controller";

const noop = () => {};
const fake = (over: Record<string, unknown>): VocabularyController =>
  ({
    items: [
      { item_uuid: "i1", word: "serendipity", definition: "意外发现珍奇事物的运气", source_language: "en", target_language: "zh" },
      { item_uuid: "i2", word: "ephemeral", definition: "短暂的 / lasting a very short time", source_language: "en", target_language: "zh" },
    ],
    word: "",
    definition: "",
    notice: "",
    busy: false,
    setWord: noop,
    setDefinition: noop,
    add: noop,
    remove: noop,
    exportFile: noop,
    ...over,
  }) as unknown as VocabularyController;

const meta: Meta<typeof VocabularyView> = { title: "Settings/Vocabulary", component: VocabularyView };
export default meta;

export const Default: StoryObj<typeof VocabularyView> = { args: { c: fake({}) } };
export const Empty: StoryObj<typeof VocabularyView> = { args: { c: fake({ items: [] }) } };
export const BusyAdd: StoryObj<typeof VocabularyView> = { args: { c: fake({ busy: true, word: "x" }) } };
export const ExportNotice: StoryObj<typeof VocabularyView> = {
  args: { c: fake({ notice: "已导出到 /Users/me/linguaray-vocabulary.csv" }) } };
export const LongChinese: StoryObj<typeof VocabularyView> = {
  args: {
    c: fake({
      items: [
        {
          item_uuid: "i3",
          word: "人工智能驱动的机器翻译",
          definition: "超长中文释义用于验证列表行不溢出：本条目混合中英 (mixed CJK + Latin) 且长度远超列宽。",
          source_language: "zh",
          target_language: "en",
        },
      ],
    }),
  },
};
export const Dark: StoryObj<typeof VocabularyView> = {
  parameters: { colorScheme: "dark", backgrounds: { default: "dark" } },
  args: { c: fake({}) },
};
export const Narrow: StoryObj<typeof VocabularyView> = {
  parameters: { viewport: { defaultViewport: "mobile1" } },
  args: { c: fake({}) },
};
