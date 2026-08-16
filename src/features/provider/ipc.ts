/**
 * Typed wrappers for the provider Rust commands. Tauri camelCases command
 * parameter names on the JS side. Structured errors (stale_version /
 * stale_scope) propagate as rejections — callers narrow on `e?.error`.
 */
import { commands } from "../../bridge/invoke";
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

type GeneratedProviderProfile = Awaited<ReturnType<typeof commands.providerList>>[number];

function fromGeneratedProfile(profile: GeneratedProviderProfile): ProviderProfile {
  return {
    ...profile,
    status: profile.status as ProviderProfile["status"],
    capabilities: {
      ...profile.capabilities,
      auth: profile.capabilities.auth ?? undefined,
      models_url: profile.capabilities.models_url ?? undefined,
    },
  };
}

function toGeneratedPatch(patch: ProviderPatch): Parameters<typeof commands.providerUpdate>[1] {
  return {
    name: patch.name ?? null,
    endpoint: patch.endpoint ?? null,
    model: patch.model ?? null,
    enabled: patch.enabled ?? null,
    sort_order: patch.sort_order ?? null,
    expected_version: patch.expected_version,
    protocol: patch.protocol ?? null,
  };
}

export async function loadProviders(): Promise<ProviderProfileFE[]> {
  const [profiles, keyMap] = await Promise.all([
    commands.providerList(),
    commands.keyStatus(),
  ]);
  return profiles.map((p) => ({
    ...fromGeneratedProfile(p),
    hasKey: !!keyMap[p.secret_ref],
  }));
}

export const providerCreate = (
  templateId: string,
  name: string,
  endpoint: string,
  model?: string,
): Promise<ProviderProfile> =>
  commands.providerCreate(templateId, name, endpoint, model ?? null).then(fromGeneratedProfile);

export const providerUpdate = (
  uuid: string,
  patch: ProviderPatch,
): Promise<ProviderProfile> =>
  commands.providerUpdate(uuid, toGeneratedPatch(patch)).then(fromGeneratedProfile);

export const providerDuplicate = (uuid: string): Promise<ProviderProfile> =>
  commands.providerDuplicate(uuid).then(fromGeneratedProfile);

export const providerDelete = (uuid: string): Promise<void> =>
  commands.providerDelete(uuid).then(() => undefined);

export const providerReorder = (uuids: string[]): Promise<void> =>
  commands.providerReorder(uuids).then(() => undefined);

export const providerToggle = (uuid: string, enabled: boolean): Promise<void> =>
  commands.providerToggle(uuid, enabled).then(() => undefined);

export const providerGetBalance = (uuid: string): Promise<BalanceResultFE> =>
  commands.providerGetBalance(uuid);

export const providerSetKey = (uuid: string, key: string): Promise<void> =>
  commands.providerSetKey(uuid, key).then(() => undefined);

export const providerSetActive = (
  primary: string,
  parallel: string[],
  fallback: string | null,
): Promise<SetActiveResult> =>
  commands.providerSetActive(primary, parallel, fallback);

export const providerConfirmAndSetActive = (
  primary: string,
  parallel: string[],
  fallback: string | null,
  expectedScope: string,
): Promise<number> =>
  commands.providerConfirmAndSetActive(primary, parallel, fallback, expectedScope);

export const providerGetModels = (uuid: string): Promise<ModelInfo[]> =>
  commands.providerGetModels(uuid);

export const providerTestConnection = (uuid: string): Promise<ConnectionResult> =>
  commands.providerTestConnection(uuid);

export const providerGetActiveSelection = (): Promise<ActiveSelectionFE> =>
  commands.providerGetActiveSelection();

export const providerListPresets = async (): Promise<Preset[]> => {
  const rows: CatalogPresetDto[] = await commands.providerListPresets();
  return rows.map(catalogDtoToPreset);
};
