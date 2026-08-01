/**
 * CAS (compare-and-swap) async operation registry.
 *
 * Each operation is keyed by `${kind}:${uuid}` — so a save on provider A
 * cannot interfere with a save on provider B, and a new op on the same
 * key+uuid cancels the old one.
 *
 * Hard constraints (from review):
 * 1. Registry entry is ALWAYS deleted from the map BEFORE the callback runs.
 * 2. cancelAll snapshots + clears the map BEFORE iterating clearBusy, so a
 *    reentrant clearBusy that starts a new op is not wiped by the loop.
 * 3. The timer itself calls finishOpIfCurrent — callers do NOT nest a second
 *    finish. This prevents the "entry already deleted → result never applies"
 *    bug.
 */

export type OpKind = "save" | "profile-save" | "test" | "fetch" | "balance";

export type OpKey = `${OpKind}:${string}`;

export type OpEntry = {
  token: number;
  timerId: number;
  clearBusy: () => void;
  result: () => void;
};

let nextToken = 0;

export class OpRegistry {
  private ops = new Map<OpKey, OpEntry>();

  private static key(kind: OpKind, uuid: string): OpKey {
    return `${kind}:${uuid}` as OpKey;
  }

  /**
   * Start a new operation. Cancels any existing op on the same key first
   * (delete-from-map → clearTimeout → clearBusy), then registers the new one.
   * Returns the new token.
   *
   * The timer calls finishOpIfCurrent internally — callers pass `result`
   * and do NOT call finish themselves.
   */
  startOp(
    kind: OpKind,
    uuid: string,
    clearBusy: () => void,
    result: () => void,
    ms: number,
  ): number {
    const key = OpRegistry.key(kind, uuid);
    this.cancelOp(key);
    const token = ++nextToken;
    const timerId = window.setTimeout(() => {
      // finishOpIfCurrent does the CAS check + delete-from-map + result().
      this.finishOpIfCurrent(kind, uuid, token, result);
    }, ms);
    this.ops.set(key, { token, timerId, clearBusy, result });
    return token;
  }

  /**
   * CAS completion: only applies the result if the stored token still matches.
   * Deletes from map first, then runs result. Returns false if stale.
   */
  finishOpIfCurrent(
    kind: OpKind,
    uuid: string,
    token: number,
    result: () => void,
  ): boolean {
    const key = OpRegistry.key(kind, uuid);
    const entry = this.ops.get(key);
    if (entry?.token !== token) return false;
    // Delete BEFORE callback (hard constraint: reentry-safe).
    this.ops.delete(key);
    result();
    return true;
  }

  /**
   * CAS cancellation: only clears if token matches. Never touches a newer op.
   * Delete from map first, then clearTimeout + clearBusy.
   */
  cancelOpIfCurrent(kind: OpKind, uuid: string, token: number): boolean {
    const key = OpRegistry.key(kind, uuid);
    const entry = this.ops.get(key);
    if (entry?.token !== token) return false;
    this.ops.delete(key);
    window.clearTimeout(entry!.timerId);
    entry!.clearBusy();
    return true;
  }

  /**
   * Cancel all ops for a specific UUID (used on provider switch).
   * Delete from map first, then clearTimeout + clearBusy for each.
   */
  cancelOpsForUuid(uuid: string): void {
    const toCancel: OpKey[] = [];
    for (const [key] of this.ops) {
      if (key.endsWith(`:${uuid}`)) toCancel.push(key);
    }
    for (const key of toCancel) {
      const entry = this.ops.get(key);
      if (!entry) continue;
      this.ops.delete(key);
      window.clearTimeout(entry.timerId);
      entry.clearBusy();
    }
  }

  /**
   * Cancel ALL operations (used on state-change reset).
   * Snapshot + clear the map BEFORE iterating, so a reentrant clearBusy that
   * starts a new op is not deleted by the loop.
   */
  cancelAll(): void {
    const snapshot = [...this.ops.values()];
    this.ops.clear();
    for (const entry of snapshot) {
      window.clearTimeout(entry.timerId);
      entry.clearBusy();
    }
  }

  /** Check if an op is currently active for a key (for testing). */
  isActive(kind: OpKind, uuid: string): boolean {
    return this.ops.has(OpRegistry.key(kind, uuid));
  }

  /** Get the current token for an op (for testing CAS). */
  currentToken(kind: OpKind, uuid: string): number | undefined {
    return this.ops.get(OpRegistry.key(kind, uuid))?.token;
  }

  private cancelOp(key: OpKey): void {
    const entry = this.ops.get(key);
    if (!entry) return;
    this.ops.delete(key);
    window.clearTimeout(entry.timerId);
    entry.clearBusy();
  }
}
