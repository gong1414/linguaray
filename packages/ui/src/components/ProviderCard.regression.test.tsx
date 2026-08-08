import { describe, it, expect } from "vitest";
import { render } from "@solidjs/testing-library";
import ProviderCard, { defaultProviderCardLabels } from "./ProviderCard";
import type { ProviderRole } from "./providerTypes";

describe("ProviderCard role/key badge regression", () => {
  const profile = { name: "OpenAI", template: "openai", status: "active" as const };
  const noop = () => {};

  it("primary role renders primary role badge", () => {
    const role: ProviderRole = { kind: "primary" };
    const { container } = render(() => (
      <ProviderCard profile={profile} role={role} hasKey={true} enabled={true}
        onEdit={noop} onDelete={noop} onToggle={noop} />
    ));
    const badge = container.querySelector(".lr-provider-card__role-badge--primary");
    expect(badge).not.toBeNull();
    expect(badge!.textContent).toContain(defaultProviderCardLabels.primary);
  });

  it("parallel role renders parallel role badge with index", () => {
    const role: ProviderRole = { kind: "parallel", index: 2 };
    const { container } = render(() => (
      <ProviderCard profile={profile} role={role} hasKey={true} enabled={true}
        onEdit={noop} onDelete={noop} onToggle={noop} />
    ));
    const badge = container.querySelector(".lr-provider-card__role-badge--parallel");
    expect(badge).not.toBeNull();
    expect(badge!.textContent).toContain("#2");
  });

  it("key saved status renders when hasKey=true", () => {
    const role: ProviderRole = { kind: "none" };
    const { container } = render(() => (
      <ProviderCard profile={profile} role={role} hasKey={true} enabled={true}
        onEdit={noop} onDelete={noop} onToggle={noop} />
    ));
    const keyStatus = container.querySelector(".lr-provider-card__key-status--saved");
    expect(keyStatus).not.toBeNull();
  });

  it("key missing status renders when hasKey=false", () => {
    const role: ProviderRole = { kind: "none" };
    const { container } = render(() => (
      <ProviderCard profile={profile} role={role} hasKey={false} enabled={true}
        onEdit={noop} onDelete={noop} onToggle={noop} />
    ));
    const keyStatus = container.querySelector(".lr-provider-card__key-status--missing");
    expect(keyStatus).not.toBeNull();
  });

  it("disabled + missing key shows key-missing (not key-saved)", () => {
    const role: ProviderRole = { kind: "none" };
    const { container } = render(() => (
      <ProviderCard profile={profile} role={role} hasKey={false} enabled={false}
        onEdit={noop} onDelete={noop} onToggle={noop} />
    ));
    const missing = container.querySelector(".lr-provider-card__key-status--missing");
    const saved = container.querySelector(".lr-provider-card__key-status--saved");
    expect(missing, "must show key-missing even when disabled").not.toBeNull();
    expect(saved, "must NOT show key-saved when hasKey=false").toBeNull();
  });
});
