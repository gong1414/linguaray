import { render, screen, fireEvent, cleanup } from "@testing-library/react";
import { renderHook, act, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { AppProviders } from "../../app/providers";
import { UpdaterPanelView } from "./view";
import { useUpdaterController } from "./controller";
import { applyCheck, applyProgress } from "./model";

const { ipc } = vi.hoisted(() => ({
  ipc: {
    updaterCheck: vi.fn(),
    updaterDownloadInstall: vi.fn(),
    getUpdaterStartupCheck: vi.fn(),
    setUpdaterStartupCheck: vi.fn(),
    onUpdaterProgress: vi.fn(),
    relaunchApp: vi.fn(),
  },
}));
vi.mock("./ipc", () => ipc);

const AVAILABLE = { state: "available" as const, current: "0.1.0", next: "0.2.0", notes: "notes" };
const listeners: Array<(p: unknown) => void> = [];

beforeEach(() => {
  listeners.length = 0;
  vi.clearAllMocks();
  ipc.updaterCheck.mockResolvedValue({ state: "up_to_date", version: "0.1.0" });
  ipc.getUpdaterStartupCheck.mockResolvedValue(true);
  ipc.setUpdaterStartupCheck.mockResolvedValue(undefined);
  ipc.onUpdaterProgress.mockImplementation(
    (cb: (p: unknown) => void) =>
      new Promise((res) => {
        listeners.push(cb);
        res(() => {});
      }),
  );
});

afterEach(cleanup);

function Live() {
  const c = useUpdaterController();
  return <UpdaterPanelView c={c} />;
}

describe("phase machine (pure)", () => {
  it("a late check response cannot knock out downloading/installing", () => {
    const dl = { kind: "downloading" as const, update: AVAILABLE, percent: 10, downloaded: 5 };
    expect(applyCheck(dl, { state: "up_to_date", version: "9" })).toBe(dl);
    expect(applyCheck({ kind: "installing", update: AVAILABLE }, AVAILABLE).kind).toBe("installing");
  });

  it("progress maps percent and finished", () => {
    const dl = { kind: "downloading" as const, update: AVAILABLE, percent: null, downloaded: 0 };
    const pct = (r: ReturnType<typeof applyProgress>) => (r.kind === "downloading" ? r.percent : "not-downloading");
    expect(pct(applyProgress(dl, { downloaded: 5, total: 10, bucket: 0 }))).toBe(50);
    expect(pct(applyProgress(dl, { downloaded: 5, total: null, bucket: 0 }))).toBeNull();
    expect(applyProgress(dl, { finished: true }).kind).toBe("installing");
    // Progress outside downloading is ignored.
    expect(applyProgress({ kind: "upToDate", version: "1" }, { finished: true }).kind).toBe("upToDate");
  });
});

describe("useUpdaterController", () => {
  it("checks on mount and renders up-to-date", async () => {
    render(<Live />, { wrapper: AppProviders });
    expect(await screen.findByText("You are on the latest version.")).toBeInTheDocument();
    expect(screen.getByTestId("updater-current-version")).toHaveTextContent("0.1.0");
  });

  it("download → progress events → installing → install-done → readyToRelaunch", async () => {
    ipc.updaterCheck.mockResolvedValueOnce(AVAILABLE);
    // The install promise resolves ONLY after the finished event lands (the
    // real backend keeps installing until its events say so).
    let finishInstall!: () => void;
    ipc.updaterDownloadInstall.mockImplementation(
      () =>
        new Promise((res) => {
          finishInstall = () => res({ state: "available", current: "0.1.0", next: "0.2.0", notes: "" });
        }),
    );
    render(<Live />, { wrapper: AppProviders });
    fireEvent.click(await screen.findByTestId("updater-download"));
    await waitFor(() => expect(screen.getByTestId("updater-progress")).toBeInTheDocument());
    act(() => {
      for (const l of listeners) l({ downloaded: 50, total: 100, bucket: 0 });
    });
    await waitFor(() => expect(screen.getByText(/Downloading 50%/)).toBeInTheDocument());
    act(() => {
      for (const l of listeners) l({ finished: true });
    });
    await waitFor(() => expect(screen.getByTestId("updater-installing")).toBeInTheDocument());
    act(() => {
      finishInstall();
    });
    await waitFor(() => expect(screen.getByTestId("updater-relaunch")).toBeInTheDocument());
  });

  it("auto-check toggle reverts when the store rejects it", async () => {
    ipc.setUpdaterStartupCheck.mockRejectedValueOnce(new Error("db"));
    const { result } = renderHook(() => useUpdaterController());
    await waitFor(() => expect(result.current.phase.kind).toBe("upToDate"));
    await act(async () => {
      result.current.toggleAutoCheck(false);
    });
    await waitFor(() => {
      expect(result.current.autoCheck).toBe(true);
      expect(result.current.autoCheckError).toBe("The update operation failed. Please try again later.");
    });
  });

  it("a rejected check surfaces the error phase", async () => {
    ipc.updaterCheck.mockRejectedValueOnce(new Error("offline"));
    render(<Live />, { wrapper: AppProviders });
    const alert = await screen.findByTestId("updater-error");
    expect(alert).toHaveTextContent("Could not reach the update server");
  });
});
