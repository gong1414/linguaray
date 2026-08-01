import { describe, it, expect } from "vitest";
import { render, fireEvent, screen } from "@solidjs/testing-library";
import { createSignal } from "solid-js";
import Dialog from "./Dialog";
import Confirm from "./Confirm";
import { assertNoAxeViolations } from "../../test/setup";

describe("Dialog", () => {
  it("renders title and description when open", () => {
    render(() => (
      <Dialog open={true} onOpenChange={() => {}} title="My Dialog" description="A desc" />
    ));
    // Portal renders to document.body — use screen (document-wide).
    expect(screen.getByText("My Dialog")).toBeTruthy();
    expect(screen.getByText("A desc")).toBeTruthy();
  });

  it("is not rendered when closed", () => {
    render(() => (
      <Dialog open={false} onOpenChange={() => {}} title="My Dialog" />
    ));
    expect(screen.queryByText("My Dialog")).toBeNull();
  });

  it("has no axe violations when open", async () => {
    render(() => (
      <Dialog open={true} onOpenChange={() => {}} title="Title" description="Desc">
        <p>Body</p>
      </Dialog>
    ));
    await assertNoAxeViolations({
      disableRules: ["color-contrast", "landmark-one-main", "page-has-heading-one", "region"],
    });
  });
});

describe("Confirm", () => {
  it("renders confirm and cancel buttons", () => {
    render(() => (
      <Confirm
        open={true}
        onOpenChange={() => {}}
        title="Delete?"
        message="Sure?"
        confirmLabel="Delete"
        cancelLabel="Cancel"
        onConfirm={() => {}}
        onCancel={() => {}}
      />
    ));
    expect(screen.getByRole("button", { name: "Delete" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Cancel" })).toBeTruthy();
  });

  it("destructive Confirm calls onConfirm and closes", () => {
    const [open, setOpen] = createSignal(true);
    let confirmed = false;
    render(() => (
      <Confirm
        open={open()}
        onOpenChange={setOpen}
        title="Delete?"
        message="Sure?"
        confirmLabel="Delete"
        cancelLabel="Cancel"
        variant="destructive"
        onConfirm={() => (confirmed = true)}
        onCancel={() => {}}
      />
    ));
    fireEvent.click(screen.getByRole("button", { name: "Delete" }));
    expect(confirmed).toBe(true);
    expect(open()).toBe(false);
  });
});
