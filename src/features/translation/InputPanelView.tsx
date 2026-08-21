import { useMemo, useRef, useState } from "react";
import type { GetRef } from "antd";
import { App, Button, Typography } from "antd";
import {
  DeleteOutlined,
  MessageOutlined,
  PlusOutlined,
  SendOutlined,
  TranslationOutlined,
} from "@ant-design/icons";
import { Bubble, Conversations, Prompts, Sender, Suggestion, Welcome } from "@ant-design/x";
import type { BubbleListProps } from "@ant-design/x";
import {
  useXChat,
  useXConversations,
  type DefaultMessageInfo,
} from "@ant-design/x-sdk";
import logo from "../../assets/logo.svg";
import { t } from "../../app/i18n";
import { TranslationResultSurface, headlineForTranslation } from "./TranslationResultSurface";
import {
  createTranslationChatProvider,
  type TranslationChatInput,
  type TranslationChatMessage,
  type TranslationChatOutput,
} from "./translationXProvider";
import type { InputController } from "./inputController";

function initialMessages(c: InputController): DefaultMessageInfo<TranslationChatMessage>[] {
  if ((!c.hasResult && c.idle) || !c.text.trim()) return [];
  return [
    {
      id: "initial-source",
      status: "success",
      message: { role: "user", content: c.text, source: c.text },
    },
    {
      id: "initial-result",
      status: c.idle ? "success" : "loading",
      message: {
        role: "assistant",
        content: "",
        source: c.text,
        state: c.idle ? c.state : { kind: "loading" },
      },
    },
  ];
}

/** Ant Design X Ultramodern workspace backed by the existing translation controller. */
export function InputPanelView({ c }: { c: InputController }) {
  const { message: messageApi } = App.useApp();
  const initialRef = useRef(initialMessages(c));
  const [firstConversationKey] = useState(
    () => `translation-${Date.now()}-${Math.random().toString(36).slice(2)}`,
  );
  const translateRef = useRef(c.translate);
  translateRef.current = c.translate;

  const [providerCache] = useState(
    () => new Map<string, ReturnType<typeof createTranslationChatProvider>>(),
  );
  const {
    conversations,
    activeConversationKey,
    setActiveConversationKey,
    addConversation,
    removeConversation,
    setConversation,
  } = useXConversations({
    defaultConversations: [{ key: firstConversationKey, label: t("input.title"), group: "Today" }],
    defaultActiveConversationKey: firstConversationKey,
  });
  if (!providerCache.has(activeConversationKey)) {
    providerCache.set(
      activeConversationKey,
      createTranslationChatProvider((query) => translateRef.current(query)),
    );
  }
  const provider = providerCache.get(activeConversationKey)!;

  const { messages, onRequest, isRequesting, abort, setMessages } = useXChat<
    TranslationChatMessage,
    TranslationChatMessage,
    TranslationChatInput,
    TranslationChatOutput
  >({
    provider,
    conversationKey: activeConversationKey,
    defaultMessages: (info?: { conversationKey?: string }) =>
      info?.conversationKey === firstConversationKey ? initialRef.current : [],
    requestPlaceholder: (params) => ({
      role: "assistant",
      content: "",
      source: params.query ?? "",
      state: { kind: "loading" },
    }),
    requestFallback: (params, { error }) => ({
      role: "assistant",
      content: error.message,
      source: params.query ?? "",
      state: { kind: "error", sub: "generic", message: error.message },
    }),
  });

  const createConversation = () => {
    if (messages.length === 0) {
      void messageApi.info("This is already a new translation");
      return;
    }
    const key = `translation-${Date.now()}`;
    addConversation({ key, label: "New translation", group: "Today" }, "prepend");
    setActiveConversationKey(key);
    c.clear();
  };

  const remove = (key: string) => {
    if (conversations.length === 1) {
      setMessages([]);
      c.clear();
      return;
    }
    const remaining = conversations.filter((item) => item.key !== key);
    removeConversation(key);
    providerCache.delete(key);
    if (activeConversationKey === key && remaining[0]) setActiveConversationKey(remaining[0].key);
  };

  const role = useMemo<BubbleListProps["role"]>(() => ({
    assistant: {
      placement: "start",
      variant: "borderless",
      contentRender: (content, info) => {
        const message = content as TranslationChatMessage;
        if (!message.state) return message.content;
        return (
          <TranslationResultSurface
            state={message.state}
            source={message.source}
            testId="input-result"
            errorTestId="input-error"
            surfaceId={`input-${String(info.key)}`}
            actions={{
              copy: c.copyText,
              favorite: (source, text, key) => c.favorite(source, text, key),
            }}
          />
        );
      },
    },
    user: {
      placement: "end",
      variant: "filled",
      contentRender: (content) => {
        const message = content as TranslationChatMessage;
        return <Typography.Text>{message.content}</Typography.Text>;
      },
    },
  }), [c.copyText, c.favorite]);

  const bindSenderRef = (ref: GetRef<typeof Sender> | null) => {
    c.textareaRef.current = (ref?.inputElement as HTMLTextAreaElement | undefined) ?? null;
    ref?.inputElement?.setAttribute("aria-label", t("input.title"));
  };

  const submit = (value: string) => {
    const query = value.trim();
    if (!query || isRequesting) return;
    onRequest({ query });
    setConversation(activeConversationKey, {
      key: activeConversationKey,
      label: query.length > 28 ? `${query.slice(0, 28)}…` : query,
      group: "Today",
    });
    c.setText("");
  };

  const clearCurrent = () => {
    abort();
    setMessages([]);
    c.clear();
  };

  return (
    <main className="lr-ultra-workspace" data-testid="input-panel">
      <aside className="lr-ultra-sidebar">
        <div className="lr-ultra-brand" data-tauri-drag-region>
          <img src={logo} alt="" width={28} height={28} draggable={false} />
          <Typography.Text strong>LinguaRay</Typography.Text>
        </div>
        <Button
          className="lr-ultra-new-translation"
          icon={<PlusOutlined aria-hidden />}
          onClick={createConversation}
        >
          New translation
        </Button>
        <Conversations
          rootClassName="lr-ultra-conversations"
          items={conversations.map((item) => ({ ...item, icon: <MessageOutlined aria-hidden /> }))}
          activeKey={activeConversationKey}
          onActiveChange={setActiveConversationKey}
          groupable
          menu={(conversation) => ({
            items: [{ key: "delete", label: "Delete", icon: <DeleteOutlined aria-hidden />, danger: true }],
            onClick: ({ key }) => { if (key === "delete") remove(conversation.key); },
          })}
        />
        <div className="lr-ultra-sidebar-footer">
          <TranslationOutlined aria-hidden />
          <Typography.Text type="secondary">Local-first workspace</Typography.Text>
        </div>
      </aside>

      <section className="lr-ultra-chat" aria-label={t("input.title")}>
        <div className="lr-ultra-chat-list">
          {messages.length > 0 ? (
            <Bubble.List
              role={role}
              items={messages.map(({ id, message, status }) => ({
                key: id,
                role: message.role,
                content: message,
                status,
                loading: status === "loading" && !message.state,
              }))}
              autoScroll
            />
          ) : (
            <div className="lr-ultra-start">
              <Welcome
                icon={<img src={logo} alt="" width={42} height={42} />}
                title="Translate with LinguaRay"
                description="Private translation with your active local and cloud providers."
                variant="borderless"
              />
              <Prompts
                title="Try a phrase"
                wrap
                items={[
                  { key: "hello", label: "Hello, how are you?", description: "English → your language" },
                  { key: "zh", label: "今天是个适合出发的日子。", description: "中文 → your language" },
                  { key: "mail", label: "Could you send me the final draft?", description: "Work message" },
                ]}
                onItemClick={({ data }) => c.setText(String(data.label ?? ""))}
              />
            </div>
          )}
        </div>

        <div className={messages.length === 0 ? "lr-ultra-composer lr-ultra-composer-start" : "lr-ultra-composer"}>
          <Suggestion
            block
            items={[
              { value: "/clear", label: "/clear", extra: "Clear this translation" },
              { value: "/new", label: "/new", extra: "Start a new translation" },
            ]}
            onSelect={(value) => {
              if (value === "/clear") clearCurrent();
              if (value === "/new") createConversation();
            }}
          >
            {({ onTrigger, onKeyDown }) => (
              <Sender
                ref={bindSenderRef}
                rootClassName="lr-ultra-sender"
                value={c.text}
                placeholder={t("input.placeholder")}
                loading={isRequesting}
                autoSize={{ minRows: 2, maxRows: 6 }}
                submitType="enter"
                onChange={(value) => {
                  c.setText(value);
                  onTrigger(value.startsWith("/") ? {} : false);
                }}
                onKeyDown={onKeyDown}
                onCancel={abort}
                onSubmit={submit}
                prefix={
                  <Button
                    type="text"
                    size="small"
                    icon={<DeleteOutlined aria-hidden />}
                    onClick={clearCurrent}
                    disabled={messages.length === 0 && !c.text.trim()}
                  >
                    {t("input.action.clear")}
                  </Button>
                }
                suffix={(_, { components }) => {
                  const SendButton = components.SendButton;
                  return (
                    <SendButton
                      type="primary"
                      shape="default"
                      icon={<SendOutlined aria-hidden />}
                      disabled={isRequesting || !c.text.trim()}
                    >
                      {t("input.action.translate")}
                    </SendButton>
                  );
                }}
                footer={() => (
                  <Typography.Text type="secondary">/ for commands · Enter to translate</Typography.Text>
                )}
              />
            )}
          </Suggestion>
        </div>
      </section>
    </main>
  );
}

export { headlineForTranslation as errorMessageFor };
export default InputPanelView;
