/**
 * Bridge — the ONLY directory under src/ allowed to import `@tauri-apps/*`
 * (enforced by test/bridge-boundary.test.ts; rule 3 in docs/UI-RULES.md).
 *
 * These modules are raw re-export choke points: business code imports
 * `invoke`/`listen`/`getCurrentWindow`/… from here, so Tauri API access has a
 * single auditable seam. Typed command wrappers and tauri-specta bindings can
 * later replace these internals without touching consumers.
 */
export { invoke } from "@tauri-apps/api/core";
