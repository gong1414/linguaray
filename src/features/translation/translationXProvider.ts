import {
  AbstractChatProvider,
  XRequest,
  type TransformMessage,
  type XRequestOptions,
} from "@ant-design/x-sdk";
import type { TranslationState } from "./types";

export type TranslationChatInput = {
  query: string;
};

export type TranslationChatOutput = {
  source: string;
  state: TranslationState;
};

export type TranslationChatMessage = {
  role: "user" | "assistant";
  content: string;
  source?: string;
  state?: TranslationState;
};

type TranslateBridge = (query: string) => Promise<TranslationState>;

function textForState(state: TranslationState): string {
  if (state.kind === "single-success") return state.text;
  if (state.kind === "multi-success" || state.kind === "partial") {
    return state.results.map((result) => result.text ?? result.errorText ?? "").join("\n\n");
  }
  if (state.kind === "error" || state.kind === "offline" || state.kind === "keystore-corrupt") {
    return state.message;
  }
  return "";
}

/**
 * Adapts the existing Tauri translation IPC to XRequest's fetch contract.
 * No provider secret or network credential crosses into the webview.
 */
function createTauriFetch(translate: TranslateBridge): NonNullable<
  XRequestOptions<TranslationChatInput, TranslationChatOutput, TranslationChatMessage>["fetch"]
> {
  return async (_baseURL, options) => {
    if (options.signal?.aborted) throw new DOMException("Aborted", "AbortError");
    const body = typeof options.body === "string" ? JSON.parse(options.body) as Partial<TranslationChatInput> : {};
    const query = body.query?.trim() ?? "";
    const state = await translate(query);
    return new Response(JSON.stringify({ source: query, state } satisfies TranslationChatOutput), {
      status: 200,
      headers: { "Content-Type": "application/json" },
    });
  };
}

/** X Skill-compliant provider: only the three required transformation methods. */
class TranslationChatProvider extends AbstractChatProvider<
  TranslationChatMessage,
  TranslationChatInput,
  TranslationChatOutput
> {
  transformParams(
    requestParams: Partial<TranslationChatInput>,
    options: XRequestOptions<TranslationChatInput, TranslationChatOutput, TranslationChatMessage>,
  ): TranslationChatInput {
    return {
      ...(options.params ?? {}),
      query: requestParams.query?.trim() ?? "",
    };
  }

  transformLocalMessage(requestParams: Partial<TranslationChatInput>): TranslationChatMessage {
    return {
      role: "user",
      content: requestParams.query?.trim() ?? "",
      source: requestParams.query?.trim() ?? "",
    };
  }

  transformMessage(
    info: TransformMessage<TranslationChatMessage, TranslationChatOutput>,
  ): TranslationChatMessage {
    const output = info.chunk ?? info.chunks[info.chunks.length - 1];
    if (!output) return info.originMessage ?? { role: "assistant", content: "" };
    return {
      role: "assistant",
      content: textForState(output.state),
      source: output.source,
      state: output.state,
    };
  }
}

export function createTranslationChatProvider(translate: TranslateBridge) {
  return new TranslationChatProvider({
    request: XRequest<TranslationChatInput, TranslationChatOutput, TranslationChatMessage>(
      "tauri://linguaray/translate-session",
      {
        manual: true,
        fetch: createTauriFetch(translate),
      },
    ),
  });
}
