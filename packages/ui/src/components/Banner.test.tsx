import { describe, it, expect } from "vitest";
import { render, fireEvent } from "@solidjs/testing-library";
import Banner from "./Banner";
import { assertNoAxeViolations } from "../../test/setup";

describe("Banner", () => {
  it("renders title + description", () => {
    const { getByText } = render(() => (
      <Banner variant="info" title="Heads up" description="Some detail" />
    ));
    expect(getByText("Heads up")).toBeTruthy();
    expect(getByText("Some detail")).toBeTruthy();
  });

  it("destructive has role=alert", () => {
    const { getByRole } = render(() => (
      <Banner variant="destructive" title="Error" />
    ));
    expect(getByRole("alert")).toBeTruthy();
  });

  it("info has role=status", () => {
    const { getByRole } = render(() => (
      <Banner variant="info" title="Info" />
    ));
    expect(getByRole("status")).toBeTruthy();
  });

  it("dismiss calls onDismiss", () => {
    let dismissed = false;
    const { getByRole } = render(() => (
      <Banner variant="info" title="Info" onDismiss={() => (dismissed = true)} />
    ));
    fireEvent.click(getByRole("button", { name: "Dismiss" }));
    expect(dismissed).toBe(true);
  });

  it("has no axe violations", async () => {
    render(() => <Banner variant="warning" title="Warning" description="Careful" />);
    await assertNoAxeViolations({
      disableRules: ["color-contrast", "landmark-one-main", "page-has-heading-one", "region"],
    });
  });
});
