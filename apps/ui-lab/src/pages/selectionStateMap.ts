import type { TranslationState } from "@app/features/translation/types";
import type { SelectionState } from "../i18n";

/**
 * Map each of the 16 ui-lab SelectionState mock values onto the production
 * TranslationState discriminant. This is the parity contract: if production
 * adds a state the lab does not cover (or vice versa), this map + its test
 * fail to compile/run, surfacing the divergence.
 *
 * `pinned` is not a state kind in production (it is a boolean flag on the
 * controller); the lab's "pinned" surface maps to single-success so the card
 * still renders, and the lab's pin button drives the same flag.
 *
 * The mock payloads (engine names, sample texts) mirror what the lab's
 * SelectionPopup renders today, so the production-state view of a lab mock
 * stays visually consistent.
 */
export function labStateToTranslationState(lab: SelectionState): TranslationState {
  switch (lab) {
    case "initial-hidden":
    case "loading":
      return { kind: "loading" };
    case "success-single":
    case "pinned":
      return {
        kind: "single-success",
        text: "The quick brown fox jumps over the lazy dog.",
        engine: "deepseek",
      };
    case "success-dual":
    case "success-multi":
      return {
        kind: "multi-success",
        results: [
          { uuid: "deepseek", engine: "DeepSeek", text: "The quick brown fox jumps over the lazy dog.", ok: true },
          { uuid: "openai", engine: "OpenAI", text: "A quick brown fox leaps over a lazy dog.", ok: true },
        ],
      };
    case "partial":
      return {
        kind: "partial",
        results: [
          { uuid: "deepseek", engine: "DeepSeek", text: "The quick brown fox jumps over the lazy dog.", ok: true },
          { uuid: "openai", engine: "OpenAI", errorText: "Network error", ok: false },
          { uuid: "google", engine: "Google", text: "The fast brown fox jumps over the lazy dog.", ok: true },
        ],
      };
    case "offline-fallback":
      return {
        kind: "single-success",
        text: "The quick brown fox jumps over the lazy dog.",
        engine: "google · fallback",
      };
    case "offline-error":
      return { kind: "offline", message: "Offline" };
    case "error-network":
      return { kind: "error", sub: "network", message: "Network error" };
    case "error-config-key":
      return { kind: "error", sub: "config-key", message: "API key missing" };
    case "error-config-401":
      return { kind: "error", sub: "config-401", message: "401 Unauthorized" };
    case "error-no-provider":
      return { kind: "error", sub: "no-provider", message: "No provider configured" };
    case "error-no-selection":
      return { kind: "no-selection" };
    case "error-no-permission":
      return { kind: "no-permission" };
    case "keystore-corrupt":
      return { kind: "keystore-corrupt", message: "Keystore unreadable" };
  }
}
