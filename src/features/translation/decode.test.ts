import { describe, it, expect } from "vitest";
import { decodePopupState, decodePopupMultiResult, decodeSessionResult, classifyError } from "./decode";

describe("decodePopupState", () => {
  it("loading → kind=loading", () => {
    const s = decodePopupState({ status: "loading", text: "", engine: "" });
    expect(s.kind).toBe("loading");
  });

  it("result → single-success with text + engine", () => {
    const s = decodePopupState({ status: "result", text: "你好", engine: "deepseek/u1" });
    expect(s).toEqual({ kind: "single-success", text: "你好", engine: "deepseek/u1" });
  });

  it("error with network message → kind=error, sub=network", () => {
    const s = decodePopupState({ status: "error", text: "network error: timeout", engine: "" });
    expect(s.kind).toBe("error");
    expect(s).toHaveProperty("sub", "network");
  });

  it("error with offline message → kind=offline", () => {
    const s = decodePopupState({ status: "error", text: "offline: no network", engine: "" });
    expect(s.kind).toBe("offline");
  });

  it("error with keystore message → kind=keystore-corrupt", () => {
    const s = decodePopupState({ status: "error", text: "keystore unreadable", engine: "" });
    expect(s.kind).toBe("keystore-corrupt");
  });

  it("error with no-selection message → kind=no-selection", () => {
    const s = decodePopupState({ status: "error", text: "no text selected", engine: "" });
    expect(s.kind).toBe("no-selection");
  });

  it("error with permission message → kind=no-permission", () => {
    const s = decodePopupState({ status: "error", text: "accessibility permission denied", engine: "" });
    expect(s.kind).toBe("no-permission");
  });

  it("error with no-provider message → kind=no-provider", () => {
    const s = decodePopupState({ status: "error", text: "no active provider selected", engine: "" });
    expect(s.kind).toBe("no-provider");
  });

  it("error with 401 message → kind=error, sub=config-401", () => {
    const s = decodePopupState({ status: "error", text: "401 Unauthorized", engine: "" });
    expect(s.kind).toBe("error");
    expect(s).toHaveProperty("sub", "config-401");
  });

  it("error with missing-key message → kind=error, sub=config-key", () => {
    const s = decodePopupState({ status: "error", text: "missing API key for deepseek", engine: "" });
    expect(s.kind).toBe("error");
    expect(s).toHaveProperty("sub", "config-key");
  });

  it("unknown error text → kind=error, sub=generic", () => {
    const s = decodePopupState({ status: "error", text: "something exploded", engine: "" });
    expect(s.kind).toBe("error");
    expect(s).toHaveProperty("sub", "generic");
  });
});

describe("decodePopupMultiResult", () => {
  it("all-ok outcomes → multi-success", () => {
    const s = decodePopupMultiResult({
      outcomes: [
        { uuid: "u1", ok: true, text: "a", engine: "deepseek/u1" },
        { uuid: "u2", ok: true, text: "b", engine: "openai/u2" },
      ],
    });
    expect(s.kind).toBe("multi-success");
    expect(s.kind === "multi-success" && s.results.length).toBe(2);
  });

  it("single ok outcome → single-success", () => {
    const s = decodePopupMultiResult({
      outcomes: [{ uuid: "u1", ok: true, text: "a", engine: "deepseek/u1" }],
    });
    expect(s.kind).toBe("single-success");
  });

  it("mixed ok/failed → partial", () => {
    const s = decodePopupMultiResult({
      outcomes: [
        { uuid: "u1", ok: true, text: "a", engine: "deepseek/u1" },
        { uuid: "u2", ok: false, error: "timeout" },
      ],
    });
    expect(s.kind).toBe("partial");
  });

  it("all-failed → kind=error", () => {
    const s = decodePopupMultiResult({
      outcomes: [
        { uuid: "u1", ok: false, error: "timeout" },
        { uuid: "u2", ok: false, error: "401" },
      ],
    });
    expect(s.kind).toBe("error");
    expect(s).toHaveProperty("sub", "generic");
  });

  it("empty outcomes → kind=error (defensive)", () => {
    const s = decodePopupMultiResult({ outcomes: [] });
    expect(s.kind).toBe("error");
  });
});

describe("decodeSessionResult", () => {
  it("single-engine success → single-success", () => {
    const s = decodeSessionResult({
      outcomes: [{ uuid: "u1", ok: true, text: "hi", engine: "deepseek/u1" }],
      actual_engine: "deepseek/u1",
    });
    expect(s.kind).toBe("single-success");
  });

  it("single outcome failed → error", () => {
    const s = decodeSessionResult({
      outcomes: [{ uuid: "u1", ok: false, error: "missing key" }],
      actual_engine: undefined,
    });
    expect(s.kind).toBe("error");
    expect(s).toHaveProperty("sub", "config-key");
  });

  it("multiple ok outcomes → multi-success", () => {
    const ok = decodeSessionResult({
      outcomes: [
        { uuid: "u1", ok: true, text: "a", engine: "deepseek/u1" },
        { uuid: "u2", ok: true, text: "b", engine: "openai/u2" },
      ],
    });
    expect(ok.kind).toBe("multi-success");
  });
});

describe("classifyError", () => {
  it("matches network keywords", () => {
    expect(classifyError("network error: timeout")).toBe("network");
    expect(classifyError("request timed out")).toBe("network");
  });
  it("matches 401/403 as config-401", () => {
    expect(classifyError("401 Unauthorized")).toBe("config-401");
    expect(classifyError("403 Forbidden")).toBe("config-401");
  });
  it("matches missing-key phrasing as config-key", () => {
    expect(classifyError("missing API key for deepseek")).toBe("config-key");
    expect(classifyError("no API key configured")).toBe("config-key");
  });
  it("matches offline", () => {
    expect(classifyError("offline: no network")).toBe("offline");
  });
  it("matches keystore", () => {
    expect(classifyError("keystore unreadable")).toBe("keystore");
  });
  it("matches no-selection", () => {
    expect(classifyError("no text selected")).toBe("no-selection");
  });
  it("matches permission", () => {
    expect(classifyError("accessibility permission denied")).toBe("no-permission");
  });
  it("matches no-provider", () => {
    expect(classifyError("no active provider selected")).toBe("no-provider");
  });
  it("falls back to generic", () => {
    expect(classifyError("something weird")).toBe("generic");
  });
});
