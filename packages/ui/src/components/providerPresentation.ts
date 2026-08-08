import type { ProviderRole, ProviderStatus } from "./providerTypes";
import type { StatusBadgeVariant } from "./StatusBadge";

/** 本地化无关：返回 status code + badge variant，不返回硬编码英文文本。 */
export function providerStatus(
  role: ProviderRole,
  hasKey: boolean,
  enabled: boolean,
): { code: ProviderStatus; variant: StatusBadgeVariant } {
  if (!enabled) return { code: "disabled", variant: "neutral" };
  if (!hasKey) return { code: "key-missing", variant: "warning" };
  if (role.kind === "none") return { code: "available", variant: "neutral" };
  return { code: "active", variant: "success" };
}

/** Key status is independent of enabled/disabled state. */
export function providerKeyStatus(hasKey: boolean): "saved" | "missing" {
  return hasKey ? "saved" : "missing";
}
