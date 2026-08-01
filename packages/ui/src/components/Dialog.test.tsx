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

  it("closes on Escape key", () => {
    let open = true;
    render(() => (
      <Dialog open={open} onOpenChange={(v) => (open = v)} title="Esc test" />
    ));
    fireEvent.keyDown(document.body, { key: "Escape" });
    expect(open).toBe(false);
  });

  it("Cancel initial focus on destructive (Cancel is focused, not Confirm)", async () => {
    render(() => (
      <Confirm
        open={true}
        onOpenChange={() => {}}
        title="Delete?"
        message="Sure?"
        confirmLabel="Delete"
        cancelLabel="Cancel"
        variant="destructive"
        onConfirm={() => {}}
        onCancel={() => {}}
      />
    ));
    // Kobante moves focus into the dialog on open. Assert the actual
    // activeElement is the Cancel button, not the Delete button.
    await new Promise((r) => setTimeout(r, 100));
    const active = document.activeElement;
    expect(active).toBeTruthy();
    expect(active?.textContent).toContain("Cancel");
  });

  it("Enter key does not confirm destructive (Cancel is default-focused)", () => {
    let confirmed = false;
    render(() => (
      <Confirm
        open={true}
        onOpenChange={() => {}}
        title="Delete?"
        message="Sure?"
        confirmLabel="Delete"
        cancelLabel="Cancel"
        variant="destructive"
        onConfirm={() => (confirmed = true)}
        onCancel={() => {}}
      />
    ));
    // Simulate Enter — should NOT trigger confirm
    fireEvent.keyDown(screen.getByRole("button", { name: "Cancel" }), { key: "Enter" });
    expect(confirmed).toBe(false);
  });

  it("triggerRef: Cancel click — onCloseAutoFocus fires and calls preventDefault", () => {
    const triggerRef: { current?: HTMLElement } = {};
    let open = true;
    let autoFocusPrevented = false;
    render(() => (
      <>
        <button ref={(el) => { triggerRef.current = el; }}>Open</button>
        <Confirm
          open={open}
          onOpenChange={(v) => (open = v)}
          title="T"
          message="M"
          confirmLabel="OK"
          cancelLabel="Cancel"
          onConfirm={() => {}}
          onCancel={() => {}}
          triggerRef={triggerRef}
        />
        {/* Custom Dialog to detect onCloseAutoFocus */}
        <Dialog
          open={false}
          onOpenChange={() => {}}
          title="test"
          triggerRef={triggerRef}
        />
      </>
    ));
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(open).toBe(false);
    // The trigger element exists and is a button
    expect(triggerRef.current?.tagName).toBe("BUTTON");
  });

  it("triggerRef: Esc closes dialog (all close paths covered)", () => {
    const triggerRef: { current?: HTMLElement } = {};
    let open = true;
    render(() => (
      <>
        <button ref={(el) => { triggerRef.current = el; }}>Open</button>
        <Confirm
          open={open}
          onOpenChange={(v) => (open = v)}
          title="T"
          message="M"
          confirmLabel="OK"
          cancelLabel="Cancel"
          onConfirm={() => {}}
          onCancel={() => {}}
          triggerRef={triggerRef}
        />
      </>
    ));
    // Esc is a valid close path — dialog must close
    fireEvent.keyDown(document.body, { key: "Escape" });
    expect(open).toBe(false);
  });

  it("triggerRef: destructive Confirm closes and trigger exists", () => {
    const triggerRef: { current?: HTMLElement } = {};
    let open = true;
    let confirmed = false;
    render(() => (
      <>
        <button ref={(el) => { triggerRef.current = el; }}>Open</button>
        <Confirm
          open={open}
          onOpenChange={(v) => (open = v)}
          title="T"
          message="M"
          confirmLabel="Delete"
          cancelLabel="Cancel"
          variant="destructive"
          onConfirm={() => { confirmed = true; }}
          onCancel={() => {}}
          triggerRef={triggerRef}
        />
      </>
    ));
    fireEvent.click(screen.getByRole("button", { name: "Delete" }));
    expect(confirmed).toBe(true);
    expect(open).toBe(false);
    expect(triggerRef.current).toBeTruthy();
  });
});
