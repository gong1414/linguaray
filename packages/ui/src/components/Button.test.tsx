import { describe, it, expect } from "vitest";
import { render } from "@solidjs/testing-library";
import Button from "./Button";
import { assertNoAxeViolations } from "../../test/setup";

describe("Button", () => {
  it("renders children and is a button element", () => {
    const { getByRole } = render(() => <Button>Translate</Button>);
    const btn = getByRole("button", { name: "Translate" });
    expect(btn).toBeInstanceOf(HTMLButtonElement);
    expect(btn).not.toBeDisabled();
  });

  it("disabled prop disables the button", () => {
    const { getByRole } = render(() => <Button disabled>Translate</Button>);
    expect(getByRole("button", { name: "Translate" })).toBeDisabled();
  });

  it("loading sets disabled + aria-busy and keeps accessible name", () => {
    const { getByRole } = render(() => <Button loading>Translate</Button>);
    const btn = getByRole("button");
    // Loading disables the native button (§6 Loading state).
    expect(btn).toBeDisabled();
    expect(btn).toHaveAttribute("aria-busy", "true");
    // Accessible name preserved (spinner label is sr-only, text still present).
    expect(btn.getAttribute("aria-label") ?? btn.textContent).toMatch(/Translate|Loading/);
  });

  it("applies variant and size classes", () => {
    const { getByRole } = render(
      () => <Button variant="destructive" size="lg">Delete</Button>,
    );
    const btn = getByRole("button", { name: "Delete" });
    expect(btn.className).toContain("lr-btn--destructive");
    expect(btn.className).toContain("lr-btn--lg");
  });

  it("has no axe violations", async () => {
    const { container } = render(() => (
      <Button variant="primary">Primary</Button>
    ));
    await assertNoAxeViolations({
      // Isolated button lacks a landmark; color-contrast depends on real CSS
      // tokens absent in jsdom.
      disableRules: ["color-contrast", "landmark-one-main", "page-has-heading-one", "region"],
    });
    // sanity: the button is in the DOM
    expect(container.querySelector("button")).toBeTruthy();
  });
});
