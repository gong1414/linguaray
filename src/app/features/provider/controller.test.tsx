import { act, cleanup, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const { ipc } = vi.hoisted(() => ({
  ipc: {
    providerListPresets: vi.fn(),
    loadProviders: vi.fn(),
    providerGetActiveSelection: vi.fn(),
    providerToggle: vi.fn(),
    providerSetActive: vi.fn(),
    providerConfirmAndSetActive: vi.fn(),
    providerCreate: vi.fn(),
    providerDuplicate: vi.fn(),
    providerDelete: vi.fn(),
    providerReorder: vi.fn(),
    providerUpdate: vi.fn(),
    providerSetKey: vi.fn(),
    providerGetModels: vi.fn(),
    providerTestConnection: vi.fn(),
    providerGetBalance: vi.fn(),
  },
}));

vi.mock("./ipc", () => ipc);
vi.mock("../../../bridge/invoke", () => ({ invoke: vi.fn() }));

import { useProviderController } from "./controller";
import type { ProviderProfileFE } from "./model";

const profile = (uuid: string, over: Partial<ProviderProfileFE> = {}): ProviderProfileFE => ({
  uuid,
  template_id: "openai",
  name: `P-${uuid}`,
  protocol: "openai_chat",
  endpoint: "https://api.openai.com/v1",
  model: "gpt-4o",
  enabled: true,
  sort_order: 0,
  is_local: false,
  needs_key: true,
  secret_ref: `ref-${uuid}`,
  capabilities: { balance: false, quota: false, model_list: false },
  status: "active",
  version: 3,
  hasKey: true,
  ...over,
});

const A = profile("a", { sort_order: 0 });
const B = profile("b", { sort_order: 1, template_id: "ollama", endpoint: "http://localhost:11434", is_local: true, needs_key: false, hasKey: false });

beforeEach(() => {
  vi.clearAllMocks();
  // The ipc module is mocked wholesale, so hand the MAPPED Preset shape (the
  // real wrapper maps the raw catalog DTO inside ./ipc).
  ipc.providerListPresets.mockResolvedValue([
    { templateId: "openai", name: "OpenAI", endpoint: "https://api.openai.com", model: "gpt-4o", needsKey: true, auth: "bearer", requiresUserEndpoint: false, notes: null, supportTier: "ready", icon: null },
  ]);
  ipc.loadProviders.mockResolvedValue([A, B]);
  ipc.providerGetActiveSelection.mockResolvedValue({ primary: "a", parallel: [], fallback: null });
  ipc.providerToggle.mockResolvedValue(undefined);
  ipc.providerSetActive.mockResolvedValue({ outcome: "written" });
  ipc.providerUpdate.mockResolvedValue({ ...A, version: 4 });
  ipc.providerDelete.mockResolvedValue(undefined);
  ipc.providerReorder.mockResolvedValue(undefined);
  ipc.providerGetModels.mockResolvedValue([{ id: "m1", label: "M1" }]);
  ipc.providerTestConnection.mockResolvedValue({ ok: true, message: "pong", latency_ms: 12 });
});

afterEach(cleanup);

const ready = async () => {
  const hook = renderHook(() => useProviderController());
  await waitFor(() => expect(hook.result.current.providers).toHaveLength(2));
  return hook;
};

describe("useProviderController — load", () => {
  it("cold-loads providers + active selection + presets", async () => {
    const hook = renderHook(() => useProviderController());
    await waitFor(() => expect(hook.result.current.providers).toHaveLength(2));
    const { result } = hook;
    expect(result.current.providers.map((p) => p.uuid)).toEqual(["a", "b"]);
    expect(result.current.selection.primaryUuid).toBe("a");
    await waitFor(() => expect(result.current.presets.length).toBe(1), { timeout: 3000 });
    expect(result.current.presets[0]?.templateId).toBe("openai");
    expect(result.current.selectionLoading).toBe(false);
  });

  it("fail-closed cold-load: loadError + selectionError, retry heals", async () => {
    ipc.loadProviders.mockRejectedValueOnce(new Error("db"));
    const { result } = renderHook(() => useProviderController());
    await waitFor(() => expect(result.current.loadError).toBe(true));
    expect(result.current.selectionError).toBe(true);
    expect(result.current.toasts.some((x) => x.variant === "destructive")).toBe(true);

    await act(async () => {
      result.current.onRetrySelectionLoad();
    });
    await waitFor(() => expect(result.current.loadError).toBe(false));
  });

  it("a rejected selection read heals via retry", async () => {
    ipc.loadProviders.mockRejectedValueOnce(new Error("db"));
    const { result } = renderHook(() => useProviderController());
    await waitFor(() => expect(result.current.loadError).toBe(true));
    await act(async () => {
      result.current.onRetrySelectionLoad();
    });
    await waitFor(() => expect(result.current.loadError).toBe(false));
  });
});

describe("useProviderController — toggle + selection", () => {
  it("toggle is optimistic; failure rolls back with a destructive toast", async () => {
    const { result } = await ready();
    ipc.providerToggle.mockRejectedValueOnce(new Error("net"));
    await act(async () => {
      result.current.onToggle("a", false);
    });
    // rolled back to enabled
    await waitFor(() => expect(result.current.providers.find((p) => p.uuid === "a")?.enabled).toBe(true));
    expect(result.current.toasts.some((x) => x.variant === "destructive")).toBe(true);
  });

  it("disabling the primary clears its slot in the session mirror", async () => {
    const { result } = await ready();
    await act(async () => {
      result.current.onToggle("a", false);
    });
    await waitFor(() => expect(result.current.selection.primaryUuid).toBeNull());
  });

  it("role mutations are fail-closed while the selection read failed", async () => {
    ipc.providerGetActiveSelection.mockRejectedValueOnce(new Error("x"));
    const { result } = renderHook(() => useProviderController());
    await waitFor(() => expect(result.current.selectionError).toBe(true));
    await act(async () => {
      result.current.onSetPrimary("b");
    });
    expect(ipc.providerSetActive).not.toHaveBeenCalled();
  });

  it("addParallel needs_consent opens the dialog; confirm writes and closes", async () => {
    ipc.providerSetActive.mockResolvedValueOnce({ outcome: "needs_consent", actual_scope: "scope-1" });
    const { result } = await ready();
    await act(async () => {
      result.current.onAddParallel("b");
    });
    await waitFor(() => expect(result.current.consentOpen).toBe(true));
    expect(result.current.consentRecipients.map((r) => r.name)).toEqual(["P-a", "P-b"]);

    ipc.providerConfirmAndSetActive.mockResolvedValueOnce(1);
    await act(async () => {
      result.current.onConfirmConsent();
    });
    await waitFor(() => {
      expect(result.current.consentOpen).toBe(false);
      expect(result.current.selection.parallelUuids).toEqual(["b"]);
    });
  });

  it("setPrimary removes the provider from parallel and clears fallback overlap", async () => {
    ipc.providerGetActiveSelection.mockResolvedValue({ primary: "a", parallel: ["b"], fallback: null });
    const { result } = await ready();
    await act(async () => {
      result.current.onSetPrimary("b");
    });
    await waitFor(() => {
      expect(result.current.selection.primaryUuid).toBe("b");
      expect(result.current.selection.parallelUuids).toEqual([]);
    });
    expect(ipc.providerSetActive).toHaveBeenCalledWith("b", [], null);
  });
});

describe("useProviderController — save / key", () => {
  it("stale_version surfaces the save-conflict banner and preserves drafts", async () => {
    const { result } = await ready();
    act(() => result.current.select("a"));
    await act(async () => {
      result.current.onEndpointInput("a", "https://new.example.com");
      result.current.onNameInput("a", "renamed");
    });
    ipc.providerUpdate.mockRejectedValueOnce({ error: "stale_version", actual_version: 9 });
    await act(async () => {
      result.current.onSaveProfile("a");
    });
    await waitFor(() => expect(result.current.detail?.saveConflict).toBe(true));
    // Draft preserved, not clobbered by the conflict.
    expect(result.current.detail?.endpointDraft).toBe("https://new.example.com");
    expect(result.current.detail?.nameDraft).toBe("renamed");
  });

  it("resolveSaveConflict reloads and clears drafts for that uuid", async () => {
    const { result } = await ready();
    act(() => result.current.select("a"));
    await act(async () => {
      result.current.onEndpointInput("a", "https://new.example.com");
      result.current.onSaveProfile("a");
    });
    await act(async () => {
      result.current.onResolveSaveConflict("a");
    });
    await waitFor(() => expect(result.current.detail?.endpointDraft).toBe(A.endpoint));
  });

  it("duplicate names are rejected client-side before IPC", async () => {
    const { result } = await ready();
    act(() => result.current.select("a"));
    await act(async () => {
      result.current.onNameInput("a", "P-b"); // collides with provider b
      result.current.onSaveProfile("a");
    });
    expect(ipc.providerUpdate).not.toHaveBeenCalled();
    await waitFor(() => expect(result.current.detail?.nameError).toBeTruthy());
  });

  it("invalid endpoints abort the save before IPC", async () => {
    const { result } = await ready();
    act(() => result.current.select("a"));
    await act(async () => {
      result.current.onEndpointInput("a", "http://evil.example.com");
      result.current.onSaveProfile("a");
    });
    expect(ipc.providerUpdate).not.toHaveBeenCalled();
    expect(result.current.detail?.endpointError).toBeTruthy();
  });

  it("saveKey clears the input immediately and refreshes hasKey", async () => {
    const { result } = await ready();
    act(() => result.current.select("b")); // needs_key=false → guard
    await act(async () => {
      result.current.onKeyInput("b", "sk-nope");
      result.current.onSaveKey("b");
    });
    expect(ipc.providerSetKey).not.toHaveBeenCalled(); // fail-closed for keyless

    const r2 = await ready();
    act(() => r2.result.current.select("a"));
    await act(async () => {
      r2.result.current.onKeyInput("a", "sk-new");
      r2.result.current.onSaveKey("a");
    });
    expect(ipc.providerSetKey).toHaveBeenCalledWith("a", "sk-new");
    await waitFor(() => expect(r2.result.current.detail?.keyText).toBe(""));
  });
});

describe("useProviderController — test/fetch epoch guards", () => {
  it("a stale test completion is discarded after an endpoint edit", async () => {
    let releaseTest!: (v: { ok: boolean; message: string }) => void;
    ipc.providerTestConnection.mockImplementationOnce(
      () => new Promise((res) => (releaseTest = res)),
    );
    const { result } = await ready();
    act(() => result.current.select("a"));
    await act(async () => {
      result.current.onTestConnection("a");
    });
    await waitFor(() => expect(result.current.detail?.conn).toBe("testing"));
    // Config changes while the test is in flight.
    await act(async () => {
      result.current.onEndpointInput("a", "https://changed.example.com");
    });
    await act(async () => {
      releaseTest({ ok: true, message: "stale pong" });
    });
    await waitFor(() => expect(result.current.detail?.conn).toBe("idle"));
  });

  it("a newer test supersedes an older completion (request-id ABA)", async () => {
    let releaseFirst!: (v: { ok: boolean; message: string }) => void;
    ipc.providerTestConnection
      .mockImplementationOnce(() => new Promise((res) => (releaseFirst = res)))
      .mockResolvedValueOnce({ ok: true, message: "second", latency_ms: 5 });
    const { result } = await ready();
    act(() => result.current.select("a"));
    await act(async () => {
      result.current.onTestConnection("a");
    });
    await act(async () => {
      result.current.onTestConnection("a");
    });
    await waitFor(() => expect(result.current.detail?.conn).not.toBe("testing"));
    expect((result.current.detail?.conn as { message: string }).message).toBe("second");
    await act(async () => {
      releaseFirst({ ok: false, message: "first-stale" });
    });
    // Older completion discarded.
    expect((result.current.detail?.conn as { message: string }).message).toBe("second");
  });
});

describe("useProviderController — delete", () => {
  it("a failed delete closes the dialog and surfaces the retry banner", async () => {
    const { result } = await ready();
    act(() => result.current.onDelete("a"));
    expect(result.current.deleteConfirmUuid).toBe("a");
    ipc.providerDelete.mockRejectedValueOnce(new Error("net"));
    await act(async () => {
      result.current.onConfirmDelete();
    });
    await waitFor(() => {
      expect(result.current.deleteError).toBe(true);
      expect(result.current.deleteFailedUuid).toBe("a");
      expect(result.current.deleteConfirmUuid).toBeNull();
    });

    // Retry path succeeds and refreshes the list.
    ipc.loadProviders.mockResolvedValue([B]);
    ipc.providerGetActiveSelection.mockResolvedValue({ primary: null, parallel: [], fallback: null });
    await act(async () => {
      result.current.onRetryDelete();
    });
    await waitFor(() => {
      expect(result.current.deleteError).toBe(false);
      expect(result.current.providers).toHaveLength(1);
    });
  });
});

describe("useProviderController — serial mutex", () => {
  it("mutations never overlap: the second starts only after the first finishes", async () => {
    const order: string[] = [];
    let releaseFirst!: () => void;
    ipc.providerToggle.mockImplementationOnce(
      () => new Promise((res) => ((releaseFirst = () => res(undefined)), order.push("t1-start"))),
    );
    ipc.providerSetActive.mockImplementationOnce(async () => {
      order.push("t2-start");
      return { outcome: "written" };
    });
    const { result } = await ready();
    await act(async () => {
      result.current.onToggle("a", false);
    });
    await act(async () => {
      result.current.onSetPrimary("b");
    });
    // t2 hasn't started while the mutex is held.
    expect(order).toEqual(["t1-start"]);
    await act(async () => {
      releaseFirst();
    });
    await waitFor(() => expect(order).toEqual(["t1-start", "t2-start"]));
    await waitFor(() => expect(result.current.exclusiveBusy).toBe(false));
  });
});
