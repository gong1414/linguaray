import { describe, it, expect } from "vitest";
import { render } from "@solidjs/testing-library";
import Select from "./Select";
import { assertNoAxeViolations } from "../../test/setup";

const opts = [
  { value: "gpt-4", label: "GPT-4", disabled: false },
  { value: "gpt-3.5", label: "GPT-3.5", disabled: false },
];

describe("Select", () => {
  it("renders a label associated via Kobante", () => {
    const { getByText, getByRole } = render(() => (
      <Select label="Model" value={null} options={opts} onChange={() => {}} placeholder="Pick…" />
    ));
    expect(getByText("Model")).toBeTruthy();
    expect(getByRole("button")).toBeTruthy();
  });

  it("loading disables the trigger, shows spinner, and sets aria-busy", () => {
    const { container } = render(() => (
      <Select label="Model" value={null} options={opts} onChange={() => {}} loading />
    ));
    const trigger = container.querySelector(".lr-select__trigger") as HTMLElement;
    expect(trigger.getAttribute("aria-busy")).toBe("true");
    expect(trigger.hasAttribute("disabled")).toBe(true);
    expect(container.querySelector(".lr-spinner")).toBeTruthy();
  });

  it("errorText shows error message", () => {
    const { getByText } = render(() => (
      <Select label="Model" value={null} options={opts} onChange={() => {}} errorText="Fetch failed" />
    ));
    expect(getByText("Fetch failed")).toBeTruthy();
  });

  it("label is associated with the trigger via Kobante (aria-labelledby)", () => {
    const { container } = render(() => (
      <Select label="Model" value="gpt-4" options={opts} onChange={() => {}} />
    ));
    const trigger = container.querySelector(".lr-select__trigger") as HTMLElement;
    // Kobante auto-generates aria-labelledby pointing to the label element
    expect(trigger.getAttribute("aria-labelledby")).toBeTruthy();
  });

  it("errorText renders an error element with id", () => {
    const { getByText, container } = render(() => (
      <Select label="Model" value={null} options={opts} onChange={() => {}} errorText="Fetch failed" />
    ));
    const errorEl = getByText("Fetch failed");
    // The error element has an id (Kobante generates it for aria-errormessage)
    expect(errorEl.id).toBeTruthy();
    // The trigger is marked invalid
    const trigger = container.querySelector(".lr-select__trigger") as HTMLElement;
    expect(trigger.getAttribute("aria-invalid") === "true" || trigger.getAttribute("data-invalid") !== null).toBeTruthy();
  });

  it("has no axe violations", async () => {
    render(() => (
      <Select label="Model" value="gpt-4" options={opts} onChange={() => {}} />
    ));
    await assertNoAxeViolations({
      disableRules: ["color-contrast", "landmark-one-main", "page-has-heading-one", "region"],
    });
  });
});
