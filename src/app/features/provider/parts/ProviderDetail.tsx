import {
  Alert,
  Badge,
  Button,
  Checkbox,
  Group,
  Select,
  Stack,
  Text,
  TextInput,
} from "@mantine/core";
import { Server } from "lucide-react";
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
  const t = props.t;
  const d = props.detail;
  const uuid = d.provider.uuid;
  const locked = props.exclusiveBusy;
  const isReloading = props.reloading;
  const fieldDisabled = d.saveState === "saving" || isReloading || locked;
  const canListModels = d.provider.capabilities.model_list;
  // Test/Fetch probe the BACKEND's stored config — blocked while drafts are
  // unsaved (a no-op edit that equals the stored value does not count).
  const hasUnsavedDrafts =
    d.nameDraft !== d.provider.name ||
    d.endpointDraft !== d.provider.endpoint ||
    d.modelDraft !== (d.provider.model ?? "") ||
    d.keyText.length > 0;

  const selectData =
    d.modelOptions.length > 0
      ? d.modelOptions.map((m) => ({ value: m.id, label: m.label }))
      : [{ value: d.modelDraft || "—", label: d.modelDraft || "—" }];

  const conn = d.conn;
  const connResult: ConnectionResult | null =
    conn !== "testing" && conn !== "idle" ? conn : null;

  return (
    <Stack gap="sm" aria-label={t.detailLabel} data-testid="provider-detail">
      {d.saveConflict && (
        <Alert color="red" title={t.saveConflict} data-testid="save-conflict">
          <Button
            size="xs"
            variant="light"
            color="red"
            loading={isReloading}
            disabled={isReloading || locked}
            onClick={() => props.onResolveSaveConflict(uuid)}
          >
            {t.reload}
          </Button>
        </Alert>
      )}

      <TextInput
        label={t.name}
        value={d.nameDraft}
        error={d.nameError}
        disabled={fieldDisabled}
        onChange={(e) => props.onNameInput(uuid, e.currentTarget.value)}
      />

      <TextInput
        label={t.endpoint.label}
        placeholder={t.endpoint.placeholder}
        value={d.endpointDraft}
        error={d.endpointError}
        disabled={fieldDisabled}
        onChange={(e) => props.onEndpointInput(uuid, e.currentTarget.value)}
      />
      {d.provider.template_id === "azure-openai" && (
        <Button
          variant="subtle" size="compact-sm" disabled={fieldDisabled}
          onClick={() =>
            props.onEndpointInput(uuid, "https://{resource}.openai.azure.com/openai/v1/chat/completions")
          }
        >
          {t.insertAzureTemplate}
        </Button>
      )}
      {d.provider.template_id === "kimi" && (
        <Button
          variant="subtle" size="compact-sm" disabled={fieldDisabled}
          onClick={() => props.onEndpointInput(uuid, "https://api.moonshot.ai/v1/chat/completions")}
        >
          {t.useKimiGlobal}
        </Button>
      )}
      {d.provider.template_id === "custom" && (
        <Checkbox
          label={t.customAnthropic}
          checked={d.provider.protocol === "anthropic"}
          disabled={fieldDisabled}
          onChange={(e) => props.onToggleCustomAnthropic(uuid, e.currentTarget.checked)}
        />
      )}

      {/* Model: dropdown + Fetch when the provider advertises model_list;
          manual entry otherwise (and on fetch error). */}
      {d.modelFetch !== "error" && canListModels ? (
        <Group align="flex-end" gap="xs" wrap="nowrap">
          <Select
            label={t.models}
            style={{ flex: 1 }}
            data={selectData}
            value={d.modelDraft || null}
            disabled={fieldDisabled}
            onChange={(v) => v !== null && props.onModelChange(uuid, v)}
          />
          <Button
            variant="light"
            size="sm"
            disabled={locked || hasUnsavedDrafts}
            onClick={() => props.onFetchModels(uuid)}
            data-testid="fetch-models"
          >
            {d.modelFetch === "loading" ? t.loadingModels : t.fetchModels}
          </Button>
        </Group>
      ) : (
        <TextInput
          label={t.models}
          placeholder={t.manualModelPlaceholder}
          value={d.modelDraft}
          disabled={fieldDisabled}
          onChange={(e) => props.onModelInput(uuid, e.currentTarget.value)}
        />
      )}
      {hasUnsavedDrafts && (
        <Text size="xs" c="dimmed" role="status">{t.saveFirstToFetch}</Text>
      )}

      <Group gap="sm">
        <Button size="sm" loading={d.saveState === "saving"} disabled={isReloading || locked} onClick={() => props.onSaveProfile(uuid)}>
          {t.saveProfile}
        </Button>
        {d.saveState === "saved" && <Text size="sm" c="teal">{t.profileSaved}</Text>}
      </Group>

      {/* Key section — three states: not required / saved / input. */}
      {!d.provider.needs_key ? (
        <Text size="sm" c="dimmed">{t.noKeyRequired}</Text>
      ) : d.provider.hasKey ? (
        <Badge variant="light" color="success">{t.keySaved}</Badge>
      ) : (
        <Group align="flex-start" gap="xs" wrap="nowrap">
          <TextInput
            style={{ flex: 1 }}
            type="password"
            label={t.apiKey}
            placeholder={t.apiKeyPlaceholder}
            value={d.keyText}
            error={d.keyError}
            disabled={d.saveState === "saving" || locked}
            onChange={(e) => props.onKeyInput(uuid, e.currentTarget.value)}
          />
          <Button
            size="sm"
            disabled={d.keyText.trim().length === 0 || locked}
            loading={d.saveState === "saving"}
            onClick={() => props.onSaveKey(uuid)}
          >
            {t.saveKey}
          </Button>
        </Group>
      )}

      {/* Connection test */}
      <Group gap="sm" wrap="nowrap">
        <Button
          variant="light"
          size="sm"
          loading={conn === "testing"}
          disabled={locked || hasUnsavedDrafts}
          onClick={() => props.onTestConnection(uuid)}
          data-testid="test-connection"
        >
          {t.testConnection}
        </Button>
        {connResult && (
          <>
            <Badge variant="light" color={connResult.ok ? "success" : "danger"}>
              {connResult.ok ? t.connectionOk : t.connectionFailed}
            </Badge>
            <Text size="sm" c="dimmed" style={{ minWidth: 0 }}>
              {connResult.message}
              {typeof connResult.latency_ms === "number" && ` · ${connResult.latency_ms}ms`}
            </Text>
          </>
        )}
        {connResult && !connResult.ok && (
          <Text span size="sm" c="dimmed" role="status">{""}</Text>
        )}
      </Group>
      {hasUnsavedDrafts && (
        <Text size="xs" c="dimmed" role="status">{t.saveFirstToTest}</Text>
      )}

      {/* Balance */}
      <Group gap="sm">
        <Text size="sm" fw={500}>{t.balance.title}</Text>
        {d.provider.capabilities.balance ? (
          <>
            <Button variant="subtle" size="compact-sm" onClick={() => props.onFetchBalance(uuid)}>
              {t.balance.fetch}
            </Button>
            {props.balanceText && <Text size="sm" c="dimmed">{props.balanceText}</Text>}
          </>
        ) : (
          <Text size="sm" c="dimmed">{t.balance.unsupportedNote}</Text>
        )}
      </Group>
    </Stack>
  );
}

export function DetailEmpty({ t }: { t: ProviderCopy }) {
  return (
    <Stack align="center" gap="xs" py="xl" data-testid="provider-detail-empty">
      <Server size={28} aria-hidden />
      <Text c="dimmed">{t.selectPrimary}</Text>
    </Stack>
  );
}

export default ProviderDetail;
