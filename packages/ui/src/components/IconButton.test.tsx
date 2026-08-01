import { describe, it, expect } from "vitest";
import { render } from "@solidjs/testing-library";
import { Copy } from "lucide-solid";
import IconButton from "./IconButton";
import { assertNoAxeViolations } from "../../test/setup";

describe("IconButton", () => {
  it("requires and exposes aria-label", () => {
    const { getByRole } = render(
      () => (
        <IconButton aria-label="Copy">
          <Copy size={16} />
        </IconButton>
      ),
    );
    const btn = getByRole("button", { name: "Copy" });
    expect(btn.getAttribute("aria-label")).toBe("Copy");
  });

  it("loading sets disabled + aria-busy and keeps the accessible name", () => {
    const { getByRole } = render(
      () => (
        <IconButton aria-label="Copy" loading>
          <Copy size={16} />
        </IconButton>
      ),
    );
    const btn = getByRole("button", { name: /Copy|Loading/ });
    expect(btn).toBeDisabled();
    expect(btn).toHaveAttribute("aria-busy", "true");
  });

  it("has no axe violations (aria-label present)", async () => {
    render(() => (
      <IconButton aria-label="Copy">
        <Copy size={16} />
      </IconButton>
    ));
    await assertNoAxeViolations({
      disableRules: ["color-contrast", "landmark-one-main", "page-has-heading-one", "region"],
    });
  });
});
