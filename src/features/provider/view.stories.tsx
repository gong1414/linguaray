import type { Meta, StoryObj } from "@storybook/react-vite";
import { FluentProvider, webDarkTheme } from "@fluentui/react-components";
import ProviderCenterView from "./view";
import type { ProviderController } from "./controller";
import type { ProviderProfileFE, ProviderDetailState } from "./model";

const profile = (uuid: string, over: Partial<ProviderProfileFE> = {}): ProviderProfileFE => ({
  uuid,
  template_id: "openai",
  name: `Provider ${uuid}`,
  protocol: "openai_chat",
  endpoint: "https://api.openai.com/v1",
  model: "gpt-4o",
  enabled: true,
  sort_order: 0,
  is_local: false,
  needs_key: true,
  secret_ref: `ref-${uuid}`,
  capabilities: { balance: false, quota: false, model_list: true },
  status: "active",
  version: 3,
  hasKey: true,
  ...over,
});

const A = profile("a", { sort_order: 0 });
const B = profile("b", { sort_order: 1, template_id: "ollama", endpoint: "http://localhost:11434", is_local: true, needs_key: false, hasKey: false, capabilities: { balance: false, quota: false, model_list: false } });

const detailOf = (p: ProviderProfileFE, over: Partial<ProviderDetailState> = {}): ProviderDetailState => ({
  provider: p,
  nameDraft: p.name,
  endpointDraft: p.endpoint,
  modelDraft: p.model ?? "",
  keyText: "",
  saveState: "idle",
  conn: "idle",
  modelOptions: [],
  modelFetch: "idle",
  saveConflict: false,
  ...over,
});

const noop = () => {};
const fake = (over: Record<string, unknown>): ProviderController =>
  ({
    presets: [{ templateId: "openai", name: "OpenAI", endpoint: "https://api.openai.com", model: "gpt-4o", needsKey: true, auth: "bearer", requiresUserEndpoint: false, notes: null, supportTier: "ready", icon: null }],
    providers: [A, B],
    selection: { primaryUuid: "a", parallelUuids: ["b"], fallbackUuid: null },
    loadError: false,
    selectionError: false,
    selectionLoading: false,
    selectedUuid: "a",
    deletingUuid: null,
    reloadingUuid: null,
    exclusiveBusy: false,
    deleteConfirmUuid: null,
    deleteError: false,
    deleteFailedUuid: null,
    consentOpen: false,
    consentRecipients: [],
    toasts: [],
    balanceByUuid: {},
    detail: detailOf(A),
    roleFor: (uuid: string) =>
      uuid === "a" ? { kind: "primary" } : uuid === "b" ? { kind: "parallel", index: 1 } : { kind: "none" },
    select: noop,
    onToggle: noop,
    onDelete: noop,
    onSetPrimary: noop,
    onAddParallel: noop,
    onRemoveParallel: noop,
    onSetFallback: noop,
    onDuplicate: noop,
    onMoveUp: noop,
    onMoveDown: noop,
    onAddPreset: noop,
    onNameInput: noop,
    onEndpointInput: noop,
    onModelInput: noop,
    onModelChange: noop,
    onKeyInput: noop,
    onSaveProfile: noop,
    onToggleCustomAnthropic: noop,
    onSaveKey: noop,
    onFetchModels: noop,
    onTestConnection: noop,
    onFetchBalance: noop,
    onResolveSaveConflict: noop,
    onReloadFromError: noop,
    onRetrySelectionLoad: noop,
    onConfirmDelete: noop,
    onCancelDelete: noop,
    onRetryDelete: noop,
    onDismissDeleteError: noop,
    onConfirmConsent: noop,
    onCancelConsent: noop,
    onDismissToast: noop,
    ...over,
  }) as unknown as ProviderController;

const meta: Meta<typeof ProviderCenterView> = {
  title: "Settings/Provider Center",
  component: ProviderCenterView,
  parameters: { viewport: { defaultViewport: "onboarding600" } },
};
export default meta;

export const Default: StoryObj<typeof ProviderCenterView> = { args: { c: fake({}) } };

export const Empty: StoryObj<typeof ProviderCenterView> = {
  args: { c: fake({ providers: [], selection: { primaryUuid: null, parallelUuids: [], fallbackUuid: null }, detail: null }) },
};

export const LoadError: StoryObj<typeof ProviderCenterView> = {
  args: { c: fake({ loadError: true, selectionError: true }) },
};

export const Locked: StoryObj<typeof ProviderCenterView> = {
  args: { c: fake({ exclusiveBusy: true, detail: detailOf(A, { saveState: "saving" }) }) },
};

export const KeyMissing: StoryObj<typeof ProviderCenterView> = {
  args: { c: fake({ detail: detailOf({ ...A, hasKey: false }), selectedUuid: "a" }) },
};

export const Testing: StoryObj<typeof ProviderCenterView> = {
  args: { c: fake({ detail: detailOf(A, { conn: "testing" }) }) },
};

export const Connected: StoryObj<typeof ProviderCenterView> = {
  args: { c: fake({ detail: detailOf(A, { conn: { ok: true, message: "200 OK", latency_ms: 132 } }) }) },
};

export const SaveConflict: StoryObj<typeof ProviderCenterView> = {
  args: { c: fake({ detail: detailOf(A, { saveConflict: true, saveState: "failed", endpointDraft: "https://draft.example.com" }) }) },
};

export const DeleteFlow: StoryObj<typeof ProviderCenterView> = {
  args: { c: fake({ deleteConfirmUuid: "a", deletingUuid: "a" }) },
};

export const DeleteError: StoryObj<typeof ProviderCenterView> = {
  args: { c: fake({ deleteError: true, deleteFailedUuid: "a" }) },
};

export const ConsentOpen: StoryObj<typeof ProviderCenterView> = {
  args: {
    c: fake({
      consentOpen: true,
      consentRecipients: [
        { name: "Provider a", localLabel: "remote" },
        { name: "Provider b", localLabel: "local" },
      ],
    }),
  },
};

export const LongChinese: StoryObj<typeof ProviderCenterView> = {
  parameters: { viewport: { defaultViewport: "mobile1" } },
  args: {
    c: fake({
      providers: [profile("zh", { name: "本地大模型服务商（Ollama 长名称测试）", sort_order: 0 })],
      detail: detailOf(profile("zh", { name: "本地大模型服务商（Ollama 长名称测试）" })),
    }),
  },
};

export const Dark: StoryObj<typeof ProviderCenterView> = {
  parameters: { backgrounds: { default: "dark" } },
  decorators: [
    (Story) => (
      <FluentProvider theme={webDarkTheme}>
        <Story />
      </FluentProvider>
    ),
  ],
  args: { c: fake({}) },
};
