import { invoke } from "../../../bridge/invoke";

const nameMap = new Map<string, string>();
let loaded = false;
let pending: Promise<void> | null = null;

/**
 * Load the provider name map once (idempotent). Returns a Promise that
 * resolves when the map is populated (or immediately if already loaded). The
 * caller awaits this before rendering results so card labels resolve
 * synchronously.
 */
export function ensureProviderNameMap(): Promise<void> {
  if (loaded) return Promise.resolve();
  if (pending) return pending;
  pending = (async () => {
    try {
      const profiles = await invoke<{ uuid: string; name: string }[]>("provider_list");
      for (const p of profiles) nameMap.set(p.uuid, p.name);
      loaded = true;
    } catch {
      // Best-effort; labels fall back below.
    } finally {
      pending = null;
    }
  })();
  return pending;
}

/** Reset the cached map (test-only: isolates each test's provider_list data). */
export function resetProviderNameMap(): void {
  nameMap.clear();
  loaded = false;
  pending = null;
}

const PRESET_LABELS: Record<string, string> = {
  openai: "OpenAI",
  anthropic: "Anthropic",
  gemini: "Gemini",
  ollama: "Ollama",
};

export function engineLabel(raw: string): string {
  if (nameMap.has(raw)) return nameMap.get(raw)!;
  if (raw.startsWith("provider/")) {
    const uuid = raw.slice("provider/".length);
    if (nameMap.has(uuid)) return nameMap.get(uuid)!;
  }
  if (PRESET_LABELS[raw]) return PRESET_LABELS[raw];
  if (["google", "deepl", "microsoft", "baidu", "youdao", "tencent"].includes(raw)) {
    return "Fallback";
  }
  return "Unknown";
}
