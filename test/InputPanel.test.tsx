import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, fireEvent, cleanup, waitFor } from "@solidjs/testing-library";
import InputPanel from "../src/InputPanel";
import { resetProviderNameMap } from "../src/features/translation/inputController";

/**
 * rev-6-5: wire `invoke` to a route table keyed by command name. Every invoke
 * is answered by its command, regardless of call order (provider_list at mount,
 * translate_session on Enter). NO mockResolvedValueOnce anywhere.
 */
const { inputInvokeMock } = vi.hoisted(() => ({
  inputInvokeMock: vi.fn(async (_cmd: string, _args?: unknown): Promise<unknown> => {
    throw new Error(`unexpected invoke ${_cmd}`);
  }),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: inputInvokeMock }));

function routeInputInvoke(routes: Record<string, (args?: unknown) => unknown>): void {
  inputInvokeMock.mockImplementation(async (cmd: string, args?: unknown) => {
    const fn = routes[cmd];
    if (!fn) throw new Error(`unexpected invoke ${cmd}`);
    return fn(args);
  });
}

beforeEach(() => {
  vi.clearAllMocks();
  resetProviderNameMap();
});

describe("InputPanel (Surface 02)", () => {
  it("renders a textarea + Translate button", () => {
    routeInputInvoke({
      provider_list: () => [],
      translate_session: () => ({
        outcomes: [{ uuid: "u1", ok: true, text: "hello", engine: "deepseek/u1" }],
        actual_engine: "deepseek/u1",
      }),
    });
    const { getByRole } = render(() => <InputPanel />);
    expect(getByRole("textbox")).toBeTruthy();
    expect(getByRole("button", { name: /翻译|Translate/ })).toBeTruthy();
    cleanup();
  });

  it("Enter (no shift) triggers translate_session and shows the result", async () => {
    routeInputInvoke({
      provider_list: () => [],
      translate_session: () => ({
        outcomes: [{ uuid: "u1", ok: true, text: "hello", engine: "deepseek/u1" }],
        actual_engine: "deepseek/u1",
      }),
    });
    const { getByRole, findByText } = render(() => <InputPanel />);
    const ta = getByRole("textbox") as HTMLTextAreaElement;
    fireEvent.input(ta, { target: { value: "你好" } });
    fireEvent.keyDown(ta, { key: "Enter", shiftKey: false });
    await waitFor(() => expect(inputInvokeMock).toHaveBeenCalledWith(
      "translate_session",
      expect.objectContaining({ req: expect.objectContaining({ text: "你好" }) }),
    ));
    expect(await findByText("hello")).toBeTruthy();
    cleanup();
  });

  it("Shift+Enter does NOT trigger translation", async () => {
    routeInputInvoke({
      provider_list: () => [],
      translate_session: () => ({
        outcomes: [{ uuid: "u1", ok: true, text: "hello", engine: "deepseek/u1" }],
      }),
    });
    const { getByRole } = render(() => <InputPanel />);
    const ta = getByRole("textbox") as HTMLTextAreaElement;
    fireEvent.input(ta, { target: { value: "你好" } });
    fireEvent.keyDown(ta, { key: "Enter", shiftKey: true });
    // Only the mount-time provider_list call should have fired, NOT translate_session.
    expect(inputInvokeMock.mock.calls.some((c) => c[0] === "translate_session")).toBe(false);
    cleanup();
  });

  it("shows InlineError when the engine fails", async () => {
    routeInputInvoke({
      provider_list: () => [],
      translate_session: () => ({
        outcomes: [{ uuid: "u1", ok: false, error: "missing API key" }],
      }),
    });
    const { getByRole, findByText } = render(() => <InputPanel />);
    const ta = getByRole("textbox") as HTMLTextAreaElement;
    fireEvent.input(ta, { target: { value: "hi" } });
    fireEvent.keyDown(ta, { key: "Enter" });
    // classifyError maps "missing key" → config-key → 缺少 API 密钥 / API key missing
    expect(await findByText(/缺少 API 密钥|API key missing/)).toBeTruthy();
    cleanup();
  });

  it("Clear button empties the textarea and result", async () => {
    routeInputInvoke({
      provider_list: () => [],
      translate_session: () => ({
        outcomes: [{ uuid: "u1", ok: true, text: "hello", engine: "deepseek/u1" }],
      }),
    });
    const { getByRole, queryByText } = render(() => <InputPanel />);
    const ta = getByRole("textbox") as HTMLTextAreaElement;
    fireEvent.input(ta, { target: { value: "你好" } });
    fireEvent.keyDown(ta, { key: "Enter" });
    await waitFor(() => expect(inputInvokeMock.mock.calls.some((c) => c[0] === "translate_session")).toBe(true));
    fireEvent.click(getByRole("button", { name: /清空|Clear/ }));
    expect((getByRole("textbox") as HTMLTextAreaElement).value).toBe("");
    expect(queryByText("hello")).toBeNull();
    cleanup();
  });

  // ── B1: multi-engine rendering + friendly engine labels ──────────────────

  it("renders multi-success ResultCards with friendly engine labels", async () => {
    routeInputInvoke({
      provider_list: () => [
        { uuid: "u1", name: "My OpenAI", secret_ref: "provider/u1" },
        { uuid: "u2", name: "My Anthropic", secret_ref: "provider/u2" },
      ],
      translate_session: () => ({
        outcomes: [
          { uuid: "u1", ok: true, text: "你好", engine: "provider/u1" },
          { uuid: "u2", ok: true, text: "您好", engine: "provider/u2" },
        ],
        actual_engine: undefined,
      }),
    });
    const { getByRole, findByText } = render(() => <InputPanel />);
    const textarea = getByRole("textbox") as HTMLTextAreaElement;
    fireEvent.input(textarea, { target: { value: "hello" } });
    fireEvent.keyDown(textarea, { key: "Enter" });
    expect(await findByText("你好")).toBeTruthy();
    expect(await findByText("您好")).toBeTruthy();
    expect(await findByText("My OpenAI")).toBeTruthy();
    expect(await findByText("My Anthropic")).toBeTruthy();
    expect(document.body.textContent).not.toContain("provider/u1");
    cleanup();
  });

  it("renders all-failed InlineError when every engine fails", async () => {
    routeInputInvoke({
      provider_list: () => [],
      translate_session: () => ({
        outcomes: [{ uuid: "u1", ok: false, error: "network" }],
        actual_engine: undefined,
      }),
    });
    const { getByRole, findByText } = render(() => <InputPanel />);
    const textarea = getByRole("textbox") as HTMLTextAreaElement;
    fireEvent.input(textarea, { target: { value: "hello" } });
    fireEvent.keyDown(textarea, { key: "Enter" });
    expect(await findByText(/网络错误|Network error/)).toBeTruthy();
    cleanup();
  });

  it("renders a partial result (one ok, one failed)", async () => {
    routeInputInvoke({
      provider_list: () => [],
      translate_session: () => ({
        outcomes: [
          { uuid: "u1", ok: true, text: "你好", engine: "provider/u1" },
          { uuid: "u2", ok: false, error: "config-401" },
        ],
        actual_engine: undefined,
      }),
    });
    const { getByRole, findByText } = render(() => <InputPanel />);
    const textarea = getByRole("textbox") as HTMLTextAreaElement;
    fireEvent.input(textarea, { target: { value: "hello" } });
    fireEvent.keyDown(textarea, { key: "Enter" });
    expect(await findByText("你好")).toBeTruthy();
    // The failed entry's errorText ("config-401") is rendered verbatim by the
    // ResultCard (B1 renders friendly ENGINE labels, not error labels).
    expect(await findByText("config-401")).toBeTruthy();
    cleanup();
  });

  // ── B2: autosave / restore / clear-purge / focus ──────────────────────────

  it("restores a saved draft on mount", () => {
    localStorage.setItem("linguaray.input-draft", "saved draft");
    routeInputInvoke({
      provider_list: () => [],
      translate_session: () => ({ outcomes: [] }),
    });
    render(() => <InputPanel />);
    const textarea = document.querySelector("textarea") as HTMLTextAreaElement;
    expect(textarea.value).toBe("saved draft");
    localStorage.removeItem("linguaray.input-draft");
    cleanup();
  });

  it("persists the draft after 300ms debounce", () => {
    vi.useFakeTimers();
    routeInputInvoke({
      provider_list: () => [],
      translate_session: () => ({ outcomes: [] }),
    });
    render(() => <InputPanel />);
    const textarea = document.querySelector("textarea")!;
    fireEvent.input(textarea, { target: { value: "typing" } });
    expect(localStorage.getItem("linguaray.input-draft")).toBeNull();
    vi.advanceTimersByTime(350);
    expect(localStorage.getItem("linguaray.input-draft")).toBe("typing");
    localStorage.removeItem("linguaray.input-draft");
    vi.useRealTimers();
    cleanup();
  });

  it("Clear purges the persisted draft", async () => {
    routeInputInvoke({
      provider_list: () => [],
      translate_session: () => ({
        outcomes: [{ uuid: "u1", ok: true, text: "你好", engine: "openai" }],
        actual_engine: "openai",
      }),
    });
    localStorage.setItem("linguaray.input-draft", "leftover");
    const { getByRole, findByText } = render(() => <InputPanel />);
    const textarea = document.querySelector("textarea")!;
    fireEvent.input(textarea, { target: { value: "hello" } });
    fireEvent.keyDown(textarea, { key: "Enter" });
    // Wait for the translation to resolve (hasResult=true → Clear enabled).
    await findByText("你好");
    fireEvent.click(getByRole("button", { name: /清空|Clear/ }));
    expect(localStorage.getItem("linguaray.input-draft")).toBeNull();
    cleanup();
  });

  it("focuses the textarea on mount", () => {
    routeInputInvoke({
      provider_list: () => [],
      translate_session: () => ({ outcomes: [] }),
    });
    render(() => <InputPanel />);
    const textarea = document.querySelector("textarea")!;
    expect(document.activeElement).toBe(textarea);
    cleanup();
  });

  // ── B4 / P1-5: Clear gating fix ───────────────────────────────────────────

  it("Clear is enabled when text is typed but no result yet", () => {
    routeInputInvoke({
      provider_list: () => [],
      translate_session: () => ({
        outcomes: [{ uuid: "u1", ok: true, text: "hello", engine: "deepseek/u1" }],
      }),
    });
    const { getByRole } = render(() => <InputPanel />);
    const ta = getByRole("textbox") as HTMLTextAreaElement;
    fireEvent.input(ta, { target: { value: "你好" } });
    // Text is present but no translation has run (hasResult is false) — the
    // Clear button must be enabled so the user can discard the typed text.
    const clearBtn = getByRole("button", { name: /清空|Clear/ }) as HTMLButtonElement;
    expect(clearBtn.disabled).toBe(false);
    cleanup();
  });

  it("Favorite on a result card saves the source and translation", async () => {
    routeInputInvoke({
      provider_list: () => [],
      translate_session: () => ({
        outcomes: [{ uuid: "u1", ok: true, text: "你好", engine: "deepseek/u1" }],
        actual_engine: "deepseek/u1",
      }),
      vocabulary_add: () => ({ item_uuid: "v1" }),
    });
    const { getByRole, findByLabelText } = render(() => <InputPanel />);
    const ta = getByRole("textbox") as HTMLTextAreaElement;
    fireEvent.input(ta, { target: { value: "hello" } });
    fireEvent.keyDown(ta, { key: "Enter", shiftKey: false });
    const favorite = await findByLabelText(/收藏到生词本|Save to vocabulary/);
    await fireEvent.click(favorite);
    expect(inputInvokeMock).toHaveBeenCalledWith(
      "vocabulary_add",
      expect.objectContaining({
        word: "hello",
        definition: "你好",
      }),
    );
    cleanup();
  });
});
