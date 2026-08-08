import { describe, it, expect, vi } from "vitest";
import { render, fireEvent } from "@solidjs/testing-library";
import ProviderRow from "./ProviderRow";
import { providerStatus } from "./providerPresentation";
import { assertNoAxeViolations } from "../../test/setup";

describe("providerPresentation", () => {
  it("disabled → { code: disabled, variant: neutral }", () => {
    expect(providerStatus({ kind: "none" }, true, false)).toEqual({ code: "disabled", variant: "neutral" });
  });
  it("no key → { code: key-missing, variant: warning }", () => {
    expect(providerStatus({ kind: "primary" }, false, true)).toEqual({ code: "key-missing", variant: "warning" });
  });
  it("primary + key → { code: active, variant: success }", () => {
    expect(providerStatus({ kind: "primary" }, true, true)).toEqual({ code: "active", variant: "success" });
  });
  it("none + key → { code: available, variant: neutral }", () => {
    expect(providerStatus({ kind: "none" }, true, true)).toEqual({ code: "available", variant: "neutral" });
  });
});

describe("ProviderRow", () => {
  const labels = {
    edit: "Edit provider", delete: "Delete provider", enabled: "Enabled",
    statusText: { active: "Active", available: "Available", "key-missing": "Key missing", disabled: "Disabled" },
  };
  const baseProps = {
    name: "OpenAI", template: "openai", hasKey: true, enabled: true,
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

  it("no axe violations", async () => {
    render(() => <ProviderRow {...baseProps} />);
    await assertNoAxeViolations({ disableRules: ["region"] });
  });
});
