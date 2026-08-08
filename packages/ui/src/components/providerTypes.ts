export type ProviderRole =
  | { kind: "none" }
  | { kind: "primary" }
  | { kind: "parallel"; index: number }
  | { kind: "fallback" };

export type ProviderStatus =
  | "active"
  | "available"
  | "key-missing"
  | "disabled";
