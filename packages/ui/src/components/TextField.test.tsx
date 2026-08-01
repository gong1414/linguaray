import { describe, it, expect } from "vitest";
import { render } from "@solidjs/testing-library";
import TextField from "./TextField";
import { assertNoAxeViolations } from "../../test/setup";

describe("TextField", () => {
  it("associates label with input via for/id", () => {
    const { getByLabelText } = render(
      () => <TextField label="API key" value="sk-test" />,
    );
    const input = getByLabelText("API key");
    expect(input).toBeInstanceOf(HTMLInputElement);
  });

  it("helperText links via aria-describedby when no error", () => {
    const { getByLabelText, getByText } = render(
      () => <TextField label="Key" helperText="Stored encrypted on disk" />,
    );
    const input = getByLabelText("Key") as HTMLInputElement;
    const helper = getByText("Stored encrypted on disk");
    expect(input.getAttribute("aria-describedby")).toBe(helper.id);
    expect(input.getAttribute("aria-invalid")).toBeNull();
  });

  it("errorText sets aria-invalid, points describedby to error, hides helper", () => {
    const { getByLabelText, queryByText, getByText } = render(
      () => (
        <TextField
          label="Key"
          helperText="Stored encrypted on disk"
          errorText="Key is required"
        />
      ),
    );
    const input = getByLabelText("Key") as HTMLInputElement;
    const error = getByText("Key is required");
    expect(input.getAttribute("aria-invalid")).toBe("true");
    expect(input.getAttribute("aria-describedby")).toBe(error.id);
    // helper hidden when error present
    expect(queryByText("Stored encrypted on disk")).toBeNull();
  });

  it("has no axe violations", async () => {
    render(() => <TextField label="Endpoint" helperText="HTTPS only" />);
    await assertNoAxeViolations({
      disableRules: ["color-contrast", "landmark-one-main", "page-has-heading-one", "region"],
    });
  });
});
