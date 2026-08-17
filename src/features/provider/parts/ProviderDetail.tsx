import {
  Badge,
  Button,
  Checkbox,
  Field,
  Input,
  MessageBar,
  MessageBarActions,
  MessageBarBody,
  MessageBarTitle,
  Select,
  Spinner,
  Text,
} from "@fluentui/react-components";
import { ServerRegular } from "@fluentui/react-icons";
import { Setting, SettingGroup, SettingGroupList } from "../../../ui/ueli";
import { useUiStyles } from "../../../ui/styles";
import type { ProviderCopy } from "../copy";
import type { ConnectionResult, ProviderDetailState } from "../model";

export type ProviderDetailProps = {
  t: ProviderCopy;
  detail: ProviderDetailState;
  reloading: boolean;
  exclusiveBusy: boolean;
  balanceText?: string;
  onNameInput: (uuid: string, value: string) => void;
  onEndpointInput: (uuid: string, value: string) => void;
  onModelInput: (uuid: string, value: string) => void;
  onModelChange: (uuid: string, value: string) => void;
  onKeyInput: (uuid: string, value: string) => void;
  onSaveProfile: (uuid: string) => void;
  onToggleCustomAnthropic: (uuid: string, anthropic: boolean) => void;
  onSaveKey: (uuid: string) => void;
  onFetchModels: (uuid: string) => void;
  onTestConnection: (uuid: string) => void;
  onFetchBalance: (uuid: string) => void;
  onResolveSaveConflict: (uuid: string) => void;
};

/** Ueli setting-group adapter for one provider profile. */
export function ProviderDetail(props: ProviderDetailProps) {
  const styles = useUiStyles();
  const t = props.t;
  const detail = props.detail;
  const uuid = detail.provider.uuid;
  const locked = props.exclusiveBusy;
  const isReloading = props.reloading;
  const fieldDisabled = detail.saveState === "saving" || isReloading || locked;
  const canListModels = detail.provider.capabilities.model_list;
  const hasUnsavedDrafts =
    detail.nameDraft !== detail.provider.name ||
    detail.endpointDraft !== detail.provider.endpoint ||
    detail.modelDraft !== (detail.provider.model ?? "") ||
    detail.keyText.length > 0;
  const modelOptions = detail.modelOptions.length > 0
    ? detail.modelOptions
    : [{ id: detail.modelDraft || "—", label: detail.modelDraft || "—" }];
  const connection = detail.conn;
  const connectionResult: ConnectionResult | null = connection !== "testing" && connection !== "idle" ? connection : null;
  const controlStyle = { width: "min(420px, 42vw)" };

  return (
    <div aria-label={t.detailLabel} data-testid="provider-detail">
      <SettingGroupList>
        <SettingGroup title={detail.provider.name}>
          {detail.saveConflict ? (
            <MessageBar intent="error" data-testid="save-conflict">
              <MessageBarBody><MessageBarTitle>{t.saveConflict}</MessageBarTitle></MessageBarBody>
              <MessageBarActions>
                <Button appearance="secondary" size="small" icon={isReloading ? <Spinner size="tiny" /> : undefined} disabled={isReloading || locked} onClick={() => props.onResolveSaveConflict(uuid)}>{t.reload}</Button>
              </MessageBarActions>
            </MessageBar>
          ) : null}

          <Setting
            label={t.name}
            control={
              <Field validationMessage={detail.nameError || undefined} validationState={detail.nameError ? "error" : "none"} style={controlStyle}>
                <Input aria-label={t.name} value={detail.nameDraft} disabled={fieldDisabled} onChange={(_, data) => props.onNameInput(uuid, data.value)} />
              </Field>
            }
          />

          <Setting
            label={t.endpoint.label}
            description={t.endpoint.placeholder}
            control={
              <div className={styles.stackTight} style={controlStyle}>
                <Field validationMessage={detail.endpointError || undefined} validationState={detail.endpointError ? "error" : "none"}>
                  <Input aria-label={t.endpoint.label} placeholder={t.endpoint.placeholder} value={detail.endpointDraft} disabled={fieldDisabled} onChange={(_, data) => props.onEndpointInput(uuid, data.value)} />
                </Field>
                {detail.provider.template_id === "azure-openai" ? <Button appearance="subtle" size="small" disabled={fieldDisabled} onClick={() => props.onEndpointInput(uuid, "https://{resource}.openai.azure.com/openai/v1/chat/completions")}>{t.insertAzureTemplate}</Button> : null}
                {detail.provider.template_id === "kimi" ? <Button appearance="subtle" size="small" disabled={fieldDisabled} onClick={() => props.onEndpointInput(uuid, "https://api.moonshot.ai/v1/chat/completions")}>{t.useKimiGlobal}</Button> : null}
                {detail.provider.template_id === "custom" ? <Checkbox label={t.customAnthropic} checked={detail.provider.protocol === "anthropic"} disabled={fieldDisabled} onChange={(_, data) => props.onToggleCustomAnthropic(uuid, Boolean(data.checked))} /> : null}
              </div>
            }
          />

          <Setting
            label={t.models}
            description={hasUnsavedDrafts ? t.saveFirstToFetch : undefined}
            control={
              <div className={styles.row} style={controlStyle}>
                {detail.modelFetch !== "error" && canListModels ? (
                  <Select aria-label={t.models} value={detail.modelDraft || ""} disabled={fieldDisabled} onChange={(event) => props.onModelChange(uuid, event.currentTarget.value)} style={{ flexGrow: 1 }}>
                    {modelOptions.map((model) => <option key={model.id} value={model.id}>{model.label}</option>)}
                  </Select>
                ) : (
                  <Input aria-label={t.models} placeholder={t.manualModelPlaceholder} value={detail.modelDraft} disabled={fieldDisabled} onChange={(_, data) => props.onModelInput(uuid, data.value)} style={{ flexGrow: 1 }} />
                )}
                {canListModels ? <Button appearance="secondary" disabled={locked || hasUnsavedDrafts || detail.modelFetch === "loading"} icon={detail.modelFetch === "loading" ? <Spinner size="tiny" /> : undefined} onClick={() => props.onFetchModels(uuid)} data-testid="fetch-models">{detail.modelFetch === "loading" ? t.loadingModels : t.fetchModels}</Button> : null}
              </div>
            }
          />

          <Setting
            label={t.saveProfile}
            control={
              <div className={styles.rowWrap} style={controlStyle}>
                <Button appearance="primary" icon={detail.saveState === "saving" ? <Spinner size="tiny" /> : undefined} disabled={detail.saveState === "saving" || isReloading || locked} onClick={() => props.onSaveProfile(uuid)}>{t.saveProfile}</Button>
                {detail.saveState === "saved" ? <Text className={styles.success}>{t.profileSaved}</Text> : null}
              </div>
            }
          />
        </SettingGroup>

        <SettingGroup title={t.apiKey}>
          <Setting
            label={t.apiKey}
            description={!detail.provider.needs_key ? t.noKeyRequired : detail.provider.hasKey ? t.keySaved : undefined}
            control={
              !detail.provider.needs_key ? <Badge appearance="tint" color="subtle">{t.noKeyRequired}</Badge>
                : detail.provider.hasKey ? <Badge appearance="tint" color="success">{t.keySaved}</Badge>
                  : (
                    <div className={styles.row} style={controlStyle}>
                      <Field validationMessage={detail.keyError || undefined} validationState={detail.keyError ? "error" : "none"} style={{ flexGrow: 1 }}>
                        <Input type="password" aria-label={t.apiKey} placeholder={t.apiKeyPlaceholder} value={detail.keyText} disabled={detail.saveState === "saving" || locked} onChange={(_, data) => props.onKeyInput(uuid, data.value)} />
                      </Field>
                      <Button appearance="primary" disabled={detail.keyText.trim().length === 0 || locked || detail.saveState === "saving"} icon={detail.saveState === "saving" ? <Spinner size="tiny" /> : undefined} onClick={() => props.onSaveKey(uuid)}>{t.saveKey}</Button>
                    </div>
                  )
            }
          />
        </SettingGroup>

        <SettingGroup title={t.testConnection}>
          <Setting
            label={t.testConnection}
            description={connectionResult ? `${connectionResult.message}${typeof connectionResult.latency_ms === "number" ? ` · ${connectionResult.latency_ms}ms` : ""}` : hasUnsavedDrafts ? t.saveFirstToTest : undefined}
            control={
              <div className={styles.rowWrap} style={controlStyle}>
                <Button appearance="secondary" icon={connection === "testing" ? <Spinner size="tiny" /> : undefined} disabled={locked || hasUnsavedDrafts || connection === "testing"} onClick={() => props.onTestConnection(uuid)} data-testid="test-connection">{t.testConnection}</Button>
                {connectionResult ? <Badge appearance="tint" color={connectionResult.ok ? "success" : "danger"}>{connectionResult.ok ? t.connectionOk : t.connectionFailed}</Badge> : null}
              </div>
            }
          />
          <Setting
            label={t.balance.title}
            description={!detail.provider.capabilities.balance ? t.balance.unsupportedNote : props.balanceText}
            control={detail.provider.capabilities.balance ? <Button appearance="subtle" size="small" onClick={() => props.onFetchBalance(uuid)}>{t.balance.fetch}</Button> : <Text size={300}>{t.balance.unsupportedNote}</Text>}
          />
        </SettingGroup>
      </SettingGroupList>
    </div>
  );
}

export function DetailEmpty({ t }: { t: ProviderCopy }) {
  const styles = useUiStyles();
  return <div className={styles.empty} data-testid="provider-detail-empty"><ServerRegular fontSize={28} aria-hidden /><Text>{t.selectPrimary}</Text></div>;
}

export default ProviderDetail;
