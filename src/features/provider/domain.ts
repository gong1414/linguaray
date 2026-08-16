/**
 * JS-side endpoint validation (mirrors the frozen S0 constraint). Returns a
 * STABLE ERROR CODE — never a display string; the caller maps it through
 * ./copy.ts i18n.
 */

export type EndpointErrorCode =
  | "endpoint-required"
  | "endpoint-invalid-url"
  | "endpoint-must-https";

export type EndpointValidationResult =
  | { ok: true }
  | { ok: false; code: EndpointErrorCode };

/**
 * Global HTTPS; HTTP only for exact loopback hosts (localhost, 127.0.0.1,
 * [::1]). Rejects localhost.evil.com etc.
 */
export function validateEndpoint(
  endpoint: string,
  opts?: { allowEmpty?: boolean },
): EndpointValidationResult {
  const trimmed = endpoint.trim();
  if (!trimmed) {
    return opts?.allowEmpty ? { ok: true } : { ok: false, code: "endpoint-required" };
  }

  let url: URL;
  try {
    url = new URL(trimmed);
  } catch {
    return { ok: false, code: "endpoint-invalid-url" };
  }

  const protocol = url.protocol;
  const host = url.hostname.toLowerCase();

  if (protocol === "https:") return { ok: true };
  if (protocol === "http:") {
    if (host === "localhost" || host === "127.0.0.1" || host === "[::1]") {
      return { ok: true };
    }
    return { ok: false, code: "endpoint-must-https" };
  }
  return { ok: false, code: "endpoint-must-https" };
}
