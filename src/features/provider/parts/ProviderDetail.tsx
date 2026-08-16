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

/** Detail panel: edit form + model selection + key + connection test + balance. */
export function ProviderDetail(props: ProviderDetailProps) {
  const styles = useUiStyles();
  const t = props.t;
  const d = props.detail;
  const uuid = d.provider.uuid;
  const locked = props.exclusiveBusy;
  const isReloading = props.reloading;
  const fieldDisabled = d.saveState === "saving" || isReloading || locked;
  const canListModels = d.provider.capabilities.model_list;
  const hasUnsavedDrafts =
    d.nameDraft !== d.provider.name ||
    d.endpointDraft !== d.provider.endpoint ||
    d.modelDraft !== (d.provider.model ?? "") ||
    d.keyText.length > 0;
  const modelOptions = d.modelOptions.length > 0
    ? d.modelOptions
    : [{ id: d.modelDraft || "—", label: d.modelDraft || "—" }];
  const conn = d.conn;
  const connResult: ConnectionResult | null = conn !== "testing" && conn !== "idle" ? conn : null;

  return (
    <div className={styles.stack} aria-label={t.detailLabel} data-testid="provider-detail">
      {d.saveConflict && (
        <MessageBar intent="error" data-testid="save-conflict">
          <MessageBarBody><MessageBarTitle>{t.saveConflict}</MessageBarTitle></MessageBarBody>
          <MessageBarActions>
            <Button appearance="secondary" size="small" icon={isReloading ? <Spinner size="tiny" /> : undefined} disabled={isReloading || locked} onClick={() => props.onResolveSaveConflict(uuid)}>{t.reload}</Button>
          </MessageBarActions>
        </MessageBar>
      )}

      <Field label={t.name} validationMessage={d.nameError || undefined} validationState={d.nameError ? "error" : "none"}>
        <Input value={d.nameDraft} disabled={fieldDisabled} onChange={(e) => props.onNameInput(uuid, e.currentTarget.value)} />
      </Field>

      <Field label={t.endpoint.label} validationMessage={d.endpointError || undefined} validationState={d.endpointError ? "error" : "none"}>
        <Input placeholder={t.endpoint.placeholder} value={d.endpointDraft} disabled={fieldDisabled} onChange={(e) => props.onEndpointInput(uuid, e.currentTarget.value)} />
      </Field>
      {d.provider.template_id === "azure-openai" && <Button appearance="subtle" size="small" disabled={fieldDisabled} onClick={() => props.onEndpointInput(uuid, "https://{resource}.openai.azure.com/openai/v1/chat/completions")}>{t.insertAzureTemplate}</Button>}
      {d.provider.template_id === "kimi" && <Button appearance="subtle" size="small" disabled={fieldDisabled} onClick={() => props.onEndpointInput(uuid, "https://api.moonshot.ai/v1/chat/completions")}>{t.useKimiGlobal}</Button>}
      {d.provider.template_id === "custom" && (
        <Checkbox label={t.customAnthropic} checked={d.provider.protocol === "anthropic"} disabled={fieldDisabled} onChange={(_, data) => props.onToggleCustomAnthropic(uuid, Boolean(data.checked))} />
      )}

      {d.modelFetch !== "error" && canListModels ? (
        <div className={styles.row}>
          <Field label={t.models} className={styles.grow}>
            <Select value={d.modelDraft || ""} disabled={fieldDisabled} onChange={(e) => props.onModelChange(uuid, e.currentTarget.value)}>
              {modelOptions.map((model) => <option key={model.id} value={model.id}>{model.label}</option>)}
            </Select>
          </Field>
          <Button appearance="secondary" disabled={locked || hasUnsavedDrafts || d.modelFetch === "loading"} icon={d.modelFetch === "loading" ? <Spinner size="tiny" /> : undefined} onClick={() => props.onFetchModels(uuid)} data-testid="fetch-models">
            {d.modelFetch === "loading" ? t.loadingModels : t.fetchModels}
          </Button>
        </div>
      ) : (
        <Field label={t.models}>
          <Input placeholder={t.manualModelPlaceholder} value={d.modelDraft} disabled={fieldDisabled} onChange={(e) => props.onModelInput(uuid, e.currentTarget.value)} />
        </Field>
      )}
      {hasUnsavedDrafts && <Text size={200} className={styles.muted} role="status">{t.saveFirstToFetch}</Text>}

      <div className={styles.rowWrap}>
        <Button appearance="primary" icon={d.saveState === "saving" ? <Spinner size="tiny" /> : undefined} disabled={d.saveState === "saving" || isReloading || locked} onClick={() => props.onSaveProfile(uuid)}>{t.saveProfile}</Button>
        {d.saveState === "saved" && <Text className={styles.success}>{t.profileSaved}</Text>}
      </div>

      {!d.provider.needs_key ? (
        <Text size={300} className={styles.muted}>{t.noKeyRequired}</Text>
      ) : d.provider.hasKey ? (
        <Badge appearance="tint" color="success">{t.keySaved}</Badge>
      ) : (
        <div className={styles.row}>
          <Field label={t.apiKey} validationMessage={d.keyError || undefined} validationState={d.keyError ? "error" : "none"} className={styles.grow}>
            <Input type="password" placeholder={t.apiKeyPlaceholder} value={d.keyText} disabled={d.saveState === "saving" || locked} onChange={(e) => props.onKeyInput(uuid, e.currentTarget.value)} />
          </Field>
          <Button appearance="primary" disabled={d.keyText.trim().length === 0 || locked || d.saveState === "saving"} icon={d.saveState === "saving" ? <Spinner size="tiny" /> : undefined} onClick={() => props.onSaveKey(uuid)}>{t.saveKey}</Button>
        </div>
      )}

      <div className={styles.rowWrap}>
        <Button appearance="secondary" icon={conn === "testing" ? <Spinner size="tiny" /> : undefined} disabled={locked || hasUnsavedDrafts || conn === "testing"} onClick={() => props.onTestConnection(uuid)} data-testid="test-connection">{t.testConnection}</Button>
        {connResult && (
          <>
            <Badge appearance="tint" color={connResult.ok ? "success" : "danger"}>{connResult.ok ? t.connectionOk : t.connectionFailed}</Badge>
            <Text size={300} className={styles.muted}>{connResult.message}{typeof connResult.latency_ms === "number" && ` · ${connResult.latency_ms}ms`}</Text>
          </>
        )}
      </div>
      {hasUnsavedDrafts && <Text size={200} className={styles.muted} role="status">{t.saveFirstToTest}</Text>}

      <div className={styles.rowWrap}>
        <Text weight="semibold">{t.balance.title}</Text>
        {d.provider.capabilities.balance ? (
          <><Button appearance="subtle" size="small" onClick={() => props.onFetchBalance(uuid)}>{t.balance.fetch}</Button>{props.balanceText && <Text size={300} className={styles.muted}>{props.balanceText}</Text>}</>
        ) : <Text size={300} className={styles.muted}>{t.balance.unsupportedNote}</Text>}
      </div>
    </div>
  );
}

export function DetailEmpty({ t }: { t: ProviderCopy }) {
  const styles = useUiStyles();
  return <div className={styles.empty} data-testid="provider-detail-empty"><ServerRegular fontSize={28} aria-hidden /><Text>{t.selectPrimary}</Text></div>;
}

export default ProviderDetail;
