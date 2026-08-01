import { describe, it, expect } from "vitest";
import { render } from "@solidjs/testing-library";
import { Server } from "lucide-solid";
import EmptyState from "./EmptyState";
import { assertNoAxeViolations } from "../../test/setup";

describe("EmptyState", () => {
  it("renders icon + title + description", () => {
    const { getByText } = render(() => (
      <EmptyState
        icon={<Server size={32} />}
        title="No providers"
        description="Add your first provider to get started"
      />
    ));
    expect(getByText("No providers")).toBeTruthy();
    expect(getByText("Add your first provider to get started")).toBeTruthy();
  });

  it("has no axe violations", async () => {
    render(() => (
      <EmptyState icon={<Server size={32} />} title="Empty" description="Nothing here" />
    ));
    await assertNoAxeViolations({
      disableRules: ["color-contrast", "landmark-one-main", "page-has-heading-one", "region"],
    });
  });
});
