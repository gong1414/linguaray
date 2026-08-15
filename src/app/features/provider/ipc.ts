/**
 * Typed wrappers for the provider Rust commands. Tauri camelCases command
 * parameter names on the JS side. Structured errors (stale_version /
 * stale_scope) propagate as rejections — callers narrow on `e?.error`.
 */
import { invoke } from "../../../bridge/invoke";
import type {
  ActiveSelectionFE,
  ConnectionResult,
  ModelInfo,
  Preset,
  CatalogPresetDto,
  ProviderPatch,
  ProviderProfile,
  ProviderProfileFE,
  SetActiveResult,
  BalanceResultFE,
} from "./model";
import { catalogDtoToPreset } from "./model";

export async function loadProviders(): Promise<ProviderProfileFE[]> {
  const [profiles, keyMap] = await Promise.all([
    invoke<ProviderProfile[]>("provider_list"),
    invoke<Record<string, boolean>>("key_status"),
  ]);
  return profiles.map((p) => ({ ...p, hasKey: !!keyMap[p.secret_ref] }));
}

export const providerCreate = (
  templateId: string,
  name: string,
  endpoint: string,
  model?: string,
): Promise<ProviderProfile> =>
  invoke<ProviderProfile>("provider_create", {
    templateId,
    name,
    endpoint,
    model: model ?? null,
  });

export const providerUpdate = (
  uuid: string,
  patch: ProviderPatch,
): Promise<ProviderProfile> => invoke<ProviderProfile>("provider_update", { uuid, patch });

export const providerDuplicate = (uuid: string): Promise<ProviderProfile> =>
  invoke<ProviderProfile>("provider_duplicate", { uuid });

export const providerDelete = (uuid: string): Promise<void> =>
  invoke<void>("provider_delete", { uuid });

export const providerReorder = (uuids: string[]): Promise<void> =>
  invoke<void>("provider_reorder", { uuids });

export const providerToggle = (uuid: string, enabled: boolean): Promise<void> =>
  invoke<void>("provider_toggle", { uuid, enabled });

export const providerGetBalance = (uuid: string): Promise<BalanceResultFE> =>
  invoke<BalanceResultFE>("provider_get_balance", { uuid });

export const providerSetKey = (uuid: string, key: string): Promise<void> =>
  invoke<void>("provider_set_key", { uuid, key });

export const providerSetActive = (
  primary: string,
  parallel: string[],
  fallback: string | null,
): Promise<SetActiveResult> =>
  invoke<SetActiveResult>("provider_set_active", { primary, parallel, fallback });

export const providerConfirmAndSetActive = (
  primary: string,
  parallel: string[],
  fallback: string | null,
  expectedScope: string,
): Promise<number> =>
  invoke<number>("provider_confirm_and_set_active", {
    primary,
    parallel,
    fallback,
    expectedScope,
  });

export const providerGetModels = (uuid: string): Promise<ModelInfo[]> =>
  invoke<ModelInfo[]>("provider_get_models", { uuid });

export const providerTestConnection = (uuid: string): Promise<ConnectionResult> =>
  invoke<ConnectionResult>("provider_test_connection", { uuid });

export const providerGetActiveSelection = (): Promise<ActiveSelectionFE> =>
  invoke<ActiveSelectionFE>("provider_get_active_selection");

export const providerListPresets = async (): Promise<Preset[]> => {
  const rows = await invoke<CatalogPresetDto[]>("provider_list_presets");
  return rows.map(catalogDtoToPreset);
};
