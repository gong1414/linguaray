import { createContext, useContext, useMemo, useState, type ComponentProps, type ReactNode } from "react";
import {
  CheckOutlined,
  CopyOutlined,
  PushpinFilled,
  PushpinOutlined,
  RedoOutlined,
  SettingOutlined,
  SoundFilled,
  SoundOutlined,
  StarFilled,
  StarOutlined,
} from "@ant-design/icons";
import { Actions, ThoughtChain } from "@ant-design/x";
import XMarkdown from "@ant-design/x-markdown";
import {
  XCard,
  registerCatalog,
  type ActionPayload,
  type Catalog,
  type XAgentCommand_v0_9,
} from "@ant-design/x-card";
import { Alert, Button, Card, Result, Tag, Typography } from "antd";
import { useAppColorScheme } from "../../app/providers";
import { t } from "../../app/i18n";
import { engineLabel as defaultEngineLabel } from "./providerNames";
import type { ResultEntry, TranslationState } from "./types";

const CATALOG_ID = "local://linguaray/translation-result-v1.json";

const translationCatalog: Catalog = {
  catalogId: CATALOG_ID,
  components: {
    ResultStack: {
      type: "object",
      properties: { children: {} },
    },
    TranslationResult: {
      type: "object",
      properties: {
        resultKey: { type: "string" },
        engine: { type: "string" },
        text: { type: "string" },
        ok: { type: "boolean" },
        source: { type: "string" },
        testId: { type: "string" },
        action: {},
      },
      required: ["resultKey", "engine", "text", "ok"],
    },
    TranslationProgress: {
      type: "object",
      properties: { source: { type: "string" } },
    },
    TranslationError: {
      type: "object",
      properties: {
        title: { type: "string" },
        kind: { type: "string" },
        canRetry: { type: "boolean" },
        testId: { type: "string" },
        action: {},
      },
      required: ["title", "kind"],
    },
  },
};

registerCatalog(translationCatalog);

type ComponentAction = (name: string, context: Record<string, unknown>) => void;

type SurfaceActions = {
  copy?: (text: string) => Promise<void>;
  favorite?: (source: string, text: string, key: string) => Promise<void>;
  speak?: (text: string) => Promise<void>;
  stopSpeaking?: () => Promise<void>;
  togglePin?: () => void;
  retry?: () => void;
  openSettings?: (section: string) => Promise<void>;
};

type RuntimeState = {
  actions: SurfaceActions;
  copiedKey: string | null;
  favoritedKey: string | null;
  speakingKey: string | null;
  pinned: boolean;
  setCopiedKey: (key: string | null) => void;
  setFavoritedKey: (key: string | null) => void;
  setSpeakingKey: (key: string | null) => void;
};

const SurfaceRuntime = createContext<RuntimeState | null>(null);

function ResultStack({ children }: { children?: ReactNode }) {
  return <div className="lr-a2ui-result-stack">{children}</div>;
}

function TranslationResult({
  resultKey,
  engine,
  text,
  ok,
  source,
  testId,
  onAction,
}: {
  resultKey: string;
  engine: string;
  text: string;
  ok: boolean;
  source?: string;
  testId?: string;
  onAction?: ComponentAction;
}) {
  const runtime = useContext(SurfaceRuntime);
  const colorScheme = useAppColorScheme();
  if (!runtime) return null;
  const copied = runtime.copiedKey === resultKey;
  const favorited = runtime.favoritedKey === resultKey;
  const speaking = runtime.speakingKey === resultKey;
  const items = [
    runtime.actions.copy ? {
      key: "copy",
      actionRender: () => (
        <Button
          type="text"
          size="small"
          icon={copied ? <CheckOutlined aria-hidden /> : <CopyOutlined aria-hidden />}
          onClick={() => onAction?.("copy", { resultKey, text })}
        >
          {copied ? t("selection.action.copied") : t("selection.action.copy")}
        </Button>
      ),
    } : null,
    runtime.actions.speak ? {
      key: "speak",
      actionRender: () => (
        <Button
          type="text"
          size="small"
          icon={speaking ? <SoundFilled aria-hidden /> : <SoundOutlined aria-hidden />}
          onClick={() => onAction?.(speaking ? "stop-speaking" : "speak", { resultKey, text })}
        >
          {speaking ? t("selection.action.stop") : t("selection.action.speak")}
        </Button>
      ),
    } : null,
    runtime.actions.togglePin ? {
      key: "pin",
      actionRender: () => (
        <Button
          type="text"
          size="small"
          icon={runtime.pinned ? <PushpinFilled aria-hidden /> : <PushpinOutlined aria-hidden />}
          onClick={() => onAction?.("toggle-pin", { resultKey })}
        >
          {runtime.pinned ? t("selection.action.unpin") : t("selection.action.pin")}
        </Button>
      ),
    } : null,
    runtime.actions.favorite && ok ? {
      key: "favorite",
      actionRender: () => (
        <Button
          type="text"
          size="small"
          icon={favorited ? <StarFilled aria-hidden /> : <StarOutlined aria-hidden />}
          onClick={() => onAction?.("favorite", { resultKey, text, source: source ?? "" })}
        >
          {favorited ? t("selection.action.favorited") : t("selection.action.favorite")}
        </Button>
      ),
    } : null,
  ].filter(Boolean) as NonNullable<ComponentProps<typeof Actions>["items"]>;

  return (
    <Card
      size="small"
      className={ok ? "lr-a2ui-result" : "lr-a2ui-result lr-a2ui-result-error"}
      data-testid={testId}
      title={<Typography.Text strong>{engine}</Typography.Text>}
      extra={<Tag color={ok ? "success" : "error"}>{ok ? "Done" : "Error"}</Tag>}
    >
      {ok ? (
        <XMarkdown
          content={text}
          className={colorScheme === "dark" ? "x-markdown-dark" : "x-markdown-light"}
          openLinksInNewTab
          escapeRawHtml
        />
      ) : (
        <Alert type="error" showIcon title={text} />
      )}
      {items.length > 0 ? <Actions items={items} variant="borderless" /> : null}
    </Card>
  );
}

function TranslationProgress({ source }: { source?: string }) {
  return (
    <div className="lr-a2ui-progress" data-testid="translation-progress">
      <ThoughtChain
        items={[
          { key: "capture", title: "Capture source", description: source || "Waiting for text", status: "success" },
          { key: "route", title: "Route providers", description: "Use active translation providers", status: "success" },
          { key: "translate", title: t("selection.loading"), status: "loading", blink: true },
        ]}
      />
    </div>
  );
}

function TranslationError({
  title,
  kind,
  canRetry,
  testId,
  onAction,
}: {
  title: string;
  kind: string;
  canRetry?: boolean;
  testId?: string;
  onAction?: ComponentAction;
}) {
  const runtime = useContext(SurfaceRuntime);
  const buttons = [
    canRetry && runtime?.actions.retry ? (
      <Button key="retry" icon={<RedoOutlined aria-hidden />} onClick={() => onAction?.("retry", {})}>
        {t("selection.action.retry")}
      </Button>
    ) : null,
    (kind === "config-key" || kind === "config-401" || kind === "no-provider") && runtime?.actions.openSettings ? (
      <Button key="settings" type="primary" icon={<SettingOutlined aria-hidden />} onClick={() => onAction?.("open-settings", { section: "provider-center" })}>
        {t("selection.action.openSettings")}
      </Button>
    ) : null,
    kind === "keystore-corrupt" && runtime?.actions.openSettings ? (
      <Button key="recovery" type="primary" icon={<SettingOutlined aria-hidden />} onClick={() => onAction?.("open-settings", { section: "keystore-recovery" })}>
        {t("selection.action.recovery")}
      </Button>
    ) : null,
  ].filter(Boolean);
  return <div data-testid={testId}><Result status="error" title={title} extra={buttons} /></div>;
}

const A2UI_COMPONENTS = {
  ResultStack,
  TranslationResult,
  TranslationProgress,
  TranslationError,
};

export function headlineForTranslation(state: TranslationState): string {
  switch (state.kind) {
    case "loading": return t("selection.loading");
    case "single-success":
    case "multi-success":
    case "partial": return t("selection.multi.title");
    case "error":
      return state.sub === "network" ? t("selection.error.network")
        : state.sub === "config-key" ? t("selection.error.config.key")
          : state.sub === "config-401" ? t("selection.error.config.auth") : state.message;
    case "offline": return t("selection.error.offline");
    case "no-selection": return t("selection.error.noSelection");
    case "no-permission": return t("selection.error.noPermission");
    case "no-provider": return t("selection.error.noProvider");
    case "keystore-corrupt": return t("selection.error.keystore");
  }
}

function resultsFor(state: TranslationState): ResultEntry[] {
  if (state.kind === "single-success") {
    return [{ uuid: "single", engine: state.engine, text: state.text, ok: true }];
  }
  if (state.kind === "multi-success" || state.kind === "partial") return state.results;
  return [];
}

function buildCommands(
  surfaceId: string,
  state: TranslationState,
  source: string,
  testId: string,
  resolveEngine: (engine: string) => string,
  canRetry: boolean,
  errorTestId: string,
): XAgentCommand_v0_9[] {
  const commands: XAgentCommand_v0_9[] = [
    { version: "v0.9", createSurface: { surfaceId, catalogId: CATALOG_ID } },
  ];
  if (state.kind === "loading") {
    commands.push({
      version: "v0.9",
      updateComponents: {
        surfaceId,
        components: [{ id: "root", component: "TranslationProgress", source: { path: "/source" } }],
      },
    });
    commands.push({ version: "v0.9", updateDataModel: { surfaceId, path: "/source", value: source } });
    return commands;
  }

  const results = resultsFor(state);
  if (results.length > 0) {
    const children = results.map((_, index) => `result-${index}`);
    commands.push({
      version: "v0.9",
      updateComponents: {
        surfaceId,
        components: [
          { id: "root", component: "ResultStack", children },
          ...results.map((_, index) => ({
            id: `result-${index}`,
            component: "TranslationResult",
            resultKey: { path: `/results/${index}/resultKey` },
            engine: { path: `/results/${index}/engine` },
            text: { path: `/results/${index}/text` },
            ok: { path: `/results/${index}/ok` },
            source: { path: "/source" },
            testId,
            action: {
              event: {
                name: "translation-result-action",
                context: {
                  resultKey: { path: `/results/${index}/resultKey` },
                  text: { path: `/results/${index}/text` },
                  source: { path: "/source" },
                },
              },
            },
          })),
        ],
      },
    });
    commands.push({
      version: "v0.9",
      updateDataModel: {
        surfaceId,
        path: "/results",
        value: results.map((result) => ({
          resultKey: result.uuid,
          engine: resolveEngine(result.engine),
          text: result.text ?? result.errorText ?? "",
          ok: result.ok,
        })),
      },
    });
    commands.push({ version: "v0.9", updateDataModel: { surfaceId, path: "/source", value: source } });
    return commands;
  }

  const kind = state.kind === "error" ? state.sub : state.kind;
  commands.push({
    version: "v0.9",
    updateComponents: {
      surfaceId,
      components: [{
        id: "root",
        component: "TranslationError",
        title: headlineForTranslation(state),
        kind,
        canRetry,
        testId: errorTestId,
        action: { event: { name: "translation-error-action", context: {} } },
      }],
    },
  });
  return commands;
}

function contextValue(value: unknown): string {
  if (typeof value === "string") return value;
  if (value && typeof value === "object" && "value" in value) return String((value as { value: unknown }).value ?? "");
  return "";
}

export function TranslationResultSurface({
  state,
  source = "",
  testId,
  surfaceId,
  actions = {},
  pinned = false,
  engineLabel = defaultEngineLabel,
  errorTestId = "translation-error",
}: {
  state: TranslationState;
  source?: string;
  testId: "input-result" | "popup-card";
  surfaceId: string;
  actions?: SurfaceActions;
  pinned?: boolean;
  engineLabel?: (engine: string) => string;
  errorTestId?: string;
}) {
  const [copiedKey, setCopiedKey] = useState<string | null>(null);
  const [favoritedKey, setFavoritedKey] = useState<string | null>(null);
  const [speakingKey, setSpeakingKey] = useState<string | null>(null);
  const commands = useMemo(
    () => buildCommands(surfaceId, state, source, testId, engineLabel, Boolean(source), errorTestId),
    [engineLabel, errorTestId, source, state, surfaceId, testId],
  );
  const runtime = useMemo<RuntimeState>(() => ({
    actions,
    copiedKey,
    favoritedKey,
    speakingKey,
    pinned,
    setCopiedKey,
    setFavoritedKey,
    setSpeakingKey,
  }), [actions, copiedKey, favoritedKey, pinned, speakingKey]);
  const cardKey = useMemo(() => JSON.stringify(state), [state]);

  const onAction = (payload: ActionPayload) => {
    const key = contextValue(payload.context.resultKey);
    const text = contextValue(payload.context.text);
    const actionSource = contextValue(payload.context.source) || source;
    if (payload.name === "copy" && actions.copy) {
      void actions.copy(text).then(() => setCopiedKey(key));
    } else if (payload.name === "favorite" && actions.favorite) {
      void actions.favorite(actionSource, text, key).then(() => setFavoritedKey(key));
    } else if (payload.name === "speak" && actions.speak) {
      void actions.speak(text).then(() => setSpeakingKey(key));
    } else if (payload.name === "stop-speaking" && actions.stopSpeaking) {
      void actions.stopSpeaking().then(() => setSpeakingKey(null));
    } else if (payload.name === "toggle-pin") {
      actions.togglePin?.();
    } else if (payload.name === "retry") {
      actions.retry?.();
    } else if (payload.name === "open-settings" && actions.openSettings) {
      void actions.openSettings(contextValue(payload.context.section));
    }
  };

  return (
    <SurfaceRuntime.Provider value={runtime}>
      <XCard.Box key={cardKey} commands={commands} components={A2UI_COMPONENTS} onAction={onAction}>
        <XCard.Card id={surfaceId} />
      </XCard.Box>
    </SurfaceRuntime.Provider>
  );
}
