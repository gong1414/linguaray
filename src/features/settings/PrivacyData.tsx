import { createSignal, For, onMount, Show, type Component } from "solid-js";
import { Banner, Button, Confirm, Select, Switch, Toast } from "@linguaray/ui";
import { invoke } from "../../bridge/invoke";
import { detectLocale } from "../../i18n";
import { PRIVACY_COPY } from "./privacy-copy";
import {
  historyClearAll,
  historyPrivacyStatus,
  historySetEnabled,
  historySetRetention,
} from "./privacy-ipc";
import type { HistoryPrivacyStatus, HistoryRetentionDays } from "./privacy-types";
import "./PrivacyData.css";

type ToastEntry = { id: number; variant: "success" | "destructive"; message: string };

export type PrivacyDataViewProps = {
  status: HistoryPrivacyStatus | null;
  loading: boolean;
  error: string | null;
  busy: "enabled" | "retention" | "clear" | null;
  clearOpen: boolean;
  toasts: ToastEntry[];
  onRetry: () => void;
  onEnabledChange: (enabled: boolean) => void;
  onRetentionChange: (days: HistoryRetentionDays) => void;
  onOpenClear: () => void;
  onCloseClear: () => void;
  onConfirmClear: () => void;
  onDismissToast: (id: number) => void;
};

const ExternalApiControls: Component<{ t: typeof PRIVACY_COPY["en"] }> = (props) => {
  const [status, setStatus] = createSignal<{ state: string; port?: number } | null>(null);
  const [token, setToken] = createSignal("");
  const refresh = async () => {
    const s = await invoke<{ state: string; port?: number }>("external_api_status");
    setStatus(s);
  };
  onMount(() => {
    void refresh().catch(() => setStatus({ state: "disabled" }));
  });
  return (
    <div>
      <p>
        {status()?.state === "enabled"
          ? props.t.externalOn.replace("{port}", String(status()?.port ?? ""))
          : props.t.externalOff}
      </p>
      <Button
        onClick={() => {
          void invoke<string>("external_api_enable", { port: null })
            .then((tok) => {
              setToken(tok);
              return refresh();
            })
            .catch(() => refresh());
        }}
      >
        {props.t.externalEnable}
      </Button>
      <Button
        variant="ghost"
        onClick={() => {
          void invoke("external_api_disable").then(() => {
            setToken("");
            return refresh();
          });
        }}
      >
        {props.t.externalDisable}
      </Button>
      <Button
        variant="ghost"
        onClick={() => {
          void invoke<string>("external_api_regenerate_token").then(setToken);
        }}
      >
        {props.t.externalRegen}
      </Button>
      <Show when={token()}>
        <p role="status">
          {props.t.externalTokenOnce}: <code>{token()}</code>
        </p>
      </Show>
    </div>
  );
};

export const PrivacyDataView: Component<PrivacyDataViewProps> = (props) => {
  const t = PRIVACY_COPY[detectLocale()];
  const countCopy = () =>
    t.records.replace("{count}", String(props.status?.record_count ?? 0));

  return (
    <section class="privacy-data" aria-label={t.title} aria-busy={props.loading ? "true" : undefined}>
      <header class="privacy-data__header">
        <h1>{t.title}</h1>
      </header>

      <Show when={props.error}>
        <Banner
          variant="destructive"
          title={t.loadFailed}
          description={props.error ?? undefined}
          action={<Button variant="secondary" size="sm" onClick={props.onRetry}>{t.retry}</Button>}
        />
      </Show>

      <Show when={!props.error && props.status}>
        <div class="privacy-data__panel" data-testid="history-panel">
          <div class="privacy-data__panel-heading">
            <div>
              <h2>{t.historyTitle}</h2>
              <p>{props.status?.enabled ? t.historyEnabledNotice : t.historyDisabledNotice}</p>
            </div>
            <Switch
              checked={props.status?.enabled ?? false}
              label={t.historyEnable}
              disabled={props.busy !== null}
              onChange={props.onEnabledChange}
            />
          </div>

          <div class="privacy-data__controls">
            <Select
              label={t.retention}
              value={String(props.status?.retention_days ?? 30)}
              options={[
                { value: "30", label: t.retention30, disabled: false },
                { value: "90", label: t.retention90, disabled: false },
              ]}
              disabled={!props.status?.enabled || props.busy !== null}
              loading={props.busy === "retention"}
              loadingLabel={t.loading}
              onChange={(value) => props.onRetentionChange(Number(value) as HistoryRetentionDays)}
            />
            <div class="privacy-data__clear">
              <span class="privacy-data__record-count">{countCopy()}</span>
              <Button
                variant="destructive"
                size="md"
                disabled={(props.status?.record_count ?? 0) === 0 || props.busy !== null}
                loading={props.busy === "clear"}
                loadingLabel={t.loading}
                onClick={props.onOpenClear}
              >
                {t.clearAll}
              </Button>
            </div>
          </div>
        </div>

        <div class="privacy-data__panel" aria-labelledby="privacy-external-title">
          <h2 id="privacy-external-title">{t.externalTitle}</h2>
          <p>{t.externalDeferred}</p>
        </div>
      </Show>

      <Confirm
        open={props.clearOpen}
        onOpenChange={(open) => (open ? props.onOpenClear() : props.onCloseClear())}
        variant="destructive"
        title={t.clearConfirmTitle}
        message={t.clearConfirmMessage}
        confirmLabel={t.clearAll}
        cancelLabel={t.cancel}
        onConfirm={props.onConfirmClear}
        onCancel={props.onCloseClear}
      />

      <Show when={props.toasts.length > 0}>
        <div class="privacy-data__toasts" aria-live="polite">
          <For each={props.toasts}>{(entry) => (
            <Toast
              variant={entry.variant}
              message={entry.message}
              onDismiss={() => props.onDismissToast(entry.id)}
            />
          )}</For>
        </div>
      </Show>
    </section>
  );
};

const PrivacyData: Component = () => {
  const t = PRIVACY_COPY[detectLocale()];
  const [status, setStatus] = createSignal<HistoryPrivacyStatus | null>(null);
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  const [busy, setBusy] = createSignal<PrivacyDataViewProps["busy"]>(null);
  const [clearOpen, setClearOpen] = createSignal(false);
  const [toasts, setToasts] = createSignal<ToastEntry[]>([]);
  let requestEpoch = 0;
  let toastId = 0;

  const pushToast = (variant: ToastEntry["variant"], message: string) => {
    setToasts((items) => [...items, { id: ++toastId, variant, message }]);
  };

  const load = async () => {
    const epoch = ++requestEpoch;
    setLoading(true);
    setError(null);
    try {
      const next = await historyPrivacyStatus();
      if (epoch === requestEpoch) setStatus(next);
    } catch (reason) {
      if (epoch === requestEpoch) setError(reason instanceof Error ? reason.message : String(reason));
    } finally {
      if (epoch === requestEpoch) setLoading(false);
    }
  };

  const mutate = async (
    kind: NonNullable<PrivacyDataViewProps["busy"]>,
    operation: () => Promise<HistoryPrivacyStatus>,
  ) => {
    if (busy() !== null) return;
    const epoch = ++requestEpoch;
    setBusy(kind);
    try {
      const next = await operation();
      if (epoch === requestEpoch) setStatus(next);
    } catch (_reason) {
      if (epoch === requestEpoch) pushToast("destructive", t.updateFailed);
    } finally {
      if (epoch === requestEpoch) setBusy(null);
    }
  };

  onMount(() => void load());

  return (
    <>
    <PrivacyDataView
      status={status()}
      loading={loading()}
      error={error()}
      busy={busy()}
      clearOpen={clearOpen()}
      toasts={toasts()}
      onRetry={() => void load()}
      onEnabledChange={(enabled) => void mutate("enabled", () => historySetEnabled(enabled))}
      onRetentionChange={(days) => void mutate("retention", () => historySetRetention(days))}
      onOpenClear={() => setClearOpen(true)}
      onCloseClear={() => setClearOpen(false)}
      onConfirmClear={() => {
        setClearOpen(false);
        void mutate("clear", async () => {
          const next = await historyClearAll();
          pushToast("success", t.cleared);
          return next;
        });
      }}
      onDismissToast={(id) => setToasts((items) => items.filter((item) => item.id !== id))}
    />
    <Show when={!loading() && !error()}>
      <section aria-label={t.externalTitle}>
        <ExternalApiControls t={t} />
      </section>
    </Show>
    </>
  );
};

export default PrivacyData;
