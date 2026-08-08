import { describe, it, expect } from "vitest";
import { render } from "@solidjs/testing-library";
import InlineError from "./InlineError";
import { assertNoAxeViolations } from "../../test/setup";

describe("InlineError", () => {
  it("renders children", () => {
    const { getByText } = render(() => <InlineError>Something went wrong</InlineError>);
    expect(getByText("Something went wrong")).toBeInTheDocument();
  });

  it("has role=alert", () => {
    const { container } = render(() => <InlineError>Error</InlineError>);
    expect(container.querySelector("[role='alert']")).not.toBeNull();
  });

  it("id is applied", () => {
    const { container } = render(() => <InlineError id="field-err">Err</InlineError>);
    expect(container.querySelector("#field-err")).not.toBeNull();
  });

  it("custom icon", () => {
    const { container } = render(() => (
      <InlineError icon={<span data-testid="ci" />}>Err</InlineError>
    ));
    expect(container.querySelector(".inline-error__icon")).not.toBeNull();
  });

  it("no axe violations", async () => {
    render(() => <InlineError>Error occurred</InlineError>);
    await assertNoAxeViolations();
  });
});
