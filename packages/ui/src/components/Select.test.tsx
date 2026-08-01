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

  it("loading disables the trigger and shows spinner", () => {
    const { container } = render(() => (
      <Select label="Model" value={null} options={opts} onChange={() => {}} loading />
    ));
    const trigger = container.querySelector(".lr-select__trigger") as HTMLButtonElement;
    expect(trigger.disabled).toBe(true);
    expect(container.querySelector(".lr-spinner")).toBeTruthy();
  });

  it("errorText shows error message", () => {
    const { getByText } = render(() => (
      <Select label="Model" value={null} options={opts} onChange={() => {}} errorText="Fetch failed" />
    ));
    expect(getByText("Fetch failed")).toBeTruthy();
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
