import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, fireEvent, cleanup, waitFor } from "@solidjs/testing-library";
import OcrOverlay, {
  captureArgs,
  ocrBytesThenTranslate,
  ocrClipboardThenTranslate,
  ocrPathThenTranslate,
} from "../src/OcrOverlay";

const { invokeMock, destroyMock, showMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(async (cmd: string) => {
    if (cmd.startsWith("ocr_")) return { text: "HELLO" };
    return undefined;
  }),
  destroyMock: vi.fn(async () => undefined),
  showMock: vi.fn(async () => undefined),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    destroy: destroyMock,
    show: showMock,
    hide: destroyMock,
  }),
}));

beforeEach(() => {
  localStorage.setItem("linguaray.locale", "en");
  invokeMock.mockClear();
  destroyMock.mockClear();
  showMock.mockClear();
});
afterEach(() => cleanup());

describe("OCR overlay", () => {
  it("sends unscaled screen points to screencapture -R", () => {
    expect(captureArgs({ x: 10.4, y: 20.6, w: 100, h: 50 })).toEqual({
      x: 10,
      y: 21,
      width: 100,
      height: 50,
    });
    // Must not multiply by a devicePixelRatio (Retina would be 2).
    Object.defineProperty(window, "devicePixelRatio", { value: 2, configurable: true });
    expect(captureArgs({ x: 10, y: 20, w: 30, h: 40 })).toEqual({
      x: 10,
      y: 20,
      width: 30,
      height: 40,
    });
  });

  it("file bytes go through ocr_recognize_bytes then translate_selection_ipc", async () => {
    await ocrBytesThenTranslate(new Uint8Array([1, 2, 3]));
    expect(invokeMock).toHaveBeenCalledWith(
      "ocr_recognize_bytes",
      expect.objectContaining({ bytes: [1, 2, 3] }),
    );
    expect(invokeMock).toHaveBeenCalledWith("translate_selection_ipc", { text: "HELLO" });
  });

  it("file path goes through ocr_from_image then translate", async () => {
    await ocrPathThenTranslate("/tmp/shot.png");
    expect(invokeMock).toHaveBeenCalledWith("ocr_from_image", { path: "/tmp/shot.png" });
    expect(invokeMock).toHaveBeenCalledWith("translate_selection_ipc", { text: "HELLO" });
  });

  it("clipboard image goes through ocr_from_clipboard then translate", async () => {
    await ocrClipboardThenTranslate();
    expect(invokeMock).toHaveBeenCalledWith("ocr_from_clipboard");
    expect(invokeMock).toHaveBeenCalledWith("translate_selection_ipc", { text: "HELLO" });
  });

  it("toolbar Clipboard image button invokes the clipboard OCR command", async () => {
    const { getByText } = render(() => <OcrOverlay />);
    fireEvent.click(getByText("Clipboard image"));
    await waitFor(() => expect(invokeMock).toHaveBeenCalledWith("ocr_from_clipboard"));
    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith("translate_selection_ipc", { text: "HELLO" }),
    );
  });

  it("mouseup after a finished drag does not recapture leftover rect", async () => {
    const { container, getByText } = render(() => <OcrOverlay />);
    const root = container.firstChild as HTMLElement;
    fireEvent.mouseDown(root, { screenX: 10, screenY: 20 });
    fireEvent.mouseMove(root, { screenX: 80, screenY: 90 });
    fireEvent.mouseUp(root, { screenX: 80, screenY: 90 });
    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith(
        "ocr_capture_region",
        expect.objectContaining({ x: 10, y: 20, width: 70, height: 70 }),
      ),
    );
    // Wait for the session chain to finish (destroy) before re-triggering —
    // runOcr is busy-gated while a capture is in flight.
    await waitFor(() => expect(destroyMock).toHaveBeenCalled());
    invokeMock.mockClear();
    fireEvent.mouseUp(root, { screenX: 80, screenY: 90 });
    fireEvent.click(getByText("Clipboard image"));
    await waitFor(() => expect(invokeMock).toHaveBeenCalledWith("ocr_from_clipboard"));
    expect(invokeMock.mock.calls.some((c) => c[0] === "ocr_capture_region")).toBe(false);
  });

  it("mouseup on the toolbar never starts region capture", async () => {
    const { getByText } = render(() => <OcrOverlay />);
    const toolbar = getByText("Clipboard image");
    fireEvent.mouseDown(toolbar, { screenX: 10, screenY: 20 });
    fireEvent.mouseMove(toolbar, { screenX: 80, screenY: 90 });
    fireEvent.mouseUp(toolbar, { screenX: 80, screenY: 90 });
    expect(invokeMock.mock.calls.some((c) => c[0] === "ocr_capture_region")).toBe(false);
  });

  it("dropping a file invokes ocr_recognize_bytes", async () => {
    const { container } = render(() => <OcrOverlay />);
    const file = {
      name: "shot.png",
      type: "image/png",
      arrayBuffer: async () => Uint8Array.from([9, 8, 7]).buffer,
    } as unknown as File;
    const dataTransfer = { files: [file] };
    fireEvent.drop(container.firstChild as Element, { dataTransfer });
    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith(
        "ocr_recognize_bytes",
        expect.objectContaining({ bytes: expect.any(Array) }),
      ),
    );
  });

  it("mount shows the on-demand window (built hidden by ocr_capture)", async () => {
    render(() => <OcrOverlay />);
    await waitFor(() => expect(showMock).toHaveBeenCalled());
  });

  it("Esc destroys the overlay session (cancel is quiet)", async () => {
    render(() => <OcrOverlay />);
    fireEvent.keyDown(window, { key: "Escape" });
    await waitFor(() => expect(destroyMock).toHaveBeenCalled());
  });

  it("right-click cancels the session", async () => {
    const { container } = render(() => <OcrOverlay />);
    fireEvent.contextMenu(container.firstChild as Element);
    await waitFor(() => expect(destroyMock).toHaveBeenCalled());
  });

  it("failed region OCR keeps the overlay open with a visible error", async () => {
    invokeMock.mockImplementation(async (cmd: string) => {
      if (cmd === "ocr_capture_region") throw new Error("no OCR language pack");
      if (cmd.startsWith("ocr_")) return { text: "HELLO" };
      return undefined;
    });
    const { container, findByRole } = render(() => <OcrOverlay />);
    const root = container.firstChild as HTMLElement;
    fireEvent.mouseDown(root, { screenX: 10, screenY: 20 });
    fireEvent.mouseMove(root, { screenX: 80, screenY: 90 });
    fireEvent.mouseUp(root, { screenX: 80, screenY: 90 });
    expect(await findByRole("alert")).toHaveTextContent("no OCR language pack");
    expect(destroyMock).not.toHaveBeenCalled();
  });

  it("successful region OCR destroys the session window", async () => {
    // The previous test replaced the implementation; mockClear does not
    // restore it — reinstall the default success behavior explicitly.
    invokeMock.mockImplementation(async (cmd: string) => {
      if (cmd.startsWith("ocr_")) return { text: "HELLO" };
      return undefined;
    });
    const { container } = render(() => <OcrOverlay />);
    const root = container.firstChild as HTMLElement;
    fireEvent.mouseDown(root, { screenX: 10, screenY: 20 });
    fireEvent.mouseMove(root, { screenX: 80, screenY: 90 });
    fireEvent.mouseUp(root, { screenX: 80, screenY: 90 });
    await waitFor(() => expect(destroyMock).toHaveBeenCalled());
  });
});
