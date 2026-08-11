/**
 * Production contract tests for ProviderCenter (Surface 05).
 *
 * Migrated from the deleted `apps/ui-lab/test/ProviderCenter.interactions.test.tsx`
 * (commit 7f21adc) which tested the lab mock fixture with fake timers. These
 * drive the REAL production ProviderCenter controller against mocked invoke
 * routes and verify cross-provider isolation + async-safety contracts:
 *
 *  - save-key isolation: per-UUID key state is not polluted across providers.
 *    The async mutex (`runExclusive`) structurally eliminates the ABA race —
 *    two key saves for different providers can never overlap — so this verifies
 *    cross-UUID isolation after a completed save, NOT a true ABA window.
 *  - connection test isolation (cross-UUID): a stale completion does not
 *    overwrite another provider's panel.
 *  - connection test config-version binding (same-UUID): a Test started against
 *    config version N is discarded if a save bumps the config to version N+1
 *    before the probe resolves (R8-P1). This is the true same-UUID ABA window
 *    the mutex CANNOT close — Test/Fetch run outside the mutex — closed by a
 *    config-version guard instead of the requestId counter alone.
 *  - fetch models config-version binding (same-UUID): same guard for Fetch
 *    Models (R8-P2-1).
 *  - refreshCore failure feedback: a successful mutation whose list-refresh
 *    fails shows the warning, not the success toast, and skips DOM-dependent
 *    focus restoration (R8-P2-2).
 *  - delete focus: after the deleted row is removed, focus lands on a safe
 *    fallback (not lost to body).
 *
 * R7-P1-1: the boolean globalMutationLock was replaced by an async mutex
 * (`runExclusive`). Three new tests verify the serial-operation contract:
 *  - mutation in-flight disables ALL controls until it completes
 *  - refresh (initial load) disables ALL controls until it completes
 *  - a mutation that internally calls refreshCore does NOT deadlock
 */
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, fireEvent, cleanup, waitFor, screen } from "@solidjs/testing-library";

const { invokeMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(async (_cmd: string, _args?: unknown): Promise<unknown> => {
    throw new Error(`unexpected invoke ${_cmd}`);
  }),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));
vi.mock("../src/i18n", () => ({ detectLocale: () => "en" }));

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
  capabilities: { balance: false, quota: false, model_list: false },
  status: "active",
  version: 1,
  ...over,
});

function routeInvoke(routes: Record<string, (args?: unknown) => unknown>): void {
  invokeMock.mockImplementation(async (cmd: string, args?: unknown) => {
    const fn = routes[cmd];
    if (!fn) throw new Error(`unexpected invoke ${cmd}`);
    return fn(args);
  });
}

const TWO_NO_KEY: ProviderProfile[] = [
  profile({ uuid: "u1", name: "Alpha", sort_order: 0, secret_ref: "provider/u1" }),
  profile({ uuid: "u2", name: "Beta", sort_order: 1, secret_ref: "provider/u2" }),
];

const DEFAULT_ROUTES: Record<string, (args?: unknown) => unknown> = {
  provider_list: () => TWO_NO_KEY,
  key_status: () => ({}), // both keyless
  provider_get_active_selection: () => ({ primary: null, parallel: [], fallback: null }),
};

beforeEach(() => {
  invokeMock.mockReset();
  routeInvoke(DEFAULT_ROUTES);
});

afterEach(() => cleanup());

describe("ProviderCenter — production interaction contracts", () => {
  it("save-key isolation: completed save for one provider does not pollute another's key state", async () => {
    // R8-P2-3: this is cross-UUID isolation after a COMPLETED save, not a true
    // ABA race. The async mutex (`runExclusive`) serializes every mutation, so
    // two key saves for different providers can never overlap — the mutex
    // structurally eliminates the ABA window. The test still has value: it
    // verifies per-UUID key state does not leak across providers once a save
    // settles. For the real same-UUID config-version ABA that the mutex CANNOT
    // close (Test/Fetch run outside the mutex), see the connection-test and
    // fetch-models config-version binding tests below.
    let u1HasKey = false;
    routeInvoke({
      ...DEFAULT_ROUTES,
      key_status: () => ({ "provider/u1": u1HasKey }),
      provider_set_key: () => {
        u1HasKey = true;
      },
    });
    render(() => <ProviderCenter />);
    await waitFor(() => expect(screen.getByText("Alpha")).toBeTruthy());

    // Select u1 and type + save a key. provider_set_key resolves immediately.
    fireEvent.click(screen.getByLabelText("Edit Alpha"));
    await flush();
    const keyInput = screen.getByLabelText("API key") as HTMLInputElement;
    fireEvent.input(keyInput, { target: { value: "sk-alpha-secret" } });
    await flush();
    expect(keyInput.value).toBe("sk-alpha-secret");
    fireEvent.click(screen.getByText("Save key"));
    await flush();
    // After save: u1 re-fetches providers; u1 now hasKey → "Key saved" badge.
    // ("Key saved" also appears in the success toast, so assert at least one.)
    await waitFor(() =>
      expect(screen.getAllByText("Key saved").length).toBeGreaterThanOrEqual(1),
    );

    // Switch to u2 — its key state is UNCHANGED (keyless, no pollution).
    fireEvent.click(screen.getByLabelText("Edit Beta"));
    await flush();
    const u2KeyInput = screen.getByLabelText("API key") as HTMLInputElement;
    expect(u2KeyInput.value).toBe("");

    // Type a different key for u2.
    fireEvent.input(u2KeyInput, { target: { value: "sk-beta-secret" } });
    await flush();
    expect(u2KeyInput.value).toBe("sk-beta-secret");

    // Switch back to u1 — its saved state is intact (badge, no key input).
    fireEvent.click(screen.getByLabelText("Edit Alpha"));
    await flush();
    // u1 shows the "Key saved" badge (not the key input) — ABA isolation.
    expect(screen.queryByLabelText("API key")).toBeNull();
  });

  it("connection test ABA: stale completion does not overwrite newer result", async () => {
    // Cross-UUID isolation: testing u1 and switching to u2 must NOT show u1's
    // connection result on u2's panel. A stale u1 completion arriving while u2
    // is selected stays in connByUuid[u1] and never bleeds into u2.
    // (Same-UUID overlapping tests are architecturally blocked: the Test
    // button auto-disables during loading, so two rapid clicks on the same
    // provider cannot fire. The per-UUID requestId guard covers this
    // unreachable race as defense-in-depth.)
    let resolveU1: (r: { ok: boolean; message: string }) => void = () => {};
    routeInvoke({
      ...DEFAULT_ROUTES,
      provider_test_connection: (args) => {
        const a = args as { uuid: string };
        if (a.uuid === "u1") {
          return new Promise((res) => {
            resolveU1 = res as (r: { ok: boolean; message: string }) => void;
          });
        }
        // u2 completes immediately with a success.
        return { ok: true, message: "u2 reachable" };
      },
    });
    render(() => <ProviderCenter />);
    await waitFor(() => expect(screen.getByText("Alpha")).toBeTruthy());

    // Select u1 and start a connection test (deferred — stays pending).
    fireEvent.click(screen.getByLabelText("Edit Alpha"));
    await flush();
    fireEvent.click(screen.getByText("Test"));
    await flush();

    // Switch to u2 and test it — u2 completes immediately with "u2 reachable".
    fireEvent.click(screen.getByLabelText("Edit Beta"));
    await flush();
    fireEvent.click(screen.getByText("Test"));
    await flush();
    await waitFor(() => expect(screen.getByText("u2 reachable")).toBeTruthy());
    await waitFor(() => expect(screen.getByText("Connected")).toBeTruthy());

    // Now u1's deferred test resolves. It must NOT overwrite u2's panel — the
    // result lands in connByUuid[u1] only, and u2's display stays intact.
    resolveU1({ ok: false, message: "u1 unreachable (stale)" });
    await flush();
    await flush();
    // u2's result is still showing (not overwritten by u1's stale completion).
    expect(screen.getByText("u2 reachable")).toBeTruthy();
    expect(screen.queryByText("u1 unreachable (stale)")).toBeNull();

    // Switch back to u1 — its stale result IS there (per-UUID isolation).
    fireEvent.click(screen.getByLabelText("Edit Alpha"));
    await flush();
    await waitFor(() => expect(screen.getByText("u1 unreachable (stale)")).toBeTruthy());
  });

  it("connection test config-version binding: a Test started against an old config is discarded after a save bumps the version (R8-P1)", async () => {
    // R8-P1: the true same-UUID ABA window. `providerTestConnection` probes the
    // BACKEND's stored config. If the user starts a Test, then saves a NEW
    // endpoint (bumping the config version), the in-flight Test still resolves
    // against the OLD endpoint. Without the config-version guard, its stale
    // "Connected" would overwrite the cleared result next to the freshly-saved
    // row. The mutex CANNOT close this — Test runs outside runExclusive — so a
    // config-version guard discards the stale completion.
    let resolveFirstTest!: (r: { ok: boolean; message: string }) => void;
    let testCallCount = 0;
    routeInvoke({
      ...DEFAULT_ROUTES,
      provider_test_connection: () => {
        testCallCount++;
        if (testCallCount === 1) {
          // First Test stays pending across the save.
          return new Promise((res) => {
            resolveFirstTest = res as (r: { ok: boolean; message: string }) => void;
          });
        }
        // Second Test (after save) resolves immediately against the new config.
        return { ok: true, message: "new reachable" };
      },
      provider_update: () =>
        profile({ uuid: "u1", name: "Alpha", endpoint: "https://new.example.com", version: 2 }),
    });
    render(() => <ProviderCenter />);
    await waitFor(() => expect(screen.getByText("Alpha")).toBeTruthy());

    // Select u1 (version=1, old endpoint) and start a Test — stays pending.
    fireEvent.click(screen.getByLabelText("Edit Alpha"));
    await flush();
    // No drafts yet → Test button is enabled.
    expect((screen.getByText("Test").closest("button") as HTMLButtonElement).disabled).toBe(false);
    fireEvent.click(screen.getByText("Test"));
    await flush();

    // Edit the endpoint → unsaved drafts disable the Test button + surface the
    // hint, and clear any prior connection result for this UUID.
    const ep = screen.getByLabelText("Endpoint") as HTMLInputElement;
    fireEvent.input(ep, { target: { value: "https://new.example.com" } });
    await flush();
    await waitFor(() =>
      expect((screen.getByText("Test").closest("button") as HTMLButtonElement).disabled).toBe(true),
    );
    expect(screen.getByText("Save changes before testing")).toBeTruthy();

    // Save → provider_update returns version=2 + the new endpoint. The Test
    // button re-enables (the saved value now matches the draft).
    fireEvent.click(screen.getByText("Save profile"));
    await flush();
    await waitFor(() =>
      expect((screen.getByText("Test").closest("button") as HTMLButtonElement).disabled).toBe(false),
    );

    // The OLD Test (against version=1) now resolves. Its stale result MUST be
    // discarded — not shown as "Connected" or "old reachable".
    resolveFirstTest({ ok: true, message: "old reachable" });
    await flush();
    await flush();
    expect(screen.queryByText("old reachable")).toBeNull();
    expect(screen.queryByText("Connected")).toBeNull();

    // A fresh Test against the saved config (version=2) writes its result.
    fireEvent.click(screen.getByText("Test"));
    await flush();
    await waitFor(() => expect(screen.getByText("new reachable")).toBeTruthy());
    await waitFor(() => expect(screen.getByText("Connected")).toBeTruthy());
  });

  it("fetch models config-version binding: a Fetch started against an old config is discarded after a save bumps the version (R8-P2-1)", async () => {
    // R8-P2-1: same ABA window as the connection test, but for Fetch Models.
    // A Fetch started against version=1 whose await resolves AFTER a save bumps
    // the config to version=2 must NOT write its (stale) model list — the
    // config-version guard discards it.
    //
    // Observable strategy: the Model Select's trigger displays the label of the
    // option matching the current modelDraft ("gpt-4o-mini"). While the fetched
    // model list is empty, the only option IS the modelDraft, so the trigger
    // shows "gpt-4o-mini". If the STALE list were written, its options would
    // REPLACE the fallback and "gpt-4o-mini" would no longer match any option —
    // the trigger would fall back to its (empty) placeholder. So "trigger still
    // shows gpt-4o-mini after the stale resolve" proves the guard discarded it.
    const listable = (over: Partial<ProviderProfile> = {}): ProviderProfile =>
      profile({
        uuid: "u1",
        name: "Alpha",
        sort_order: 0,
        secret_ref: "provider/u1",
        capabilities: { balance: false, quota: false, model_list: true },
        ...over,
      });
    let resolveFirstFetch!: (m: { id: string; label: string }[]) => void;
    let fetchCallCount = 0;
    let saved = false;
    routeInvoke({
      ...DEFAULT_ROUTES,
      provider_list: () => [listable(saved ? { endpoint: "https://new.example.com", version: 2 } : {})],
      provider_get_models: () => {
        fetchCallCount++;
        if (fetchCallCount === 1) {
          // First Fetch stays pending across the save.
          return new Promise((res) => {
            resolveFirstFetch = res as (m: { id: string; label: string }[]) => void;
          });
        }
        // Second Fetch (after save) returns a list that includes the current
        // modelDraft so the trigger keeps its match.
        return [
          { id: "gpt-4o-mini", label: "gpt-4o-mini" },
          { id: "new-model", label: "New Model" },
        ];
      },
      provider_update: () => {
        saved = true;
        return listable({ endpoint: "https://new.example.com", version: 2 });
      },
    });
    render(() => <ProviderCenter />);
    await waitFor(() => expect(screen.getByText("Alpha")).toBeTruthy());

    const triggerValue = () =>
      (document.querySelector(".lr-select__value") as HTMLElement | null)?.textContent ?? "";

    // Select u1 and start a Fetch — stays pending.
    fireEvent.click(screen.getByLabelText("Edit Alpha"));
    await flush();
    await waitFor(() => expect(triggerValue()).toBe("gpt-4o-mini"));
    fireEvent.click(screen.getByText("Fetch models"));
    await flush();

    // Edit the endpoint + save → version bumps to 2 (clears model state).
    const ep = screen.getByLabelText("Endpoint") as HTMLInputElement;
    fireEvent.input(ep, { target: { value: "https://new.example.com" } });
    await flush();
    fireEvent.click(screen.getByText("Save profile"));
    await flush();
    await waitFor(() =>
      expect(screen.getAllByText("Profile saved").length).toBeGreaterThanOrEqual(1),
    );
    // After save the model state was cleared → trigger back to the modelDraft.
    await waitFor(() => expect(triggerValue()).toBe("gpt-4o-mini"));

    // The OLD Fetch (against version=1) resolves with stale models. They MUST
    // be discarded — the trigger must NOT lose its match (which would happen if
    // the stale list replaced the fallback options).
    resolveFirstFetch([{ id: "stale-model", label: "Stale Model" }]);
    await flush();
    await flush();
    expect(triggerValue()).toBe("gpt-4o-mini");
    expect(screen.queryByText("Stale Model")).toBeNull();

    // A fresh Fetch against the saved config writes the new model list (which
    // includes the current modelDraft, so the trigger keeps its match and the
    // fetch settles to idle — no loading spinner, no error toast).
    fireEvent.click(screen.getByText("Fetch models"));
    await flush();
    await waitFor(() => expect(triggerValue()).toBe("gpt-4o-mini"));
    // No fetch-error toast surfaced.
    expect(screen.queryByText("Failed to fetch models — enter manually")).toBeNull();
    expect(fetchCallCount).toBe(2);
  });

  it("refreshCore failure: a successful mutation whose list-refresh fails shows the warning, not the success toast (R8-P2-2)", async () => {
    // R8-P2-2: provider_delete succeeds, but the post-delete refreshCore fails
    // (provider_list rejects). confirmDelete must NOT run the DOM-dependent
    // focus restoration, and must show the "saved but reload failed" warning
    // instead of the (now-misleading) success path. refreshCore already pushes
    // its own destructive toast + sets loadError; the warning adds the accurate
    // context that the delete itself went through.
    let deleted = false;
    let refreshShouldFail = false;
    routeInvoke({
      ...DEFAULT_ROUTES,
      provider_delete: () => {
        deleted = true;
      },
      provider_list: () => {
        if (refreshShouldFail) throw new Error("reload failed");
        return deleted
          ? [profile({ uuid: "u2", name: "Beta", sort_order: 0, secret_ref: "provider/u2" })]
          : TWO_NO_KEY;
      },
      provider_get_active_selection: () => {
        if (refreshShouldFail) throw new Error("reload failed");
        return { primary: null, parallel: [], fallback: null };
      },
    });
    render(() => <ProviderCenter />);
    await waitFor(() => expect(screen.getByText("Alpha")).toBeTruthy());

    // Make the post-delete refresh fail.
    refreshShouldFail = true;
    fireEvent.click(screen.getByLabelText("Delete Alpha"));
    await waitFor(() => expect(screen.getByText("Delete provider?")).toBeTruthy());
    fireEvent.click(screen.getByText("Delete"));
    await flush();

    // The delete went through on the backend; the reload failed. The warning
    // surfaces (refreshCore's destructive toast is also present).
    await waitFor(() =>
      expect(
        screen.getByText("Saved, but the list could not be refreshed. Click Reload to retry."),
      ).toBeTruthy(),
    );
    expect(deleted).toBe(true);
    // The success toast must NOT have appeared.
    expect(screen.queryByText("Profile saved")).toBeNull();
  });

  it("delete: focus falls to a safe fallback when the trigger row is removed", async () => {
    let deleted = false;
    routeInvoke({
      ...DEFAULT_ROUTES,
      provider_delete: () => {
        deleted = true;
      },
      // After delete, refresh returns only u2 (u1's row is gone).
      provider_list: () =>
        deleted
          ? [profile({ uuid: "u2", name: "Beta", sort_order: 0, secret_ref: "provider/u2" })]
          : TWO_NO_KEY,
    });
    render(() => <ProviderCenter />);
    await waitFor(() => expect(screen.getByText("Alpha")).toBeTruthy());

    // Open the delete Confirm for u1 (Alpha).
    fireEvent.click(screen.getByLabelText("Delete Alpha"));
    await waitFor(() => expect(screen.getByText("Delete provider?")).toBeTruthy());

    // Confirm the delete → u1's row is removed by refreshCore.
    fireEvent.click(screen.getByText("Delete"));
    await waitFor(() => expect(screen.queryByText("Alpha")).toBeNull());
    // u2 (Beta) remains.
    await waitFor(() => expect(screen.getByText("Beta")).toBeTruthy());

    // Focus must NOT be lost to body. It should land on a focusable element
    // within the ProviderCenter (the safe fallback — u2's Edit button or a
    // preset button). The deleted trigger (Alpha's Delete button) is gone.
    await waitFor(() => {
      const active = document.activeElement;
      expect(active).toBeTruthy();
      expect(active).not.toBe(document.body);
    });
    // Specifically: focus landed on Beta's Edit button (the first remaining row).
    await waitFor(() => {
      expect(document.activeElement).toBe(screen.getByLabelText("Edit Beta"));
    });
  });

  // ─── R7-P1-1: Serial operation queue (async mutex) ──────────────────────

  it("mutex: mutation in-flight disables ALL controls until it completes", async () => {
    // A mutation (Save profile) acquires the mutex. While held, EVERY control
    // for EVERY provider is disabled — no button appears enabled but silently
    // returns. After the mutation completes, controls re-enable.
    let resolveUpdate!: (p: ProviderProfile) => void;
    routeInvoke({
      ...DEFAULT_ROUTES,
      provider_update: () =>
        new Promise<ProviderProfile>((res) => {
          resolveUpdate = res;
        }),
    });
    render(() => <ProviderCenter />);
    await waitFor(() => expect(screen.getByText("Alpha")).toBeTruthy());

    // Select u1 and trigger a save — provider_update is deferred (mutex held).
    fireEvent.click(screen.getByLabelText("Edit Alpha"));
    await flush();
    const ep = screen.getByLabelText("Endpoint") as HTMLInputElement;
    fireEvent.input(ep, { target: { value: "https://new.example.com" } });
    await flush();
    fireEvent.click(screen.getByText("Save profile"));
    await flush();

    // While the mutation is in-flight: u2's sidebar buttons are ALL disabled.
    await waitFor(() =>
      expect((screen.getByLabelText("Edit Beta") as HTMLButtonElement).disabled).toBe(true),
    );
    expect((screen.getAllByLabelText("Duplicate")[1] as HTMLButtonElement).disabled).toBe(true);
    expect((screen.getAllByLabelText("Move up")[1] as HTMLButtonElement).disabled).toBe(true);
    // Detail panel controls also disabled (Name, Endpoint).
    expect((screen.getByLabelText("Endpoint") as HTMLInputElement).disabled).toBe(true);
    expect((screen.getByLabelText("Name") as HTMLInputElement).disabled).toBe(true);

    // Resolve the mutation → mutex released → controls re-enabled.
    resolveUpdate(profile({ uuid: "u1", name: "Alpha", endpoint: "https://new.example.com", version: 2 }));
    await flush();
    await waitFor(() =>
      expect((screen.getByLabelText("Edit Beta") as HTMLButtonElement).disabled).toBe(false),
    );
  });

  it("mutex: refresh (initial load) disables ALL controls until it completes", async () => {
    // The initial onMount refresh acquires the mutex. While the deferred
    // provider_list is pending, preset buttons are disabled. After it resolves,
    // providers render and controls enable.
    let resolveList!: (p: ProviderProfile[]) => void;
    routeInvoke({
      ...DEFAULT_ROUTES,
      provider_list: () =>
        new Promise<ProviderProfile[]>((res) => {
          resolveList = res;
        }),
    });
    render(() => <ProviderCenter />);
    // Let the initial onMount refresh start (provider_list deferred → mutex held).
    await waitFor(() => expect(invokeMock).toHaveBeenCalledWith("provider_list"));

    // While the load is in-flight: preset buttons are rendered but disabled.
    // (getByText returns the inner <span>; .closest("button") gets the button.)
    await waitFor(() => {
      const presetBtn = screen.getByText("OpenAI").closest("button") as HTMLButtonElement;
      expect(presetBtn).toBeTruthy();
      expect(presetBtn.disabled).toBe(true);
    });

    // Resolve the load → providers render, preset buttons enable.
    resolveList(TWO_NO_KEY);
    await waitFor(() => expect(screen.getByText("Alpha")).toBeTruthy());
    await waitFor(() => {
      const presetBtn = screen.getByText("OpenAI").closest("button") as HTMLButtonElement;
      expect(presetBtn.disabled).toBe(false);
    });
  });

  it("mutex: mutation with internal refreshCore does not deadlock", async () => {
    // handleAddPreset wraps in runExclusive AND calls refreshCore() inside the
    // mutex body. refreshCore() does NOT re-enter the mutex (it's the raw
    // read+apply), so there is no deadlock. If this deadlocked, the test
    // would time out.
    let created = false;
    routeInvoke({
      ...DEFAULT_ROUTES,
      provider_list: () => (created ? TWO_NO_KEY : []),
      provider_create: () => {
        created = true;
        return profile({ uuid: "u1", name: "Alpha" });
      },
    });
    render(() => <ProviderCenter />);
    // Wait for the initial load to complete (preset button enables).
    await waitFor(() => {
      const presetBtn = screen.getByText("OpenAI").closest("button") as HTMLButtonElement;
      expect(presetBtn.disabled).toBe(false);
    });

    // Click a preset → create runs → calls refreshCore INSIDE the mutex.
    fireEvent.click(screen.getByText("OpenAI").closest("button")!);
    // The create + refreshCore completed without deadlock → providers rendered.
    await waitFor(() => expect(screen.getByText("Alpha")).toBeTruthy());
    await waitFor(() => expect(screen.getByText("Beta")).toBeTruthy());
  });
});
