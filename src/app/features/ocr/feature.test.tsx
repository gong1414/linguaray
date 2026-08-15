import { render, screen, fireEvent, cleanup, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { AppProviders } from "../../app/providers";
import { OcrOverlayView } from "./view";
import { useOcrController } from "./controller";
import { captureArgs } from "./ipc";

const { invoke, destroyMock, showMock } = vi.hoisted(() => ({
  invoke: vi.fn(),
  destroyMock: vi.fn(async () => {}),
  showMock: vi.fn(async () => {}),
}));
vi.mock("../../../bridge/invoke", () => ({ invoke }));
vi.mock("../../../bridge/window", () => ({
  getCurrentWindow: () => ({ show: showMock, destroy: destroyMock }),
}));

function Live() {
  const c = useOcrController();
  return <OcrOverlayView c={c} />;
}

beforeEach(() => {
  vi.clearAllMocks();
  invoke.mockImplementation(async (cmd: string) => {
    if (cmd === "ocr_capture_region") return { text: "recognized" };
    if (cmd === "ocr_recognize_bytes") return { text: "from-bytes" };
    if (cmd === "ocr_from_clipboard") return { text: "from-clipboard" };
    if (cmd === "translate_selection_ipc") return undefined;
    return undefined;
  });
});

afterEach(cleanup);

function dragRegion(from: { x: number; y: number }, to: { x: number; y: number }) {
  const surface = screen.getByTestId("ocr-overlay");
  fireEvent.mouseDown(surface, { screenX: from.x, screenY: from.y });
  fireEvent.mouseMove(surface, { screenX: to.x, screenY: to.y });
  fireEvent.mouseUp(surface, { screenX: to.x, screenY: to.y });
}

describe("OCR overlay (controller + view integration)", () => {
  it("shows the window on mount (built hidden by ocr_capture)", async () => {
    render(<Live />, { wrapper: AppProviders });
    await waitFor(() => expect(showMock).toHaveBeenCalledTimes(1));
  });

  it("drag ≥4px captures the region, OCRs it, translates, destroys", async () => {
    render(<Live />, { wrapper: AppProviders });
    dragRegion({ x: 100, y: 100 }, { x: 300, y: 200 });
    await waitFor(() => expect(destroyMock).toHaveBeenCalledTimes(1));
    const regionCall = invoke.mock.calls.find((c) => c[0] === "ocr_capture_region");
    expect(regionCall).toBeTruthy();
    expect(regionCall![1]).toEqual({ x: 100, y: 100, width: 200, height: 100 });
    expect(invoke).toHaveBeenCalledWith("translate_selection_ipc", { text: "recognized" });
  });

  it("sub-4px drags are ignored (no capture, no destroy)", async () => {
    render(<Live />, { wrapper: AppProviders });
    dragRegion({ x: 100, y: 100 }, { x: 102, y: 101 });
    await waitFor(() => expect(screen.getByRole("status")).toBeInTheDocument());
    expect(invoke).not.toHaveBeenCalledWith("ocr_capture_region", expect.anything());
    expect(destroyMock).not.toHaveBeenCalled();
  });

  it("Escape cancels: destroy without any capture", async () => {
    render(<Live />, { wrapper: AppProviders });
    fireEvent.keyDown(window, { key: "Escape" });
    await waitFor(() => expect(destroyMock).toHaveBeenCalledTimes(1));
    expect(invoke).not.toHaveBeenCalled();
  });

  it("contextmenu cancels too", async () => {
    render(<Live />, { wrapper: AppProviders });
    fireEvent.contextMenu(screen.getByTestId("ocr-overlay"));
    await waitFor(() => expect(destroyMock).toHaveBeenCalledTimes(1));
  });

  it("clipboard path OCRs and destroys", async () => {
    render(<Live />, { wrapper: AppProviders });
    fireEvent.click(screen.getByRole("button", { name: "Clipboard image" }));
    await waitFor(() => expect(destroyMock).toHaveBeenCalledTimes(1));
    expect(invoke).toHaveBeenCalledWith("translate_selection_ipc", { text: "from-clipboard" });
  });

  it("an OCR failure stays VISIBLE with the error (overlay not destroyed)", async () => {
    invoke.mockImplementation(async (cmd: string) => {
      if (cmd === "ocr_capture_region") throw new Error("engine unavailable");
      return undefined;
    });
    render(<Live />, { wrapper: AppProviders });
    dragRegion({ x: 10, y: 10 }, { x: 60, y: 60 });
    const error = await screen.findByTestId("ocr-error");
    expect(error).toHaveTextContent("engine unavailable");
    expect(destroyMock).not.toHaveBeenCalled();
    // Overlay stays actionable: cancel still works.
    fireEvent.keyDown(window, { key: "Escape" });
    await waitFor(() => expect(destroyMock).toHaveBeenCalledTimes(1));
  });

  it("captureArgs rounds to logical points", () => {
    expect(captureArgs({ x: 10.4, y: 20.6, w: 100.5, h: 50.49 })).toEqual({
      x: 10,
      y: 21,
      width: 101,
      height: 50,
    });
  });
});
