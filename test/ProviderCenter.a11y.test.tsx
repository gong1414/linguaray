/**
 * Production a11y tests for ProviderCenter (Surface 05) — axe-core on the REAL
 * production component (not the deleted lab mock).
 *
 * Migrated from the deleted `apps/ui-lab/test/ProviderCenter.test.tsx`
 * (commit 7f21adc) which tested the lab mock fixture. These render the
 * production `ProviderCenter` controller against mocked invoke routes and run
 * axe against the full rendered output.
 *
 * color-contrast is excluded (jsdom cannot compute it); it is verified via the
 * MASTER token contrast table and by browser screenshots.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, fireEvent, cleanup, waitFor, screen } from "@solidjs/testing-library";
import { assertNoAxeViolations } from "./axe";

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
  version: 1,
  ...over,
});

const POPULATED: ProviderProfile[] = [
  profile({ uuid: "u1", name: "AlphaProvider", sort_order: 0, secret_ref: "provider/u1" }),
  profile({ uuid: "u2", name: "BetaProvider", sort_order: 1, secret_ref: "provider/u2" }),
];

function routeInvoke(routes: Record<string, (args?: unknown) => unknown>): void {
  invokeMock.mockImplementation(async (cmd: string, args?: unknown) => {
    const fn = routes[cmd];
    if (!fn) throw new Error(`unexpected invoke ${cmd}`);
    return fn(args);
  });
}

const DEFAULT_ROUTES: Record<string, (args?: unknown) => unknown> = {
  provider_list: () => POPULATED,
  key_status: () => ({ "provider/u1": true, "provider/u2": true }),
  provider_get_active_selection: () => ({ primary: "u1", parallel: [], fallback: null }),
};

beforeEach(() => {
  localeMock.current = "en";
  document.documentElement.dataset.theme = "light";
  invokeMock.mockReset();
  routeInvoke(DEFAULT_ROUTES);
});

afterEach(() => cleanup());

describe("ProviderCenter — accessibility (axe)", () => {
  it("has no axe violations on populated list (light/en)", async () => {
    render(() => <ProviderCenter />);
    // Wait for the provider list to render.
    await waitFor(() => expect(screen.getByText("AlphaProvider")).toBeTruthy());
    await assertNoAxeViolations({ disableRules: ["color-contrast"] });
  });

  it("has no axe violations in dark + Chinese", async () => {
    localeMock.current = "zh";
    document.documentElement.dataset.theme = "dark";
    render(() => <ProviderCenter />);
    await waitFor(() => expect(screen.getByText("AlphaProvider")).toBeTruthy());
    await assertNoAxeViolations({ disableRules: ["color-contrast"] });
  });

  it("has no axe violations on the detail panel with key-missing", async () => {
    routeInvoke({
      ...DEFAULT_ROUTES,
      key_status: () => ({}), // u1 has no key → key input renders
    });
    render(() => <ProviderCenter />);
    await waitFor(() => expect(screen.getByText("AlphaProvider")).toBeTruthy());
    fireEvent.click(screen.getByLabelText("Edit AlphaProvider"));
    // Wait for the key input to render.
    await waitFor(() => expect(screen.getByLabelText("API key")).toBeTruthy());
    await assertNoAxeViolations({ disableRules: ["color-contrast"] });
  });

  it("has no axe violations on the delete Confirm dialog", async () => {
    render(() => <ProviderCenter />);
    await waitFor(() => expect(screen.getByText("AlphaProvider")).toBeTruthy());
    fireEvent.click(screen.getByLabelText("Delete AlphaProvider"));
    await waitFor(() => expect(screen.getByText("Delete provider?")).toBeTruthy());
    await assertNoAxeViolations({ disableRules: ["color-contrast"] });
  });
});
