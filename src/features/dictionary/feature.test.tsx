import { render, screen, fireEvent, cleanup } from "@testing-library/react";
import { renderHook, act, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { AppProviders } from "../../app/providers";
import { DictionaryView } from "./view";
import { useDictionaryController } from "./controller";

const { ipc } = vi.hoisted(() => ({
  ipc: {
    dictLookup: vi.fn(),
    dictListPackages: vi.fn(),
    dictInstallPackage: vi.fn(),
  },
}));
vi.mock("./ipc", () => ipc);

beforeEach(() => {
  vi.clearAllMocks();
  ipc.dictListPackages.mockResolvedValue([{ package_id: "en-zh", name: "EN-ZH" }]);
  ipc.dictInstallPackage.mockResolvedValue(undefined);
});

afterEach(cleanup);

function DictionaryLive() {
  const c = useDictionaryController();
  return <DictionaryView c={c} />;
}

describe("useDictionaryController", () => {
  it("loads packages on mount; a rejection surfaces the error", async () => {
    ipc.dictListPackages.mockRejectedValueOnce(new Error("io"));
    const { result } = renderHook(() => useDictionaryController());
    await waitFor(() => expect(result.current.error).toContain("io"));
  });

  it("lookup maps hit / miss / error", async () => {
    const { result } = renderHook(() => useDictionaryController());
    act(() => result.current.setWord("hello"));
    ipc.dictLookup.mockResolvedValueOnce({ definition: "你好", source: "en-zh" });
    await act(async () => {
      result.current.lookup();
    });
    await waitFor(() => expect(result.current.result?.definition).toBe("你好"));

    ipc.dictLookup.mockResolvedValueOnce(null);
    await act(async () => {
      result.current.lookup();
    });
    await waitFor(() => expect(result.current.miss).toBe(true));

    ipc.dictLookup.mockRejectedValueOnce(new Error("boom"));
    await act(async () => {
      result.current.lookup();
    });
    await waitFor(() => expect(result.current.error).toContain("boom"));
  });

  it("install defaults name/version and reloads packages", async () => {
    const { result } = renderHook(() => useDictionaryController());
    act(() => result.current.setSourceDir("/data/dict"));
    act(() => result.current.setPackageId("en-zh"));
    await act(async () => {
      result.current.install();
    });
    expect(ipc.dictInstallPackage).toHaveBeenCalledWith("/data/dict", "en-zh", "en-zh", "1.0");
    await waitFor(() => expect(result.current.notice).toBeTruthy());
    expect(ipc.dictListPackages).toHaveBeenCalledTimes(2);
  });
});

describe("DictionaryView (integration)", () => {
  it("lookup flow renders result + source; Enter triggers lookup", async () => {
    ipc.dictLookup.mockResolvedValue({ definition: "你好", source: "en-zh" });
    render(<DictionaryLive />, { wrapper: AppProviders });
    const input = screen.getByRole("textbox", { name: "Word" });
    fireEvent.change(input, { target: { value: "hello" } });
    fireEvent.keyDown(input, { key: "Enter" });
    expect(await screen.findByTestId("dictionary-result")).toHaveTextContent("你好");
    expect(screen.getByText("Source: en-zh")).toBeInTheDocument();
  });

  it("install disabled until folder + id filled", async () => {
    render(<DictionaryLive />, { wrapper: AppProviders });
    const buttons = screen.getAllByRole("button", { name: "Install package" });
    expect(buttons[0]).toBeDisabled();
    fireEvent.change(screen.getByRole("textbox", { name: "Folder path" }), {
      target: { value: "/d" },
    });
    expect(buttons[0]).toBeDisabled();
    fireEvent.change(screen.getByRole("textbox", { name: "Package id" }), {
      target: { value: "x" },
    });
    expect(buttons[0]).toBeEnabled();
  });
});
