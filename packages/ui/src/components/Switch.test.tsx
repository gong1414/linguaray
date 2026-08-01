import { describe, it, expect } from "vitest";
import { render, fireEvent } from "@solidjs/testing-library";
import Switch from "./Switch";
import { assertNoAxeViolations } from "../../test/setup";

describe("Switch", () => {
  it("renders a label", () => {
    const { getByText } = render(() => (
      <Switch checked={false} onChange={() => {}} label="Enable" />
    ));
    expect(getByText("Enable")).toBeTruthy();
  });

  it("toggles on click", () => {
    let val = false;
    const { container } = render(() => (
      <Switch checked={val} onChange={(v) => (val = v)} label="Enable" />
    ));
    const input = container.querySelector("input[type=checkbox]") as HTMLInputElement;
    fireEvent.click(input);
    expect(val).toBe(true);
  });

  it("has no axe violations", async () => {
    render(() => <Switch checked={true} onChange={() => {}} label="Enable" />);
    await assertNoAxeViolations({
      disableRules: ["color-contrast", "landmark-one-main", "page-has-heading-one", "region"],
    });
  });
});
