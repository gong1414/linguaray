import { describe, it, expect } from "vitest";
import { render } from "@solidjs/testing-library";
import ProviderCard from "./ProviderCard";
import { assertNoAxeViolations } from "../../test/setup";

const profile = { name: "OpenAI", template: "openai", status: "active" as const };

describe("ProviderCard", () => {
  it("renders name + template + key-saved badge", () => {
    const { getByText } = render(() => (
      <ProviderCard
        profile={profile}
        hasKey={true}
        needsKey={true}
        role={{ kind: "none" }}
        enabled={true}
        onToggle={() => {}}
        onEdit={() => {}}
        onDelete={() => {}}
      />
    ));
    expect(getByText("OpenAI")).toBeTruthy();
    expect(getByText("openai")).toBeTruthy();
    expect(getByText("Key saved")).toBeTruthy();
  });

  it("shows key-missing when hasKey is false", () => {
    const { getByText } = render(() => (
      <ProviderCard
        profile={profile}
        hasKey={false}
        needsKey={true}
        role={{ kind: "none" }}
        enabled={true}
        onToggle={() => {}}
        onEdit={() => {}}
        onDelete={() => {}}
      />
    ));
    expect(getByText("Key missing")).toBeTruthy();
  });

  it("primary role shows border-left + Primary badge", () => {
    const { container, getByText } = render(() => (
      <ProviderCard
        profile={profile}
        hasKey={true}
        needsKey={true}
        role={{ kind: "primary" }}
        enabled={true}
        onToggle={() => {}}
        onEdit={() => {}}
        onDelete={() => {}}
      />
    ));
    const card = container.querySelector(".lr-provider-card") as HTMLElement;
    expect(card.className).toContain("lr-provider-card--primary");
    expect(getByText("Primary")).toBeTruthy();
  });

  it("parallel role shows index badge", () => {
    const { getByText } = render(() => (
      <ProviderCard
        profile={profile}
        hasKey={true}
        needsKey={true}
        role={{ kind: "parallel", index: 2 }}
        enabled={true}
        onToggle={() => {}}
        onEdit={() => {}}
        onDelete={() => {}}
      />
    ));
    expect(getByText("Parallel #2")).toBeTruthy();
  });

  it("card is a div, not a button (no nested interactive in a button)", () => {
    const { container } = render(() => (
      <ProviderCard
        profile={profile}
        hasKey={true}
        needsKey={true}
        role={{ kind: "none" }}
        enabled={true}
        onToggle={() => {}}
        onEdit={() => {}}
        onDelete={() => {}}
      />
    ));
    const card = container.querySelector(".lr-provider-card") as HTMLElement;
    expect(card.tagName).toBe("DIV");
    // The interactive elements (switch input, edit btn, delete btn) are
    // siblings inside the div, NOT nested inside a button.
    expect(card.querySelector("button button")).toBeNull();
  });

  it("has no axe violations", async () => {
    render(() => (
      <ProviderCard
        profile={profile}
        hasKey={true}
        needsKey={true}
        role={{ kind: "primary" }}
        enabled={true}
        onToggle={() => {}}
        onEdit={() => {}}
        onDelete={() => {}}
      />
    ));
    await assertNoAxeViolations({
      disableRules: ["color-contrast", "landmark-one-main", "page-has-heading-one", "region"],
    });
  });
});
