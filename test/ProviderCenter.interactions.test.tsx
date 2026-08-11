/**
 * Production contract tests for ProviderCenter (Surface 05).
 *
 * Migrated from the deleted `apps/ui-lab/test/ProviderCenter.interactions.test.tsx`
 * (commit 7f21adc) which tested the lab mock fixture with fake timers. These
 * drive the REAL production ProviderCenter controller against mocked invoke
 * routes and verify cross-provider isolation + async-safety contracts:
 *
 *  - save-key ABA: per-UUID key state is not polluted across providers.
 *  - connection test ABA: a stale completion does not overwrite a newer result.
 *  - delete focus: after the deleted row is removed, focus lands on a safe
 *    fallback (not lost to body).
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
  it("save-key ABA: saving key for u1 does not pollute u2's key state", async () => {
    routeInvoke({
      ...DEFAULT_ROUTES,
      provider_set_key: () => {},
    });
    render(() => <ProviderCenter />);
    await waitFor(() => expect(screen.getByText("Alpha")).toBeTruthy());

    // Select u1 and type a key.
    fireEvent.click(screen.getByLabelText("Edit Alpha"));
    await flush();
    const keyInput = screen.getByLabelText("API key") as HTMLInputElement;
    fireEvent.input(keyInput, { target: { value: "sk-alpha-secret" } });
    await flush();
    expect(keyInput.value).toBe("sk-alpha-secret");

    // Switch to u2 — its key input must be EMPTY (u1's key text must not leak).
    fireEvent.click(screen.getByLabelText("Edit Beta"));
    await flush();
    const u2KeyInput = screen.getByLabelText("API key") as HTMLInputElement;
    expect(u2KeyInput.value).toBe("");

    // Type a different key for u2.
    fireEvent.input(u2KeyInput, { target: { value: "sk-beta-secret" } });
    await flush();
    expect(u2KeyInput.value).toBe("sk-beta-secret");

    // Switch back to u1 — its key draft is PRESERVED (ABA intact).
    fireEvent.click(screen.getByLabelText("Edit Alpha"));
    await flush();
    const u1KeyInputAgain = screen.getByLabelText("API key") as HTMLInputElement;
    expect(u1KeyInputAgain.value).toBe("sk-alpha-secret");
  });

  it("connection test ABA: stale completion does not overwrite newer result", async () => {
    // Per-UUID isolation: testing u1 and switching to u2 must NOT show u1's
    // connection result on u2's panel. A stale u1 completion arriving while u2
    // is selected stays in connByUuid[u1] and never bleeds into u2.
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
    // u1's test is pending (spinner). The Test button is disabled (loading).

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

    // Confirm the delete → u1's row is removed by refresh.
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
});
