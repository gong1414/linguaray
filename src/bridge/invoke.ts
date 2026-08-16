/**
 * Bridge — the ONLY directory under src/ allowed to import `@tauri-apps/*`
 * (enforced by test/bridge-boundary.test.ts; rule 3 in docs/UI-RULES.md).
 *
 * Generated command wrappers are the only command IPC surface. Keeping the
 * export here gives feature IPC modules one stable, boundary-checked import.
 */
export { commands } from "./bindings";
export type * from "./bindings";
