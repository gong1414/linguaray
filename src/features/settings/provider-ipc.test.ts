/**
 * Unit tests for the typed `invoke` wrappers in `provider-ipc.ts`.
 *
 * `@tauri-apps/api/core` `invoke` is mocked with `vi.fn()` so the tests assert
 * the exact command name + camelCase argument shape Tauri v2 expects on the JS
 * side (snake_case Rust params are camelCased by the `tauri::command` macro).
 */
import { describe, it, expect, vi, beforeEach } from "vitest";

// Mock `invoke` before importing the wrapper module.
const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invokeMock(...args),
}));

import {
  loadProviders,
  providerCreate,
  providerUpdate,
  providerDuplicate,
  providerDelete,
  providerReorder,
  providerToggle,
  providerSetKey,
  providerSetActive,
  providerConfirmAndSetActive,
  providerGetModels,
  providerTestConnection,
  keystoreHealth,
  archiveKeystore,
  resetKeystore,
  keyStatus,
} from "./provider-ipc";
import type { ProviderProfile } from "./provider-types";

const profile = (over: Partial<ProviderProfile> = {}): ProviderProfile => ({
  uuid: "u1",
  template_id: "openai",
  name: "OpenAI",
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
  ...over,
});

describe("loadProviders", () => {
  beforeEach(() => invokeMock.mockReset());

  it("calls provider_list + key_status and joins hasKey", async () => {
    const list = [
      profile({ uuid: "u1", secret_ref: "provider/u1" }),
      profile({ uuid: "u2", secret_ref: "provider/u2" }),
    ];
    invokeMock
      .mockResolvedValueOnce(list) // provider_list
      .mockResolvedValueOnce({ "provider/u1": true }); // key_status

    const result = await loadProviders();

    expect(invokeMock).toHaveBeenNthCalledWith(1, "provider_list");
    expect(invokeMock).toHaveBeenNthCalledWith(2, "key_status");
    expect(result).toHaveLength(2);
    expect(result[0]).toMatchObject({ uuid: "u1", hasKey: true });
    expect(result[1]).toMatchObject({ uuid: "u2", hasKey: false });
  });
});

describe("provider command wrappers", () => {
  beforeEach(() => invokeMock.mockReset());

  it("providerCreate invokes with { templateId, name, endpoint, model }", async () => {
    invokeMock.mockResolvedValue(profile());
    await providerCreate("openai", "OpenAI", "https://api.openai.com", "gpt-4o");
    expect(invokeMock).toHaveBeenCalledWith("provider_create", {
      templateId: "openai",
      name: "OpenAI",
      endpoint: "https://api.openai.com",
      model: "gpt-4o",
    });
  });

  it("providerCreate sends model: null when omitted", async () => {
    invokeMock.mockResolvedValue(profile());
    await providerCreate("openai", "OpenAI", "https://api.openai.com");
    expect(invokeMock).toHaveBeenCalledWith("provider_create", {
      templateId: "openai",
      name: "OpenAI",
      endpoint: "https://api.openai.com",
      model: null,
    });
  });

  it("providerUpdate invokes with { uuid, patch }", async () => {
    invokeMock.mockResolvedValue(profile());
    await providerUpdate("u1", { endpoint: "https://new.example.com" });
    expect(invokeMock).toHaveBeenCalledWith("provider_update", {
      uuid: "u1",
      patch: { endpoint: "https://new.example.com" },
    });
  });

  it("providerDuplicate invokes with { uuid }", async () => {
    invokeMock.mockResolvedValue(profile({ uuid: "u2" }));
    await providerDuplicate("u1");
    expect(invokeMock).toHaveBeenCalledWith("provider_duplicate", { uuid: "u1" });
  });

  it("providerDelete invokes with { uuid }", async () => {
    invokeMock.mockResolvedValue(undefined);
    await providerDelete("u1");
    expect(invokeMock).toHaveBeenCalledWith("provider_delete", { uuid: "u1" });
  });

  it("providerReorder invokes with { uuids }", async () => {
    invokeMock.mockResolvedValue(undefined);
    await providerReorder(["u2", "u1"]);
    expect(invokeMock).toHaveBeenCalledWith("provider_reorder", {
      uuids: ["u2", "u1"],
    });
  });

  it("providerToggle invokes with { uuid, enabled }", async () => {
    invokeMock.mockResolvedValue(undefined);
    await providerToggle("u1", false);
    expect(invokeMock).toHaveBeenCalledWith("provider_toggle", {
      uuid: "u1",
      enabled: false,
    });
  });

  it("providerSetKey invokes with { uuid, key }", async () => {
    invokeMock.mockResolvedValue(undefined);
    await providerSetKey("u1", "sk-secret");
    expect(invokeMock).toHaveBeenCalledWith("provider_set_key", {
      uuid: "u1",
      key: "sk-secret",
    });
  });

  it("providerSetActive returns SetActiveResult (written)", async () => {
    invokeMock.mockResolvedValue({ outcome: "written" });
    const r = await providerSetActive("u1", [], null);
    expect(r).toEqual({ outcome: "written" });
    expect(invokeMock).toHaveBeenCalledWith("provider_set_active", {
      primary: "u1",
      parallel: [],
      fallback: null,
    });
  });

  it("providerSetActive returns SetActiveResult (needs_consent)", async () => {
    invokeMock.mockResolvedValue({
      outcome: "needs_consent",
      actual_scope: "v1:{u1|https://a.example.com|false}",
    });
    const r = await providerSetActive("u1", ["u2"], "u3");
    expect(r).toEqual({
      outcome: "needs_consent",
      actual_scope: "v1:{u1|https://a.example.com|false}",
    });
    expect(invokeMock).toHaveBeenCalledWith("provider_set_active", {
      primary: "u1",
      parallel: ["u2"],
      fallback: "u3",
    });
  });

  it("providerConfirmAndSetActive returns i64 (consent version)", async () => {
    invokeMock.mockResolvedValue(3);
    const v = await providerConfirmAndSetActive("u1", ["u2"], null, "v1:{...}");
    expect(v).toBe(3);
    expect(invokeMock).toHaveBeenCalledWith("provider_confirm_and_set_active", {
      primary: "u1",
      parallel: ["u2"],
      fallback: null,
      expectedScope: "v1:{...}",
    });
  });

  it("providerConfirmAndSetActive surfaces stale_scope error (rejection propagates)", async () => {
    const stale = { error: "stale_scope", actual_scope: "v1:{changed}" };
    invokeMock.mockRejectedValue(stale);
    await expect(
      providerConfirmAndSetActive("u1", ["u2"], null, "v1:{old}"),
    ).rejects.toEqual(stale);
  });

  it("providerGetModels invokes with { uuid }", async () => {
    invokeMock.mockResolvedValue([{ id: "gpt-4o", label: "gpt-4o" }]);
    await providerGetModels("u1");
    expect(invokeMock).toHaveBeenCalledWith("provider_get_models", { uuid: "u1" });
  });

  it("providerTestConnection invokes with { uuid }", async () => {
    invokeMock.mockResolvedValue({ ok: true, message: "reachable" });
    await providerTestConnection("u1");
    expect(invokeMock).toHaveBeenCalledWith("provider_test_connection", { uuid: "u1" });
  });
});

describe("keystore command wrappers", () => {
  beforeEach(() => invokeMock.mockReset());

  it("keystoreHealth invokes keystore_health", async () => {
    invokeMock.mockResolvedValue("");
    await keystoreHealth();
    expect(invokeMock).toHaveBeenCalledWith("keystore_health");
  });

  it("archiveKeystore invokes archive_keystore", async () => {
    invokeMock.mockResolvedValue("/path/to/archive");
    await archiveKeystore();
    expect(invokeMock).toHaveBeenCalledWith("archive_keystore");
  });

  it("resetKeystore invokes reset_keystore", async () => {
    invokeMock.mockResolvedValue(null);
    await resetKeystore();
    expect(invokeMock).toHaveBeenCalledWith("reset_keystore");
  });

  it("keyStatus invokes key_status", async () => {
    invokeMock.mockResolvedValue({ "provider/u1": true });
    await keyStatus();
    expect(invokeMock).toHaveBeenCalledWith("key_status");
  });
});
