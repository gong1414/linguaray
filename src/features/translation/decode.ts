import type {
  ErrorKind,
  PopupMultiPayload,
  PopupStatePayload,
  ResultEntry,
  SessionResultFE,
  TranslationOutcomeFE,
  TranslationState,
} from "./types";

/**
 * Classify a backend error string into a fine-grained kind.
 *
 * The backend serializes `Error` via its Display impl (popup.rs
 * `error: Some(e.to_string())`), so the frontend classifies by stable
 * substrings. Keyword order matters: more specific kinds are checked first
 * (e.g. "keystore" before generic; "401"/"403" before "key").
 *
 * Substrings chosen match the Rust Error/ConfigKind Display output in
 * src-tauri/src/error.rs (network/timeout/401/403/key/keystore/offline/etc.).
 */
export function classifyError(message: string): ErrorKind {
  const m = message.toLowerCase();
  if (m.includes("no text selected") || m.includes("nothing selected")) {
    return "no-selection";
  }
  if (m.includes("accessibility") || m.includes("permission")) {
    return "no-permission";
  }
  if (m.includes("keystore")) {
    return "keystore";
  }
  if (m.includes("offline") || m.includes("no network")) {
    return "offline";
  }
  if (m.includes("no provider") || m.includes("not configured")) {
    return "no-provider";
  }
  if (
    m.includes("401") ||
    m.includes("403") ||
    m.includes("unauthorized") ||
    m.includes("forbidden")
  ) {
    return "config-401";
  }
  if (m.includes("missing") && m.includes("key")) {
    return "config-key";
  }
  if (m.includes("no api key") || m.includes("api key")) {
    return "config-key";
  }
  if (
    m.includes("network") ||
    m.includes("timeout") ||
    m.includes("timed out") ||
    m.includes("connection") ||
    m.includes("unreachable")
  ) {
    return "network";
  }
  return "generic";
}

/**
 * Map a classified kind onto the right TranslationState variant. Offline,
 * no-selection, no-permission, keystore, and no-provider are their own kinds
 * (distinct render); the rest become { kind: "error", sub }.
 */
function errorToState(message: string): TranslationState {
  const sub = classifyError(message);
  switch (sub) {
    case "offline":
      return { kind: "offline", message };
    case "no-selection":
      return { kind: "no-selection" };
    case "no-permission":
      return { kind: "no-permission" };
    case "keystore":
      return { kind: "keystore-corrupt", message };
    default:
      return { kind: "error", sub, message };
  }
}

/** Decode the legacy single-channel `popup-state` event. */
export function decodePopupState(payload: PopupStatePayload): TranslationState {
  switch (payload.status) {
    case "loading":
      return { kind: "loading" };
    case "result":
      return { kind: "single-success", text: payload.text, engine: payload.engine };
    case "error":
      return errorToState(payload.text);
  }
}

function outcomeToEntry(o: TranslationOutcomeFE): ResultEntry {
  return {
    uuid: o.uuid,
    engine: o.engine ?? o.uuid,
    text: o.text,
    errorText: o.error,
    ok: o.ok,
  };
}

/**
 * Decide multi-success / single-success / partial / error from an outcomes
 * array. Shared by `decodePopupMultiResult` and `decodeSessionResult`.
 *
 * - 0 outcomes    → error (defensive; backend should never send this)
 * - 1 ok outcome  → single-success
 * - all ok (>=2)  → multi-success
 * - mixed         → partial (per-engine errors preserved in ResultEntry)
 * - all failed    → error:
 *                    * exactly 1 outcome → classify that single engine's error
 *                      (e.g. "missing key" → config-key) so the headline
 *                      matches the one engine that ran;
 *                    * >=2 outcomes → sub=generic. With several engines all
 *                      failing for different reasons, no single error string
 *                      represents the whole failure, so we surface a neutral
 *                      "all engines failed" headline (per-engine text is still
 *                      available on each ResultEntry if a UI wants detail).
 */
export function decodeOutcomes(outcomes: TranslationOutcomeFE[]): TranslationState {
  if (outcomes.length === 0) {
    return { kind: "error", sub: "generic", message: "no outcomes" };
  }
  const results = outcomes.map(outcomeToEntry);
  const okCount = results.filter((r) => r.ok).length;
  if (okCount === results.length) {
    if (results.length === 1) {
      const r = results[0];
      return { kind: "single-success", text: r.text ?? "", engine: r.engine };
    }
    return { kind: "multi-success", results };
  }
  if (okCount === 0) {
    if (results.length === 1) {
      const msg = results[0].errorText ?? "all engines failed";
      return errorToState(msg);
    }
    return { kind: "error", sub: "generic", message: "all engines failed" };
  }
  return { kind: "partial", results };
}

/** Decode the `popup-multi-result` event (R2a multi-engine channel). */
export function decodePopupMultiResult(payload: PopupMultiPayload): TranslationState {
  return decodeOutcomes(payload.outcomes);
}

/** Decode the `translate_session` IPC return value (used by the input window). */
export function decodeSessionResult(result: SessionResultFE): TranslationState {
  return decodeOutcomes(result.outcomes);
}
