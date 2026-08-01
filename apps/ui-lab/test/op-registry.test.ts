import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { OpRegistry, type OpKind } from "../src/pages/op-registry";

describe("OpRegistry — CAS semantics", () => {
  let registry: OpRegistry;

  beforeEach(() => {
    vi.useFakeTimers();
    registry = new OpRegistry();
  });
  afterEach(() => vi.useRealTimers());

  it("startOp returns a token and registers the op", () => {
    let cleared = false;
    const token = registry.startOp(
      "save" as OpKind,
      "uuid-a",
      () => (cleared = true),
      () => {},
      1000,
    );
    expect(token).toBeGreaterThan(0);
    expect(registry.isActive("save" as OpKind, "uuid-a")).toBe(true);
    expect(cleared).toBe(false);
  });

  it("startOp cancels a previous op on the same key", () => {
    let oldCleared = false;
    const oldToken = registry.startOp(
      "save" as OpKind,
      "uuid-a",
      () => (oldCleared = true),
      () => {},
      1000,
    );
    const newToken = registry.startOp(
      "save" as OpKind,
      "uuid-a",
      () => {},
      () => {},
      1000,
    );
    expect(newToken).toBeGreaterThan(oldToken);
    // Old op's clearBusy was called
    expect(oldCleared).toBe(true);
    // New token is current
    expect(registry.currentToken("save" as OpKind, "uuid-a")).toBe(newToken);
  });

  it("finishOpIfCurrent with old token returns false and does NOT run result", () => {
    let resultRan = false;
    const oldToken = registry.startOp(
      "test" as OpKind,
      "uuid-a",
      () => {},
      () => {},
      1000,
    );
    // Start a NEW op on the same key (old is cancelled)
    registry.startOp(
      "test" as OpKind,
      "uuid-a",
      () => {},
      () => {},
      1000,
    );
    // Try to finish with the OLD token
    const applied = registry.finishOpIfCurrent("test" as OpKind, "uuid-a", oldToken, () => {
      resultRan = true;
    });
    expect(applied).toBe(false);
    expect(resultRan).toBe(false);
  });

  it("finishOpIfCurrent with current token runs result and cleans up", () => {
    let resultRan = false;
    const token = registry.startOp(
      "fetch" as OpKind,
      "uuid-a",
      () => {},
      () => {},
      1000,
    );
    const applied = registry.finishOpIfCurrent("fetch" as OpKind, "uuid-a", token, () => {
      resultRan = true;
    });
    expect(applied).toBe(true);
    expect(resultRan).toBe(true);
    expect(registry.isActive("fetch" as OpKind, "uuid-a")).toBe(false);
  });

  it("cancelOpIfCurrent with old token does not clear new op's busy", () => {
    let newCleared = false;
    const oldToken = registry.startOp(
      "balance" as OpKind,
      "uuid-a",
      () => {},
      () => {},
      1000,
    );
    registry.startOp(
      "balance" as OpKind,
      "uuid-a",
      () => (newCleared = true),
      () => {},
      1000,
    );
    // Cancel with OLD token — should NOT clear the new op
    const cancelled = registry.cancelOpIfCurrent("balance" as OpKind, "uuid-a", oldToken);
    expect(cancelled).toBe(false);
    expect(newCleared).toBe(false);
    // New op still active
    expect(registry.isActive("balance" as OpKind, "uuid-a")).toBe(true);
  });

  it("cancelOpsForUuid cancels all ops for a provider", () => {
    let saveCleared = false;
    let testCleared = false;
    registry.startOp("save" as OpKind, "uuid-a", () => (saveCleared = true), () => {}, 1000);
    registry.startOp("test" as OpKind, "uuid-a", () => (testCleared = true), () => {}, 1000);
    // Different provider — should NOT be cancelled
    let otherCleared = false;
    registry.startOp("save" as OpKind, "uuid-b", () => (otherCleared = true), () => {}, 1000);

    registry.cancelOpsForUuid("uuid-a");
    expect(saveCleared).toBe(true);
    expect(testCleared).toBe(true);
    expect(otherCleared).toBe(false);
    expect(registry.isActive("save" as OpKind, "uuid-a")).toBe(false);
    expect(registry.isActive("save" as OpKind, "uuid-b")).toBe(true);
  });

  it("timer fires and runs callback only if still current", () => {
    let ran = false;
    registry.startOp(
      "save" as OpKind,
      "uuid-a",
      () => {},
      () => {
        ran = true;
      },
      1000,
    );
    vi.advanceTimersByTime(1100);
    expect(ran).toBe(true);
    expect(registry.isActive("save" as OpKind, "uuid-a")).toBe(false);
  });

  it("timer does NOT fire if a newer op replaced it", () => {
    let oldRan = false;
    registry.startOp(
      "save" as OpKind,
      "uuid-a",
      () => {},
      () => {
        oldRan = true;
      },
      1000,
    );
    // Replace with a new op before timer fires
    registry.startOp("save" as OpKind, "uuid-a", () => {}, () => {}, 2000);
    vi.advanceTimersByTime(1100); // old timer would have fired
    expect(oldRan).toBe(false);
  });

  it("cancelAll clears everything", () => {
    let cleared1 = false;
    let cleared2 = false;
    registry.startOp("save" as OpKind, "uuid-a", () => (cleared1 = true), () => {}, 1000);
    registry.startOp("test" as OpKind, "uuid-b", () => (cleared2 = true), () => {}, 1000);
    registry.cancelAll();
    expect(cleared1).toBe(true);
    expect(cleared2).toBe(true);
  });
});
