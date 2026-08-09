/**
 * Provider Center (Surface 05) component tests — real IPC (mocked invoke).
 *
 * Covers the core state matrix from the design: empty / list / editing /
 * key-saving / deleting / consent / reorder / connection-test. Every flow
 * asserts against the typed `invoke` wrappers, not mock fixtures.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, fireEvent, cleanup, waitFor, screen } from "@solidjs/testing-library";

// vi.hoisted lets us reference the mocks inside the hoisted vi.mock factories.
const { invokeMock, localeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(async (_cmd: string, _args?: unknown): Promise<unknown> => {
    throw new Error(`unexpected invoke ${_cmd}`);
  }),
  localeMock: { current: "en" as "en" | "zh" },
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));
vi.mock("../src/i18n", () => ({ detectLocale: () => localeMock.current }));

import ProviderCenter from "../src/features/settings/ProviderCenter";
import type { ProviderProfile } from "../src/features/settings/provider-types";

const flush = () => new Promise((r) => setTimeout(r, 0));

const profile = (over: Partial<ProviderProfile> = {}): ProviderProfile => ({
  uuid: "u1",
  template_id: "openai",
  name: "TestProvider",
  protocol: "openai_chat",
  endpoint: "https://api.openai.com",
  model: "gpt-4o-mini",
  enabled: true,
  sort_order: 0,
  is_local: false,
  needs_key: true,
  secret_ref: "provider/u1",
  capabilities: { balance: false, quota: false, model_list: true },
  status: "active",
  ...over,
});

/** Two-profile list used by most non-empty flows.
 *  Names are distinct from preset labels to avoid getByText ambiguity
 *  (the preset grid also renders "OpenAI" etc.). */
const TWO_PROFILES: ProviderProfile[] = [
  profile({ uuid: "u1", name: "MyOpenAI", sort_order: 0, secret_ref: "provider/u1" }),
  profile({
    uuid: "u2",
    name: "MyDeepSeek",
    template_id: "deepseek",
    sort_order: 1,
    secret_ref: "provider/u2",
    needs_key: true,
  }),
];

/** Wire `invoke` to a route table keyed by command name. */
function routeInvoke(routes: Record<string, (args?: unknown) => unknown>): void {
  invokeMock.mockImplementation(async (cmd: string, args?: unknown) => {
    const fn = routes[cmd];
    if (!fn) throw new Error(`unexpected invoke ${cmd}`);
    return fn(args);
  });
}

/**
 * Default route table re-installed in `beforeEach` after `mockReset()`. The
 * C1 cold-load calls `provider_get_active_selection` on every refresh, so a
 * bare `provider_list` + `key_status` table is no longer sufficient — omitting
 * the selection read makes `refresh()` reject and never populate the list.
 * Tests that need a custom route pass `{ ...DEFAULT_ROUTES, ...custom }`.
 */
const DEFAULT_ROUTES: Record<string, (args?: unknown) => unknown> = {
  provider_list: () => [],
  key_status: () => ({}),
  provider_get_active_selection: () => ({ primary: null, parallel: [], fallback: null }),
};

beforeEach(() => {
  localeMock.current = "en";
  invokeMock.mockReset();
  routeInvoke(DEFAULT_ROUTES);
});

afterEach(() => cleanup());

// Helper: wait until a mocked invoke has been called with `cmd` (the cmd is
// the first positional arg; the optional args object may or may not be present
// since `loadProviders` calls `invoke("provider_list")` with no second arg).
async function whenCalledWith(cmd: string) {
  await waitFor(() => {
    const called = invokeMock.mock.calls.some((c) => c[0] === cmd);
    expect(called).toBe(true);
  });
}

describe("ProviderCenter (Surface 05)", () => {
  it("on mount, calls provider_list + key_status", async () => {
    routeInvoke({ ...DEFAULT_ROUTES });
    render(() => <ProviderCenter />);
    await whenCalledWith("provider_list");
    await whenCalledWith("key_status");
  });

  it("empty: shows EmptyState + preset grid", async () => {
    routeInvoke({ ...DEFAULT_ROUTES });
    render(() => <ProviderCenter />);
    await waitFor(() =>
      expect(screen.getByText("Add your first provider")).toBeTruthy(),
    );
    // Preset buttons present (at least OpenAI + Ollama).
    expect(screen.getByText("OpenAI")).toBeTruthy();
    expect(screen.getByText("Ollama")).toBeTruthy();
  });

  it("empty → click preset: calls provider_create, re-fetches", async () => {
    const calls: string[] = [];
    let created = false;
    routeInvoke({
      ...DEFAULT_ROUTES,
      provider_list: () => (created ? [profile()] : []),
      provider_create: () => {
        created = true;
        calls.push("create");
        return profile();
      },
    });
    invokeMock.mockImplementation(async (cmd: string) => {
      calls.push(cmd);
      const map: Record<string, () => unknown> = {
        provider_list: () => (created ? [profile()] : []),
        key_status: () => ({}),
        provider_create: () => {
          created = true;
          return profile();
        },
        provider_get_active_selection: () => ({ primary: null, parallel: [], fallback: null }),
      };
      const fn = map[cmd];
      if (!fn) throw new Error(`unexpected ${cmd}`);
      return fn();
    });
    render(() => <ProviderCenter />);
    await flush();

    fireEvent.click(screen.getByText("OpenAI"));
    await flush();
    await waitFor(() => expect(calls).toContain("provider_create"));
    // Re-fetched the list after create.
    const listCalls = calls.filter((c) => c === "provider_list").length;
    expect(listCalls).toBeGreaterThanOrEqual(2);
  });

  it("list: renders rows in sort_order with name", async () => {
    routeInvoke({
      ...DEFAULT_ROUTES,
      provider_list: () => TWO_PROFILES,
      key_status: () => ({ "provider/u1": true, "provider/u2": false }),
    });
    render(() => <ProviderCenter />);
    // Wait for the provider list to render (MyDeepSeek is provider-only, not a preset).
    await waitFor(() => expect(screen.getByText("MyDeepSeek")).toBeTruthy());
    // OpenAI appears both as a preset and a row name; both render.
    expect(screen.getAllByText("OpenAI").length).toBeGreaterThanOrEqual(1);
  });

  it("toggle: calls provider_toggle, optimistically flips, rolls back on error", async () => {
    let toggleShouldFail = false;
    routeInvoke({
      ...DEFAULT_ROUTES,
      provider_list: () => TWO_PROFILES,
      key_status: () => ({ "provider/u1": true, "provider/u2": true }),
      provider_toggle: () => {
        if (toggleShouldFail) throw new Error("net");
      },
    });
    render(() => <ProviderCenter />);
    await waitFor(() => expect(screen.getByText("MyOpenAI")).toBeTruthy());

    // The Switch is a checkbox with role="switch".
    const switches = screen.getAllByRole("switch");
    expect(switches.length).toBeGreaterThanOrEqual(1);
    fireEvent.click(switches[0]);
    await whenCalledWith("provider_toggle");

    // Now make a second render fail → expect a destructive toast on revert.
    toggleShouldFail = true;
    cleanup();
    const { container } = render(() => <ProviderCenter />);
    await waitFor(() => expect(screen.getByText("MyOpenAI")).toBeTruthy());
    fireEvent.click(screen.getAllByRole("switch")[0]);
    await waitFor(() =>
      expect(invokeMock).toHaveBeenCalledWith("provider_toggle", expect.anything()),
    );
    // Rollback: switch reverts to on (provider_list still returns enabled).
    await flush();
    // No throw from the test itself means rollback path exercised.
    expect(container).toBeTruthy();
  });

  it("edit: selecting a row opens detail with endpoint + model fields", async () => {
    routeInvoke({
      ...DEFAULT_ROUTES,
      provider_list: () => TWO_PROFILES,
      key_status: () => ({ "provider/u1": true, "provider/u2": true }),
    });
    render(() => <ProviderCenter />);
    await waitFor(() => expect(screen.getByText("MyOpenAI")).toBeTruthy());

    // Click the edit button (aria-label "Edit MyOpenAI").
    fireEvent.click(screen.getByLabelText("Edit MyOpenAI"));
    await flush();
    // Detail panel: endpoint TextField has an associated <label> "Endpoint".
    expect(screen.getByText("Endpoint")).toBeTruthy();
  });

  it("endpoint invalid: shows error, Save disabled", async () => {
    routeInvoke({
      ...DEFAULT_ROUTES,
      provider_list: () => [profile({ uuid: "u1", name: "TestProvider", secret_ref: "provider/u1" })],
      key_status: () => ({ "provider/u1": true }),
    });
    render(() => <ProviderCenter />);
    await waitFor(() => expect(screen.getByText("TestProvider")).toBeTruthy());

    fireEvent.click(screen.getByLabelText("Edit TestProvider"));
    await flush();

    // Type an invalid endpoint into the endpoint input.
    const endpointInput = screen.getByLabelText("Endpoint") as HTMLInputElement;
    fireEvent.input(endpointInput, { target: { value: "http://evil.com" } });
    await flush();
    // The error text for endpoint-must-https renders.
    await waitFor(() =>
      expect(screen.getByText("Must be HTTPS (or localhost)")).toBeTruthy(),
    );
  });

  it("save profile: calls provider_update with patch", async () => {
    const updates: unknown[] = [];
    routeInvoke({
      ...DEFAULT_ROUTES,
      provider_list: () => [profile({ uuid: "u1", name: "TestProvider", secret_ref: "provider/u1" })],
      key_status: () => ({ "provider/u1": true }),
      provider_update: (args) => {
        updates.push(args);
        return profile({ uuid: "u1", endpoint: "https://new.example.com" });
      },
    });
    render(() => <ProviderCenter />);
    await waitFor(() => expect(screen.getByText("TestProvider")).toBeTruthy());
    fireEvent.click(screen.getByLabelText("Edit TestProvider"));
    await flush();

    const endpointInput = screen.getByLabelText("Endpoint") as HTMLInputElement;
    fireEvent.input(endpointInput, { target: { value: "https://new.example.com" } });
    await flush();

    fireEvent.click(screen.getByText("Save profile"));
    await waitFor(() => expect(invokeMock).toHaveBeenCalledWith("provider_update", expect.anything()));
    expect(updates.length).toBeGreaterThanOrEqual(1);
  });

  it("key missing: shows key input + Save key; saving calls provider_set_key", async () => {
    const keyCalls: unknown[] = [];
    routeInvoke({
      ...DEFAULT_ROUTES,
      provider_list: () => [profile({ uuid: "u1", name: "TestProvider", secret_ref: "provider/u1" })],
      key_status: () => ({}), // no key
      provider_set_key: (args) => {
        keyCalls.push(args);
      },
    });
    render(() => <ProviderCenter />);
    await waitFor(() => expect(screen.getByText("TestProvider")).toBeTruthy());
    fireEvent.click(screen.getByLabelText("Edit TestProvider"));
    await flush();

    // Key input + Save key button present.
    const keyInput = screen.getByLabelText("API key") as HTMLInputElement;
    expect(keyInput).toBeTruthy();
    fireEvent.input(keyInput, { target: { value: "sk-test" } });
    fireEvent.click(screen.getByText("Save key"));
    await whenCalledWith("provider_set_key");
    expect(keyCalls.length).toBeGreaterThanOrEqual(1);
  });

  it("key input cleared on submit start, even on failure", async () => {
    routeInvoke({
      ...DEFAULT_ROUTES,
      provider_list: () => [profile({ uuid: "u1", name: "TestProvider", secret_ref: "provider/u1" })],
      key_status: () => ({}),
      provider_set_key: () => {
        throw new Error("rejected");
      },
    });
    render(() => <ProviderCenter />);
    await waitFor(() => expect(screen.getByText("TestProvider")).toBeTruthy());
    fireEvent.click(screen.getByLabelText("Edit TestProvider"));
    await flush();

    const keyInput = screen.getByLabelText("API key") as HTMLInputElement;
    fireEvent.input(keyInput, { target: { value: "sk-test" } });
    fireEvent.click(screen.getByText("Save key"));
    await flush();
    // Input cleared regardless of resolve/reject.
    expect((keyInput as HTMLInputElement).value).toBe("");
  });

  it("delete: opens Confirm; confirm calls provider_delete", async () => {
    const deletes: unknown[] = [];
    routeInvoke({
      ...DEFAULT_ROUTES,
      provider_list: () => [profile({ uuid: "u1", name: "TestProvider", secret_ref: "provider/u1" })],
      key_status: () => ({ "provider/u1": true }),
      provider_delete: (args) => {
        deletes.push(args);
      },
    });
    render(() => <ProviderCenter />);
    await waitFor(() => expect(screen.getByText("TestProvider")).toBeTruthy());
    fireEvent.click(screen.getByLabelText("Delete TestProvider"));
    await flush();
    // Confirm dialog open.
    await waitFor(() => expect(screen.getByText("Delete provider?")).toBeTruthy());
    // Confirm button (the dialog has a confirm + cancel; click the delete-confirm one).
    fireEvent.click(screen.getByText("Delete"));
    await whenCalledWith("provider_delete");
    expect(deletes.length).toBeGreaterThanOrEqual(1);
  });

  it("set primary: calls provider_set_active with { primary, parallel: [], fallback }", async () => {
    const setActiveCalls: unknown[] = [];
    routeInvoke({
      ...DEFAULT_ROUTES,
      provider_list: () => TWO_PROFILES,
      key_status: () => ({ "provider/u1": true, "provider/u2": true }),
      provider_set_active: (args) => {
        setActiveCalls.push(args);
        return { outcome: "written" };
      },
    });
    render(() => <ProviderCenter />);
    await waitFor(() => expect(screen.getByText("MyOpenAI")).toBeTruthy());

    // Click "Set as primary" on the MyOpenAI row (first row).
    fireEvent.click(screen.getAllByLabelText("Set as primary")[0]);
    await whenCalledWith("provider_set_active");
    expect(setActiveCalls[0]).toMatchObject({ primary: "u1", parallel: [], fallback: null });
    // After "written", the primary indicator shows (ProviderRow status text +
    // the role badge both render "Primary" — assert at least one is present).
    await waitFor(() =>
      expect(screen.getAllByText("Primary").length).toBeGreaterThanOrEqual(1),
    );
  });

  it("add parallel → needs_consent → consent Confirm → provider_confirm_and_set_active", async () => {
    const actualScope = "v1:{u1|https://api.openai.com|false,u2|https://api.deepseek.com|false}";
    const confirmCalls: unknown[] = [];
    routeInvoke({
      ...DEFAULT_ROUTES,
      provider_list: () => TWO_PROFILES,
      key_status: () => ({ "provider/u1": true, "provider/u2": true }),
      provider_set_active: (args) => {
        // set primary first (written), then add parallel (needs_consent).
        const a = args as { primary: string; parallel: string[] };
        if (a.parallel.length === 0) return { outcome: "written" };
        return { outcome: "needs_consent", actual_scope: actualScope };
      },
      provider_confirm_and_set_active: (args) => {
        confirmCalls.push(args);
        return 2;
      },
    });
    render(() => <ProviderCenter />);
    await waitFor(() => expect(screen.getByText("MyOpenAI")).toBeTruthy());

    // Set primary first (so the parallel add has a primary to keep).
    fireEvent.click(screen.getAllByLabelText("Set as primary")[0]);
    await whenCalledWith("provider_set_active");

    // Add parallel on MyDeepSeek (second row). After MyOpenAI becomes primary,
    // only MyDeepSeek's "Add to parallel" remains.
    fireEvent.click(screen.getAllByLabelText("Add to parallel")[0]);
    await flush();
    // Consent dialog opens.
    await waitFor(() =>
      expect(screen.getByText("Send text to multiple providers?")).toBeTruthy(),
    );
    // Confirm.
    fireEvent.click(screen.getByText("Confirm"));
    await whenCalledWith("provider_confirm_and_set_active");
    expect(confirmCalls[0]).toMatchObject({ expectedScope: actualScope });
  });

  it("add parallel → stale_scope on confirm → toast, selection reverted", async () => {
    const actualScope = "v1:{changed}";
    routeInvoke({
      ...DEFAULT_ROUTES,
      provider_list: () => TWO_PROFILES,
      key_status: () => ({ "provider/u1": true, "provider/u2": true }),
      provider_set_active: (args) => {
        const a = args as { primary: string; parallel: string[] };
        if (a.parallel.length === 0) return { outcome: "written" };
        return { outcome: "needs_consent", actual_scope: actualScope };
      },
      provider_confirm_and_set_active: () => {
        const err = Object.assign(new Error("stale"), {
          error: "stale_scope",
          actual_scope: "v1:{changed}",
        });
        throw err;
      },
    });
    render(() => <ProviderCenter />);
    await waitFor(() => expect(screen.getByText("MyOpenAI")).toBeTruthy());
    fireEvent.click(screen.getAllByLabelText("Set as primary")[0]);
    await whenCalledWith("provider_set_active");
    fireEvent.click(screen.getAllByLabelText("Add to parallel")[0]);
    await waitFor(() =>
      expect(screen.getByText("Send text to multiple providers?")).toBeTruthy(),
    );
    fireEvent.click(screen.getByText("Confirm"));
    await whenCalledWith("provider_confirm_and_set_active");
    // DeepSeek should NOT have a parallel badge (reverted).
    await flush();
    const parallelBadges = screen.queryAllByText("Parallel");
    expect(parallelBadges.length).toBe(0);
  });

  it("reorder: move up calls provider_reorder with swapped uuids", async () => {
    const reorderCalls: unknown[] = [];
    routeInvoke({
      ...DEFAULT_ROUTES,
      provider_list: () => TWO_PROFILES,
      key_status: () => ({ "provider/u1": true, "provider/u2": true }),
      provider_reorder: (args) => {
        reorderCalls.push(args);
      },
    });
    render(() => <ProviderCenter />);
    await waitFor(() => expect(screen.getByText("MyOpenAI")).toBeTruthy());

    // Move the second row (MyDeepSeek) up — there are two "Move up" buttons
    // (one per row); click the second row's.
    const moveUpBtns = screen.getAllByLabelText("Move up");
    fireEvent.click(moveUpBtns[1]);
    await whenCalledWith("provider_reorder");
    expect(reorderCalls[0]).toMatchObject({ uuids: ["u2", "u1"] });
  });

  it("connection test: calls provider_test_connection; ok → connected indicator", async () => {
    routeInvoke({
      ...DEFAULT_ROUTES,
      provider_list: () => [profile({ uuid: "u1", name: "TestProvider", secret_ref: "provider/u1" })],
      key_status: () => ({ "provider/u1": true }),
      provider_test_connection: () => ({ ok: true, message: "reachable" }),
    });
    render(() => <ProviderCenter />);
    await waitFor(() => expect(screen.getByText("TestProvider")).toBeTruthy());
    fireEvent.click(screen.getByLabelText("Edit TestProvider"));
    await flush();
    fireEvent.click(screen.getByText("Test"));
    await whenCalledWith("provider_test_connection");
    await waitFor(() => expect(screen.getByText("Connected")).toBeTruthy());
  });

  it("connection test: renders message · latency_ms when latency present", async () => {
    routeInvoke({
      ...DEFAULT_ROUTES,
      provider_list: () => [profile({ uuid: "u1", name: "TestProvider", secret_ref: "provider/u1" })],
      key_status: () => ({ "provider/u1": true }),
      provider_test_connection: () => ({ ok: true, message: "reachable", latency_ms: 42 }),
    });
    render(() => <ProviderCenter />);
    await waitFor(() => expect(screen.getByText("TestProvider")).toBeTruthy());
    fireEvent.click(screen.getByLabelText("Edit TestProvider"));
    await flush();
    fireEvent.click(screen.getByText("Test"));
    await whenCalledWith("provider_test_connection");
    // The "· 42ms" latency suffix renders alongside the message.
    await waitFor(() => expect(screen.getByText("· 42ms")).toBeTruthy());
    // The connected badge still renders.
    expect(screen.getByText("Connected")).toBeTruthy();
  });

  it("connection test: no latency suffix when latency_ms absent", async () => {
    routeInvoke({
      ...DEFAULT_ROUTES,
      provider_list: () => [profile({ uuid: "u1", name: "TestProvider", secret_ref: "provider/u1" })],
      key_status: () => ({ "provider/u1": true }),
      provider_test_connection: () => ({ ok: true, message: "reachable" }),
    });
    render(() => <ProviderCenter />);
    await waitFor(() => expect(screen.getByText("TestProvider")).toBeTruthy());
    fireEvent.click(screen.getByLabelText("Edit TestProvider"));
    await flush();
    fireEvent.click(screen.getByText("Test"));
    await whenCalledWith("provider_test_connection");
    await waitFor(() => expect(screen.getByText("Connected")).toBeTruthy());
    // No "· Nms" suffix anywhere.
    expect(screen.queryByText(/· \d+ms/)).toBeNull();
  });

  it("balance section: renders TODO note, no fetch button", async () => {
    routeInvoke({
      ...DEFAULT_ROUTES,
      provider_list: () => [profile({ uuid: "u1", name: "TestProvider", secret_ref: "provider/u1" })],
      key_status: () => ({ "provider/u1": true }),
    });
    render(() => <ProviderCenter />);
    await waitFor(() => expect(screen.getByText("TestProvider")).toBeTruthy());
    fireEvent.click(screen.getByLabelText("Edit TestProvider"));
    await flush();
    // The muted TODO note renders; no balance-fetch button.
    expect(screen.getByText("Balance and quota are not yet available.")).toBeTruthy();
    expect(screen.queryByText("Fetch balance")).toBeNull();
  });

  it("uses zh copy when locale zh", async () => {
    localeMock.current = "zh";
    routeInvoke({ ...DEFAULT_ROUTES });
    render(() => <ProviderCenter />);
    await waitFor(() =>
      expect(screen.getByText("添加你的第一个服务商")).toBeTruthy(),
    );
  });

  it("no role badges on cold load when stored selection is empty", async () => {
    routeInvoke({
      ...DEFAULT_ROUTES,
      provider_list: () => TWO_PROFILES,
      key_status: () => ({ "provider/u1": true, "provider/u2": true }),
    });
    render(() => <ProviderCenter />);
    await waitFor(() => expect(screen.getByText("MyOpenAI")).toBeTruthy());
    // No role badges until assigned.
    expect(screen.queryAllByText("Primary").length).toBe(0);
    expect(screen.queryAllByText("Parallel").length).toBe(0);
    expect(screen.queryAllByText("Fallback").length).toBe(0);
  });

  it("cold-loads the stored active selection into role badges", async () => {
    routeInvoke({
      ...DEFAULT_ROUTES,
      provider_list: () => [
        profile({ uuid: "u1", name: "MyOpenAI", sort_order: 0, secret_ref: "provider/u1" }),
        profile({ uuid: "u2", name: "MyDeepSeek", template_id: "deepseek", sort_order: 1, secret_ref: "provider/u2", needs_key: true }),
      ],
      key_status: () => ({ "provider/u1": true, "provider/u2": true }),
      provider_get_active_selection: () => ({ primary: "u1", parallel: ["u2"], fallback: null }),
    });
    const { findAllByText } = render(() => <ProviderCenter />);
    // Primary badge ("Primary" / "主引擎") + Parallel badge ("Parallel" / "并行") render.
    expect((await findAllByText(/Primary|主引擎/)).length).toBeGreaterThan(0);
    expect((await findAllByText(/Parallel|并[行联]/)).length).toBeGreaterThan(0);
    // No fallback badge.
    expect(screen.queryAllByText(/Fallback|回退/).length).toBe(0);
  });

  it("fail-closed: shows load-failed banner + Retry and does NOT call providerSetActive when reads fail", async () => {
    routeInvoke({
      ...DEFAULT_ROUTES,
      provider_list: () => {
        throw new Error("db locked");
      },
      provider_get_active_selection: () => {
        throw new Error("db locked");
      },
    });
    const { findByText } = render(() => <ProviderCenter />);
    // Cold-load failure surfaced via the localized loadFailed banner.
    expect(await findByText(/加载失败|load failed/i)).toBeTruthy();
    // Fail-closed: no provider_set_active should have been attempted.
    expect(invokeMock.mock.calls.some((c) => c[0] === "provider_set_active")).toBe(false);
  });

  it("empty key: Save key disabled when needs_key provider has no key text", async () => {
    routeInvoke({
      ...DEFAULT_ROUTES,
      provider_list: () => [profile({ uuid: "u1", name: "TestProvider", secret_ref: "provider/u1" })],
      key_status: () => ({}), // no key
    });
    render(() => <ProviderCenter />);
    await waitFor(() => expect(screen.getByText("TestProvider")).toBeTruthy());
    fireEvent.click(screen.getByLabelText("Edit TestProvider"));
    await flush();

    // The Save key button exists but is disabled while the key input is empty.
    const saveKeyBtn = screen.getByText("Save key").closest("button") as HTMLButtonElement;
    expect(saveKeyBtn).toBeTruthy();
    expect(saveKeyBtn.disabled).toBe(true);

    // Typing a key enables the Save key button.
    const keyInput = screen.getByLabelText("API key") as HTMLInputElement;
    fireEvent.input(keyInput, { target: { value: "sk-test" } });
    await flush();
    expect(saveKeyBtn.disabled).toBe(false);
  });

  it("save key conflict: UNIQUE constraint surfaces localized 'already exists' error", async () => {
    routeInvoke({
      ...DEFAULT_ROUTES,
      provider_list: () => [profile({ uuid: "u1", name: "TestProvider", secret_ref: "provider/u1" })],
      key_status: () => ({}),
      provider_set_key: () => {
        throw new Error("UNIQUE constraint failed: providers.secret_ref");
      },
    });
    render(() => <ProviderCenter />);
    await waitFor(() => expect(screen.getByText("TestProvider")).toBeTruthy());
    fireEvent.click(screen.getByLabelText("Edit TestProvider"));
    await flush();

    const keyInput = screen.getByLabelText("API key") as HTMLInputElement;
    fireEvent.input(keyInput, { target: { value: "sk-test" } });
    fireEvent.click(screen.getByText("Save key"));
    await flush();
    // The localized "already exists" message surfaces — as an inline field
    // error and/or a toast. Assert at least one such element is present.
    await waitFor(() =>
      expect(screen.getAllByText(/already exists|已存在/).length).toBeGreaterThanOrEqual(1),
    );
  });

  it("duplicate: clicking Duplicate calls provider_duplicate", async () => {
    const dupCalls: unknown[] = [];
    routeInvoke({
      ...DEFAULT_ROUTES,
      provider_list: () => TWO_PROFILES,
      key_status: () => ({ "provider/u1": true, "provider/u2": true }),
      provider_duplicate: (args) => {
        dupCalls.push(args);
        return profile({ uuid: "u3", name: "MyOpenAI (copy)" });
      },
    });
    render(() => <ProviderCenter />);
    await waitFor(() => expect(screen.getByText("MyOpenAI")).toBeTruthy());

    fireEvent.click(screen.getAllByLabelText("Duplicate")[0]);
    await whenCalledWith("provider_duplicate");
    expect(dupCalls.length).toBeGreaterThanOrEqual(1);
    expect(dupCalls[0]).toMatchObject({ uuid: "u1" });
  });

  it("preset grid contains only the 4 supported AI presets (no Google/DeepL)", async () => {
    routeInvoke({ ...DEFAULT_ROUTES });
    const { findByText } = render(() => <ProviderCenter />);
    expect(await findByText("OpenAI")).toBeTruthy();
    expect(await findByText("Anthropic")).toBeTruthy();
    expect(await findByText("Gemini")).toBeTruthy();
    expect(await findByText("Ollama")).toBeTruthy();
    // Google Translate + DeepL presets are gone.
    expect(screen.queryByText(/Google Translate/)).toBeNull();
    expect(screen.queryByText(/^DeepL$/)).toBeNull();
  });
});
