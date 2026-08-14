/** Official catalog fixture for jsdom tests (mirrors linguaray-catalog ids). */
export type CatalogPresetDto = {
  id: string;
  label: string;
  endpoint: string;
  default_model: string;
  needs_key: boolean;
  auth: string;
  requires_user_endpoint: boolean;
  notes: string | null;
  console_url: string | null;
  support_tier: "ready" | "setup_required" | "unverified";
  icon: string | null;
};

const READY = ["openai", "anthropic", "gemini", "ollama"] as const;

export const OFFICIAL_PRESET_DTOS: CatalogPresetDto[] = [
  { id: "openai", label: "OpenAI", endpoint: "https://api.openai.com/v1/chat/completions", default_model: "gpt-4o-mini", needs_key: true, auth: "bearer", requires_user_endpoint: false, notes: null, console_url: null, support_tier: "ready", icon: "openai" },
  { id: "anthropic", label: "Anthropic", endpoint: "https://api.anthropic.com/v1/messages", default_model: "claude-sonnet-4-5", needs_key: true, auth: "x-api-key", requires_user_endpoint: false, notes: null, console_url: null, support_tier: "ready", icon: "anthropic" },
  { id: "gemini", label: "Gemini", endpoint: "https://generativelanguage.googleapis.com/v1beta/openai/chat/completions", default_model: "gemini-3.6-flash", needs_key: true, auth: "bearer", requires_user_endpoint: false, notes: null, console_url: null, support_tier: "ready", icon: "gemini" },
  { id: "deepseek", label: "DeepSeek", endpoint: "https://api.deepseek.com/chat/completions", default_model: "deepseek-v4-flash", needs_key: true, auth: "bearer", requires_user_endpoint: false, notes: null, console_url: null, support_tier: "unverified", icon: null },
  { id: "openrouter", label: "OpenRouter", endpoint: "https://openrouter.ai/api/v1/chat/completions", default_model: "openai/gpt-4o-mini", needs_key: true, auth: "bearer", requires_user_endpoint: false, notes: null, console_url: null, support_tier: "unverified", icon: null },
  { id: "azure-openai", label: "Azure OpenAI", endpoint: "", default_model: "", needs_key: true, auth: "azure-key", requires_user_endpoint: true, notes: "Paste full URL", console_url: null, support_tier: "setup_required", icon: null },
  { id: "ollama", label: "Ollama", endpoint: "http://localhost:11434/v1/chat/completions", default_model: "qwen2.5:7b", needs_key: false, auth: "none", requires_user_endpoint: false, notes: null, console_url: null, support_tier: "ready", icon: null },
  { id: "custom", label: "Custom", endpoint: "", default_model: "", needs_key: true, auth: "bearer", requires_user_endpoint: true, notes: null, console_url: null, support_tier: "setup_required", icon: null },
  { id: "zhipu-glm", label: "智谱 GLM", endpoint: "https://open.bigmodel.cn/api/paas/v4/chat/completions", default_model: "glm-4-flash", needs_key: true, auth: "bearer", requires_user_endpoint: false, notes: null, console_url: null, support_tier: "unverified", icon: null },
  { id: "kimi", label: "Kimi", endpoint: "https://api.moonshot.cn/v1/chat/completions", default_model: "kimi-k3", needs_key: true, auth: "bearer", requires_user_endpoint: false, notes: null, console_url: null, support_tier: "unverified", icon: null },
  { id: "minimax", label: "MiniMax", endpoint: "https://api.minimax.io/v1/chat/completions", default_model: "MiniMax-M3", needs_key: true, auth: "bearer", requires_user_endpoint: false, notes: null, console_url: null, support_tier: "unverified", icon: null },
  { id: "bailian", label: "通义 / 百炼", endpoint: "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions", default_model: "qwen-plus", needs_key: true, auth: "bearer", requires_user_endpoint: false, notes: null, console_url: null, support_tier: "unverified", icon: null },
  { id: "doubao", label: "豆包", endpoint: "https://ark.cn-beijing.volces.com/api/v3/chat/completions", default_model: "", needs_key: true, auth: "bearer", requires_user_endpoint: false, notes: "Fill model", console_url: null, support_tier: "setup_required", icon: null },
  { id: "siliconflow", label: "SiliconFlow", endpoint: "https://api.siliconflow.cn/v1/chat/completions", default_model: "Qwen/Qwen2.5-7B-Instruct", needs_key: true, auth: "bearer", requires_user_endpoint: false, notes: null, console_url: null, support_tier: "unverified", icon: null },
  { id: "modelscope", label: "ModelScope", endpoint: "https://api-inference.modelscope.cn/v1/chat/completions", default_model: "Qwen/Qwen2.5-7B-Instruct", needs_key: true, auth: "bearer", requires_user_endpoint: false, notes: null, console_url: null, support_tier: "unverified", icon: null },
  { id: "stepfun", label: "StepFun", endpoint: "https://api.stepfun.com/v1/chat/completions", default_model: "step-3.7-flash", needs_key: true, auth: "bearer", requires_user_endpoint: false, notes: null, console_url: null, support_tier: "unverified", icon: null },
  { id: "xiaomi-mimo", label: "小米 MiMo", endpoint: "https://api.xiaomimimo.com/v1/chat/completions", default_model: "mimo-v2.5-pro", needs_key: true, auth: "azure-key", requires_user_endpoint: false, notes: null, console_url: null, support_tier: "unverified", icon: null },
  { id: "nvidia-nim", label: "NVIDIA NIM", endpoint: "https://integrate.api.nvidia.com/v1/chat/completions", default_model: "meta/llama-3.1-8b-instruct", needs_key: true, auth: "bearer", requires_user_endpoint: false, notes: null, console_url: null, support_tier: "unverified", icon: null },
  { id: "groq", label: "Groq", endpoint: "https://api.groq.com/openai/v1/chat/completions", default_model: "llama-3.3-70b-versatile", needs_key: true, auth: "bearer", requires_user_endpoint: false, notes: null, console_url: null, support_tier: "unverified", icon: null },
  { id: "mistral", label: "Mistral", endpoint: "https://api.mistral.ai/v1/chat/completions", default_model: "mistral-small-latest", needs_key: true, auth: "bearer", requires_user_endpoint: false, notes: null, console_url: null, support_tier: "unverified", icon: null },
  { id: "together", label: "Together", endpoint: "https://api.together.ai/v1/chat/completions", default_model: "Qwen/Qwen2.5-7B-Instruct-Turbo", needs_key: true, auth: "bearer", requires_user_endpoint: false, notes: null, console_url: null, support_tier: "unverified", icon: null },
];

export function dtoToPreset(dto: CatalogPresetDto) {
  return {
    templateId: dto.id,
    name: dto.id === "ollama" ? null : dto.label,
    endpoint: dto.endpoint,
    model: dto.default_model || null,
    needsKey: dto.needs_key,
    auth: dto.auth,
    requiresUserEndpoint: dto.requires_user_endpoint,
    notes: dto.notes,
    supportTier: dto.support_tier,
    icon: dto.icon,
  };
}

void READY;
