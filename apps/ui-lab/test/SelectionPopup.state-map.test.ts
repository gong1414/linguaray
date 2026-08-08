import { describe, it, expect } from "vitest";
import { labStateToTranslationState } from "../src/pages/selectionStateMap";
import type { TranslationState } from "@app/features/translation/types";

describe("labStateToTranslationState", () => {
  it("maps loading → loading", () => {
    expect(labStateToTranslationState("loading").kind).toBe("loading");
  });
  it("maps success-single → single-success", () => {
    const s = labStateToTranslationState("success-single") as Extract<
      TranslationState,
      { kind: "single-success" }
    >;
    expect(s.kind).toBe("single-success");
    expect(typeof s.text).toBe("string");
  });
  it("maps success-multi → multi-success", () => {
    expect(labStateToTranslationState("success-multi").kind).toBe("multi-success");
  });
  it("maps partial → partial", () => {
    expect(labStateToTranslationState("partial").kind).toBe("partial");
  });
  it("maps error-network → error sub=network", () => {
    const s = labStateToTranslationState("error-network") as Extract<
      TranslationState,
      { kind: "error" }
    >;
    expect(s.kind).toBe("error");
    expect(s.sub).toBe("network");
  });
  it("maps error-config-401 → error sub=config-401", () => {
    const s = labStateToTranslationState("error-config-401") as Extract<
      TranslationState,
      { kind: "error" }
    >;
    expect(s.sub).toBe("config-401");
  });
  it("maps offline-error → offline", () => {
    expect(labStateToTranslationState("offline-error").kind).toBe("offline");
  });
  it("maps offline-fallback → single-success (the fallback result)", () => {
    expect(labStateToTranslationState("offline-fallback").kind).toBe("single-success");
  });
  it("maps error-no-selection → no-selection", () => {
    expect(labStateToTranslationState("error-no-selection").kind).toBe("no-selection");
  });
  it("maps error-no-permission → no-permission", () => {
    expect(labStateToTranslationState("error-no-permission").kind).toBe("no-permission");
  });
  it("maps keystore-corrupt → keystore-corrupt", () => {
    expect(labStateToTranslationState("keystore-corrupt").kind).toBe("keystore-corrupt");
  });
  it("maps pinned → single-success (pinned is a pin-flag, not a state kind)", () => {
    expect(labStateToTranslationState("pinned").kind).toBe("single-success");
  });
  it("initial-hidden → loading (lab never renders initial-hidden; production hides the window)", () => {
    expect(labStateToTranslationState("initial-hidden").kind).toBe("loading");
  });
});
