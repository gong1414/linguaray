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
      "save" as OpKind, "uuid-a",
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
      "save" as OpKind, "uuid-a",
      () => (oldCleared = true),
      () => {},
      1000,
    );
    const newToken = registry.startOp(
      "save" as OpKind, "uuid-a",
      () => {},
      () => {},
      1000,
    );
    expect(newToken).toBeGreaterThan(oldToken);
    expect(oldCleared).toBe(true);
    expect(registry.currentToken("save" as OpKind, "uuid-a")).toBe(newToken);
  });

  it("finishOpIfCurrent with old token returns false and does NOT run result", () => {
    let resultRan = false;
    const oldToken = registry.startOp(
      "test" as OpKind, "uuid-a",
      () => {}, () => {}, 1000,
    );
    registry.startOp("test" as OpKind, "uuid-a", () => {}, () => {}, 1000);
    const applied = registry.finishOpIfCurrent("test" as OpKind, "uuid-a", oldToken, () => {
      resultRan = true;
    });
    expect(applied).toBe(false);
    expect(resultRan).toBe(false);
  });

  it("finishOpIfCurrent with current token runs result and cleans up", () => {
    let resultRan = false;
    const token = registry.startOp(
      "fetch" as OpKind, "uuid-a",
      () => {}, () => {}, 1000,
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
    const oldToken = registry.startOp("balance" as OpKind, "uuid-a", () => {}, () => {}, 1000);
    registry.startOp("balance" as OpKind, "uuid-a", () => (newCleared = true), () => {}, 1000);
    const cancelled = registry.cancelOpIfCurrent("balance" as OpKind, "uuid-a", oldToken);
    expect(cancelled).toBe(false);
    expect(newCleared).toBe(false);
    expect(registry.isActive("balance" as OpKind, "uuid-a")).toBe(true);
  });

  it("cancelOpsForUuid cancels all ops for a provider but not others", () => {
    let s = false, t = false, other = false;
    registry.startOp("save" as OpKind, "uuid-a", () => (s = true), () => {}, 1000);
    registry.startOp("test" as OpKind, "uuid-a", () => (t = true), () => {}, 1000);
    registry.startOp("save" as OpKind, "uuid-b", () => (other = true), () => {}, 1000);
    registry.cancelOpsForUuid("uuid-a");
    expect(s).toBe(true);
    expect(t).toBe(true);
    expect(other).toBe(false);
  });

  it("timer fires and runs result exactly once (CAS auto-complete)", () => {
    let resultCount = 0;
    registry.startOp(
      "save" as OpKind, "uuid-a",
      () => {},
      () => { resultCount++; },
      1000,
    );
    expect(resultCount).toBe(0);
    vi.advanceTimersByTime(1100);
    expect(resultCount).toBe(1);
    expect(registry.isActive("save" as OpKind, "uuid-a")).toBe(false);
  });

  it("timer does NOT fire result if a newer op replaced it", () => {
    let oldResult = false;
    registry.startOp("save" as OpKind, "uuid-a", () => {}, () => { oldResult = true; }, 1000);
    registry.startOp("save" as OpKind, "uuid-a", () => {}, () => {}, 2000);
    vi.advanceTimersByTime(1100);
    expect(oldResult).toBe(false);
  });

  it("cancelAll: snapshot + clear BEFORE clearBusy (reentry-safe)", () => {
    let cleared1 = false, cleared2 = false;
    registry.startOp("save" as OpKind, "uuid-a", () => (cleared1 = true), () => {}, 1000);
    registry.startOp("test" as OpKind, "uuid-b", () => (cleared2 = true), () => {}, 1000);
    registry.cancelAll();
    expect(cleared1).toBe(true);
    expect(cleared2).toBe(true);
    expect(registry.isActive("save" as OpKind, "uuid-a")).toBe(false);
  });
});
