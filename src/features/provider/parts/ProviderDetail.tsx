import { Alert, Button, Checkbox, Empty, Form, Input, Select, Spin, Tag, Typography } from "antd";
import { CloudServerOutlined } from "@ant-design/icons";
import { Setting, SettingGroup, SettingGroupList } from "../../../ui/x";
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

/** Ant Design setting groups for one provider profile. */
export function ProviderDetail(props: ProviderDetailProps) {
  const styles = useUiStyles();
  const t = props.t;
  const detail = props.detail;
  const uuid = detail.provider.uuid;
  const locked = props.exclusiveBusy;
  const isReloading = props.reloading;
  const fieldDisabled = detail.saveState === "saving" || isReloading || locked;
  const canListModels = detail.provider.capabilities.model_list;
  const hasUnsavedDrafts = detail.nameDraft !== detail.provider.name || detail.endpointDraft !== detail.provider.endpoint || detail.modelDraft !== (detail.provider.model ?? "") || detail.keyText.length > 0;
  const modelOptions = detail.modelOptions.length > 0 ? detail.modelOptions : [{ id: detail.modelDraft || "—", label: detail.modelDraft || "—" }];
  const connection = detail.conn;
  const connectionResult: ConnectionResult | null = connection !== "testing" && connection !== "idle" ? connection : null;
  const controlStyle = { width: "min(420px, 42vw)" };

  return (
    <div aria-label={t.detailLabel} data-testid="provider-detail">
      <SettingGroupList>
        <SettingGroup title={detail.provider.name}>
          {detail.saveConflict ? <Alert type="error" showIcon title={t.saveConflict} action={<Button icon={isReloading ? <Spin size="small" /> : undefined} disabled={isReloading || locked} onClick={() => props.onResolveSaveConflict(uuid)}>{t.reload}</Button>} data-testid="save-conflict" /> : null}
          <Setting
            label={t.name}
            control={
              <Form.Item validateStatus={detail.nameError ? "error" : undefined} help={detail.nameError || undefined} style={controlStyle}>
                <Input aria-label={t.name} value={detail.nameDraft} disabled={fieldDisabled} onChange={(event) => props.onNameInput(uuid, event.currentTarget.value)} />
              </Form.Item>
            }
          />
          <Setting
            label={t.endpoint.label}
            description={t.endpoint.placeholder}
            control={
              <div className={styles.stackTight} style={controlStyle}>
                <Form.Item validateStatus={detail.endpointError ? "error" : undefined} help={detail.endpointError || undefined}>
                  <Input aria-label={t.endpoint.label} placeholder={t.endpoint.placeholder} value={detail.endpointDraft} disabled={fieldDisabled} onChange={(event) => props.onEndpointInput(uuid, event.currentTarget.value)} />
                </Form.Item>
                {detail.provider.template_id === "azure-openai" ? <Button type="text" size="small" disabled={fieldDisabled} onClick={() => props.onEndpointInput(uuid, "https://{resource}.openai.azure.com/openai/v1/chat/completions")}>{t.insertAzureTemplate}</Button> : null}
                {detail.provider.template_id === "kimi" ? <Button type="text" size="small" disabled={fieldDisabled} onClick={() => props.onEndpointInput(uuid, "https://api.moonshot.ai/v1/chat/completions")}>{t.useKimiGlobal}</Button> : null}
                {detail.provider.template_id === "custom" ? <Checkbox checked={detail.provider.protocol === "anthropic"} disabled={fieldDisabled} onChange={(event) => props.onToggleCustomAnthropic(uuid, event.target.checked)}>{t.customAnthropic}</Checkbox> : null}
              </div>
            }
          />
          <Setting
            label={t.models}
            description={hasUnsavedDrafts ? t.saveFirstToFetch : undefined}
            control={
              <div className={styles.row} style={controlStyle}>
                {detail.modelFetch !== "error" && canListModels ? (
                  <Select aria-label={t.models} value={detail.modelDraft || ""} disabled={fieldDisabled} options={modelOptions.map((model) => ({ value: model.id, label: model.label }))} onChange={(value) => props.onModelChange(uuid, value)} style={{ flexGrow: 1 }} />
                ) : (
                  <Input aria-label={t.models} placeholder={t.manualModelPlaceholder} value={detail.modelDraft} disabled={fieldDisabled} onChange={(event) => props.onModelInput(uuid, event.currentTarget.value)} style={{ flexGrow: 1 }} />
                )}
                {canListModels ? <Button disabled={locked || hasUnsavedDrafts || detail.modelFetch === "loading"} icon={detail.modelFetch === "loading" ? <Spin size="small" /> : undefined} onClick={() => props.onFetchModels(uuid)} data-testid="fetch-models">{detail.modelFetch === "loading" ? t.loadingModels : t.fetchModels}</Button> : null}
              </div>
            }
          />
          <Setting
            label={t.saveProfile}
            control={<div className={styles.rowWrap} style={controlStyle}><Button type="primary" icon={detail.saveState === "saving" ? <Spin size="small" /> : undefined} disabled={detail.saveState === "saving" || isReloading || locked} onClick={() => props.onSaveProfile(uuid)}>{t.saveProfile}</Button>{detail.saveState === "saved" ? <Typography.Text type="success">{t.profileSaved}</Typography.Text> : null}</div>}
          />
        </SettingGroup>
        <SettingGroup title={t.apiKey}>
          <Setting
            label={t.apiKey}
            description={!detail.provider.needs_key ? t.noKeyRequired : detail.provider.hasKey ? t.keySaved : undefined}
            control={
              !detail.provider.needs_key ? <Tag>{t.noKeyRequired}</Tag>
                : detail.provider.hasKey ? <Tag color="success">{t.keySaved}</Tag>
                  : <div className={styles.row} style={controlStyle}><Form.Item validateStatus={detail.keyError ? "error" : undefined} help={detail.keyError || undefined} style={{ flexGrow: 1 }}><Input.Password aria-label={t.apiKey} placeholder={t.apiKeyPlaceholder} value={detail.keyText} disabled={detail.saveState === "saving" || locked} onChange={(event) => props.onKeyInput(uuid, event.currentTarget.value)} /></Form.Item><Button type="primary" disabled={detail.keyText.trim().length === 0 || locked || detail.saveState === "saving"} icon={detail.saveState === "saving" ? <Spin size="small" /> : undefined} onClick={() => props.onSaveKey(uuid)}>{t.saveKey}</Button></div>
            }
          />
        </SettingGroup>
        <SettingGroup title={t.testConnection}>
          <Setting
            label={t.testConnection}
            description={connectionResult ? `${connectionResult.message}${typeof connectionResult.latency_ms === "number" ? ` · ${connectionResult.latency_ms}ms` : ""}` : hasUnsavedDrafts ? t.saveFirstToTest : undefined}
            control={<div className={styles.rowWrap} style={controlStyle}><Button icon={connection === "testing" ? <Spin size="small" /> : undefined} disabled={locked || hasUnsavedDrafts || connection === "testing"} onClick={() => props.onTestConnection(uuid)} data-testid="test-connection">{t.testConnection}</Button>{connectionResult ? <Tag color={connectionResult.ok ? "success" : "error"}>{connectionResult.ok ? t.connectionOk : t.connectionFailed}</Tag> : null}</div>}
          />
          <Setting
            label={t.balance.title}
            description={!detail.provider.capabilities.balance ? t.balance.unsupportedNote : props.balanceText}
            control={detail.provider.capabilities.balance ? <Button type="text" size="small" onClick={() => props.onFetchBalance(uuid)}>{t.balance.fetch}</Button> : <Typography.Text>{t.balance.unsupportedNote}</Typography.Text>}
          />
        </SettingGroup>
      </SettingGroupList>
    </div>
  );
}

export function DetailEmpty({ t }: { t: ProviderCopy }) {
  return <Empty image={<CloudServerOutlined aria-hidden />} description={t.selectPrimary} data-testid="provider-detail-empty" />;
}

export default ProviderDetail;
