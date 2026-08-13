import { describe, expect, it, vi } from "vitest";
import { fireEvent, render } from "@solidjs/testing-library";
import ListRow from "./ListRow";
import { assertNoAxeViolations } from "../../test/setup";

describe("ListRow", () => {
  it("renders a 36px single-line row and a two-line row marker", () => {
    const { container, getByText } = render(() => (
      <>
        <ListRow title="Translate selection" />
        <ListRow title="Translate input" subtitle="Ctrl+Space" />
      </>
    ));
    expect(getByText("Translate selection")).toBeInTheDocument();
    expect(container.querySelectorAll(".list-row--single")).toHaveLength(1);
    expect(container.querySelectorAll(".list-row--two-line")).toHaveLength(1);
  });

  it("renders the whole row as a button when clickable and there is no trailing action", () => {
    const onClick = vi.fn();
    const { getByRole } = render(() => (
      <ListRow title="Open" onClick={onClick} ariaLabel="Open shortcut" />
    ));
    fireEvent.click(getByRole("button", { name: "Open shortcut" }));
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it("keeps a trailing action outside the primary button", () => {
    const onClick = vi.fn();
    const { container, getByRole } = render(() => (
      <ListRow
        title="Shortcut"
        onClick={onClick}
        ariaLabel="Edit shortcut"
        trailing={<button type="button">Change</button>}
      />
    ));
    expect(container.querySelector("button button")).toBeNull();
    expect(getByRole("button", { name: "Edit shortcut" })).toBeInTheDocument();
    expect(getByRole("button", { name: "Change" })).toBeInTheDocument();
  });

  it("has no axe violations with leading, subtitle, and trailing content", async () => {
    render(() => (
      <main>
        <ListRow
          leading={<span aria-hidden="true">K</span>}
          title="Translate selection"
          subtitle="Global shortcut"
          trailing={<button type="button">Change</button>}
        />
      </main>
    ));
    await assertNoAxeViolations();
  });
});
