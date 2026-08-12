import { describe, it, expect, vi } from "vitest";
import { render, fireEvent } from "@solidjs/testing-library";
import ProviderRow from "./ProviderRow";
import { providerStatus } from "./providerPresentation";
import { assertNoAxeViolations } from "../../test/setup";

describe("providerPresentation", () => {
  it("disabled → { code: disabled, variant: neutral }", () => {
    expect(providerStatus({ kind: "none" }, true, false, true)).toEqual({ code: "disabled", variant: "neutral" });
  });
  it("needs_key + no key → { code: key-missing, variant: warning }", () => {
    expect(providerStatus({ kind: "primary" }, false, true, true)).toEqual({ code: "key-missing", variant: "warning" });
  });
  it("primary + key → { code: active, variant: success }", () => {
    expect(providerStatus({ kind: "primary" }, true, true, true)).toEqual({ code: "active", variant: "success" });
  });
  it("none + key → { code: available, variant: neutral }", () => {
    expect(providerStatus({ kind: "none" }, true, true, true)).toEqual({ code: "available", variant: "neutral" });
  });
  // ─── R11: needs_key=false three-state modelling ─────────────────────────
  it("R11: needs_key=false + enabled + no key → available/neutral (NOT key-missing)", () => {
    // A keyless provider (e.g. local Ollama) must never show "key-missing".
    expect(providerStatus({ kind: "primary" }, false, true, false)).toEqual({ code: "available", variant: "neutral" });
  });
  it("R11: needs_key=false + enabled + key present → available/neutral", () => {
    // Key present is irrelevant for a keyless provider — still available/neutral.
    expect(providerStatus({ kind: "none" }, true, true, false)).toEqual({ code: "available", variant: "neutral" });
  });
  it("R11: needs_key=false but disabled → disabled/neutral (disabled wins)", () => {
    expect(providerStatus({ kind: "none" }, false, false, false)).toEqual({ code: "disabled", variant: "neutral" });
  });
});

describe("ProviderRow", () => {
  const labels = {
    edit: "Edit provider", delete: "Delete provider", enabled: "Enabled",
    statusText: { active: "Active", available: "Available", "key-missing": "Key missing", disabled: "Disabled" },
  };
  const baseProps = {
    name: "OpenAI", template: "openai", hasKey: true, needsKey: true, enabled: true,
    role: { kind: "primary" } as const,
    labels,
    onToggle: () => {}, onEdit: () => {}, onDelete: () => {},
  };

  it("renders name and template", () => {
    const { getByText } = render(() => <ProviderRow {...baseProps} />);
    expect(getByText("OpenAI")).toBeInTheDocument();
    expect(getByText("openai")).toBeInTheDocument();
  });

  it("onToggle fires with enabled boolean", () => {
    const onToggle = vi.fn();
    const { getByRole } = render(() => <ProviderRow {...baseProps} onToggle={onToggle} />);
    fireEvent.click(getByRole("switch"));
    expect(onToggle).toHaveBeenCalledOnce();
    expect(onToggle).toHaveBeenCalledWith(false);
  });

  it("onEdit fires", () => {
    const onEdit = vi.fn();
    const { getByLabelText } = render(() => <ProviderRow {...baseProps} onEdit={onEdit} />);
    fireEvent.click(getByLabelText("Edit provider"));
    expect(onEdit).toHaveBeenCalledOnce();
  });

  it("onDelete fires", () => {
    const onDelete = vi.fn();
    const { getByLabelText } = render(() => <ProviderRow {...baseProps} onDelete={onDelete} />);
    fireEvent.click(getByLabelText("Delete provider"));
    expect(onDelete).toHaveBeenCalledOnce();
  });

  it("active applies active accent class on the row", () => {
    const { container } = render(() => <ProviderRow {...baseProps} active />);
    expect(container.querySelector(".provider-row--active")).not.toBeNull();
  });

  it("inactive does not apply active accent class", () => {
    const { container } = render(() => <ProviderRow {...baseProps} />);
    expect(container.querySelector(".provider-row--active")).toBeNull();
  });

  // ─── R11: ProviderRow three-state (test 8) ──────────────────────────────
  it("R11: needs_key=false + enabled + no key → neutral/Available badge, NOT warning/Key missing", () => {
    // A keyless provider must render the neutral "Available" status, never the
    // warning "Key missing" badge. This is the cross-cutting P1 fix: before R11
    // a needs_key=false provider with hasKey=false wrongly showed "Key missing".
    const { getByText, queryByText } = render(() => (
      <ProviderRow
        {...baseProps}
        hasKey={false}
        needsKey={false}
        role={{ kind: "none" }}
      />
    ));
    expect(getByText("Available")).toBeInTheDocument();
    expect(queryByText("Key missing")).toBeNull();
  });

  it("R11: needs_key=true + no key → warning/Key missing badge (unchanged)", () => {
    const { getByText } = render(() => (
      <ProviderRow {...baseProps} hasKey={false} needsKey={true} role={{ kind: "primary" }} />
    ));
    expect(getByText("Key missing")).toBeInTheDocument();
  });

  it("no axe violations", async () => {
    render(() => <ProviderRow {...baseProps} />);
    await assertNoAxeViolations({ disableRules: ["region"] });
  });
});
