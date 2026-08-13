import type { ProviderRole, ProviderStatus } from "./providerTypes";
import type { StatusBadgeVariant } from "./StatusBadge";

/** Three-state key status (R11). A keyless provider (`needs_key=false`) is
 *  `"not-required"` — distinct from `"missing"` (needs a key but has none) and
 *  `"saved"` (has a key). Localized independently of the row status badge. */
export type KeyStatus = "saved" | "missing" | "not-required";

/** 本地化无关：返回 status code + badge variant，不返回硬编码英文文本。 */
export function providerStatus(
  role: ProviderRole,
  hasKey: boolean,
  enabled: boolean,
  needsKey: boolean,
): { code: ProviderStatus; variant: StatusBadgeVariant } {
  if (!enabled) return { code: "disabled", variant: "neutral" };
  // R11: a keyless provider (needs_key=false) is available — it must NEVER show
  // "key-missing", even when hasKey is false. The badge stays neutral.
  if (!needsKey) return { code: "available", variant: "neutral" };
  if (!hasKey) return { code: "key-missing", variant: "warning" };
  if (role.kind === "none") return { code: "available", variant: "neutral" };
  return { code: "active", variant: "success" };
}

/** Key status is independent of enabled/disabled state. A keyless provider is
 *  `"not-required"` (R11) — the detail panel uses this to hide the key input and
 *  show "No key required" instead. */
export function providerKeyStatus(hasKey: boolean, needsKey: boolean): KeyStatus {
  if (!needsKey) return "not-required";
  return hasKey ? "saved" : "missing";
}
