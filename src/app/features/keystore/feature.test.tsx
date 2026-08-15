import { render, screen, fireEvent, cleanup } from "@testing-library/react";
import { renderHook, act, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { AppProviders } from "../../app/providers";
import { KeystoreRecoveryView } from "./view";
import { useKeystoreController } from "./controller";

const { ipc } = vi.hoisted(() => ({
  ipc: { keystoreHealth: vi.fn(), archiveKeystore: vi.fn(), resetKeystore: vi.fn() },
}));
vi.mock("./ipc", () => ipc);

beforeEach(() => {
  vi.clearAllMocks();
  ipc.keystoreHealth.mockResolvedValue("");
  ipc.archiveKeystore.mockResolvedValue("/backup/ks");
  ipc.resetKeystore.mockResolvedValue(null);
});

afterEach(cleanup);

function Live() {
  const c = useKeystoreController();
  return <KeystoreRecoveryView c={c} />;
}

describe("useKeystoreController", () => {
  it("empty health = healthy; non-empty = corrupt with the reason", async () => {
    const a = renderHook(() => useKeystoreController());
    await waitFor(() => expect(a.result.current.state).toBe("healthy"));

    ipc.keystoreHealth.mockResolvedValueOnce("bad header");
    const b = renderHook(() => useKeystoreController());
    await waitFor(() => expect(b.result.current.state).toBe("corrupt"));
    expect(b.result.current.reason).toBe("bad header");
  });

  it("a thrown health read is itself corrupt (fail-closed)", async () => {
    ipc.keystoreHealth.mockRejectedValueOnce(new Error("io"));
    const { result } = renderHook(() => useKeystoreController());
    await waitFor(() => expect(result.current.state).toBe("corrupt"));
  });

  it("archive success → archived; failure → destructive toast", async () => {
    ipc.keystoreHealth.mockResolvedValueOnce("x");
    const { result } = renderHook(() => useKeystoreController());
    await waitFor(() => expect(result.current.state).toBe("corrupt"));
    await act(async () => {
      result.current.archive();
    });
    await waitFor(() => expect(result.current.state).toBe("archived"));

    ipc.keystoreHealth.mockResolvedValueOnce("x");
    ipc.archiveKeystore.mockRejectedValueOnce(new Error("fs"));
    const b = renderHook(() => useKeystoreController());
    await waitFor(() => expect(b.result.current.state).toBe("corrupt"));
    await act(async () => {
      b.result.current.archive();
    });
    await waitFor(() => expect(b.result.current.toasts[0].variant).toBe("destructive"));
    expect(b.result.current.state).toBe("corrupt");
  });

  it("reset confirm flow: open → reset → archived + dialog closed", async () => {
    ipc.keystoreHealth.mockResolvedValueOnce("x");
    const { result } = renderHook(() => useKeystoreController());
    await waitFor(() => expect(result.current.state).toBe("corrupt"));
    act(() => result.current.openReset());
    expect(result.current.resetOpen).toBe(true);
    await act(async () => {
      result.current.reset();
    });
    await waitFor(() => {
      expect(result.current.state).toBe("archived");
      expect(result.current.resetOpen).toBe(false);
    });
  });
});

describe("KeystoreRecoveryView (integration)", () => {
  it("corrupt state shows the destructive banner with both actions", async () => {
    ipc.keystoreHealth.mockResolvedValueOnce("bad magic");
    render(<Live />, { wrapper: AppProviders });
    const banner = await screen.findByTestId("keystore-corrupt");
    expect(banner).toHaveTextContent("Keystore unreadable: bad magic");
    expect(screen.getByRole("button", { name: "Archive & re-enter" })).toBeInTheDocument();
    expect(screen.getByTestId("keystore-reset-trigger")).toBeInTheDocument();
  });

  it("healthy renders no banner", async () => {
    render(<Live />, { wrapper: AppProviders });
    await waitFor(() =>
      expect(screen.queryByTestId("keystore-corrupt")).toBeNull(),
    );
  });

  it("reset modal opens from the banner and confirms destructively", async () => {
    ipc.keystoreHealth.mockResolvedValueOnce("x");
    render(<Live />, { wrapper: AppProviders });
    await screen.findByTestId("keystore-corrupt");
    fireEvent.click(screen.getByTestId("keystore-reset-trigger"));
    // Mantine portals the modal content — query globally; the modal's confirm
    // is the LAST "Reset" button in DOM order (banner trigger comes first).
    const confirm = await screen.findAllByRole("button", { name: "Reset" }).then((xs) => xs.pop()!);
    fireEvent.click(confirm);
    await waitFor(() => expect(ipc.resetKeystore).toHaveBeenCalledTimes(1));
  });
});
