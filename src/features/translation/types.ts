/**
 * Frontend translation state model for R2b. This is the single source of truth
 * consumed by the rebuilt Popup (Surface 01) and InputPanel (Surface 02).
 *
 * The backend emits two Tauri events (`popup-state`, `popup-multi-result`) and
 * one IPC result (`translate_session`). Decoders in decode.ts map each wire
 * shape onto this union. UI code MUST NOT import the wire types directly — it
 * only ever sees `TranslationState`.
 */

/** One engine's outcome, frontend shape (mirrors backend TranslationOutcomeSerialized). */
export type TranslationOutcomeFE = {
  uuid: string;
  ok: boolean;
  text?: string;
  engine?: string;
  error?: string;
};

/** Backend `popup-state` payload. */
export type PopupStatePayload = {
  status: "loading" | "result" | "error";
  text: string;
  engine: string;
  source_text?: string;
};

/** Backend `popup-multi-result` payload. */
export type PopupMultiPayload = {
  outcomes: TranslationOutcomeFE[];
  source_text?: string;
};

/** Backend `translate_session` return value. */
export type SessionResultFE = {
  outcomes: TranslationOutcomeFE[];
  actual_engine?: string;
};

/** Fine-grained error category, derived by classifying the backend error string. */
export type ErrorKind =
  | "network"
  | "config-key"
  | "config-401"
  | "offline"
  | "no-selection"
  | "no-permission"
  | "keystore"
  | "no-provider"
  | "generic";

/** A single result card's data (for multi/partial rendering). */
export type ResultEntry = {
  uuid: string;
  engine: string;
  text?: string;
  errorText?: string;
  ok: boolean;
};

/**
 * Discriminant union for all popup/input translation states. `kind` is the
 * discriminant. The error variants collapse into `kind: "error"` with a `sub`
 * field, EXCEPT offline/no-selection/no-permission/keystore-corrupt which are
 * their own `kind` (they render differently — EmptyState + recovery action).
 */
export type TranslationState =
  | { kind: "loading" }
  | { kind: "single-success"; text: string; engine: string }
  | { kind: "multi-success"; results: ResultEntry[] }
  | { kind: "partial"; results: ResultEntry[] }
  | { kind: "error"; sub: ErrorKind; message: string }
  | { kind: "offline"; message: string }
  | { kind: "no-selection" }
  | { kind: "no-permission" }
  | { kind: "keystore-corrupt"; message: string };

/**
 * i18n copy keys for Surface 01 (selection popup) + Surface 02 (input window).
 * Matches design-system/linguaray/pages/01-selection-popup.md and 02-input-window.md.
 */
export type CopyKey =
  // Surface 01
  | "selection.loading"
  | "selection.error.network"
  | "selection.error.config.key"
  | "selection.error.config.auth"
  | "selection.error.noSelection"
  | "selection.error.noPermission"
  | "selection.error.keystore"
  | "selection.error.keystore.cta"
  | "selection.error.offline"
  | "selection.action.copy"
  | "selection.action.copied"
  | "selection.action.speak"
  | "selection.action.stop"
  | "selection.action.pin"
  | "selection.action.unpin"
  | "selection.action.favorite"
  | "selection.action.favorited"
  | "selection.action.retry"
  | "selection.action.openSettings"
  | "selection.action.recovery"
  | "selection.action.comingTts"
  | "selection.action.comingFavorite"
  | "selection.multi.title"
  // Surface 02
  | "input.title"
  | "input.placeholder"
  | "input.action.translate"
  | "input.action.clear"
  | "input.result.label"
  | "input.error.offline";
