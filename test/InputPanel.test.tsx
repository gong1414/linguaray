import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, fireEvent, cleanup, waitFor } from "@solidjs/testing-library";
import InputPanel from "../src/InputPanel";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async (_cmd: string, _args?: unknown) => ({
    outcomes: [{ uuid: "u1", ok: true, text: "hello", engine: "deepseek/u1" }],
    actual_engine: "deepseek/u1",
  })),
}));

beforeEach(() => vi.clearAllMocks());

describe("InputPanel (Surface 02)", () => {
  it("renders a textarea + Translate button", () => {
    const { getByRole } = render(() => <InputPanel />);
    expect(getByRole("textbox")).toBeTruthy();
    expect(getByRole("button", { name: /翻译|Translate/ })).toBeTruthy();
    cleanup();
  });

  it("Enter (no shift) triggers translate_session and shows the result", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const { getByRole, findByText } = render(() => <InputPanel />);
    const ta = getByRole("textbox") as HTMLTextAreaElement;
    fireEvent.input(ta, { target: { value: "你好" } });
    fireEvent.keyDown(ta, { key: "Enter", shiftKey: false });
    await waitFor(() => expect(vi.mocked(invoke)).toHaveBeenCalledWith(
      "translate_session",
      expect.objectContaining({ req: expect.objectContaining({ text: "你好" }) }),
    ));
    expect(await findByText("hello")).toBeTruthy();
    cleanup();
  });

  it("Shift+Enter does NOT trigger translation", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const { getByRole } = render(() => <InputPanel />);
    const ta = getByRole("textbox") as HTMLTextAreaElement;
    fireEvent.input(ta, { target: { value: "你好" } });
    fireEvent.keyDown(ta, { key: "Enter", shiftKey: true });
    expect(vi.mocked(invoke)).not.toHaveBeenCalled();
    cleanup();
  });

  it("shows InlineError when the engine fails", async () => {
    vi.mocked((await import("@tauri-apps/api/core")).invoke).mockResolvedValueOnce({
      outcomes: [{ uuid: "u1", ok: false, error: "missing API key" }],
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
    const { invoke } = await import("@tauri-apps/api/core");
    const { getByRole, queryByText } = render(() => <InputPanel />);
    const ta = getByRole("textbox") as HTMLTextAreaElement;
    fireEvent.input(ta, { target: { value: "你好" } });
    fireEvent.keyDown(ta, { key: "Enter" });
    await waitFor(() => expect(vi.mocked(invoke)).toHaveBeenCalled());
    fireEvent.click(getByRole("button", { name: /清空|Clear/ }));
    expect((getByRole("textbox") as HTMLTextAreaElement).value).toBe("");
    expect(queryByText("hello")).toBeNull();
    cleanup();
  });
});
