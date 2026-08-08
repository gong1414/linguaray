import { describe, it, expect, vi } from "vitest";
import { render, fireEvent } from "@solidjs/testing-library";
import ShortcutChip from "./ShortcutChip";
import { assertNoAxeViolations } from "../../test/setup";

describe("ShortcutChip", () => {
  const labels = { recording: "Recording…", conflict: "Conflict", clear: "Clear shortcut" };

  it("renders shortcut text in clear status", () => {
    const { getByText } = render(() => <ShortcutChip shortcut="Ctrl+Shift+T" status="clear" labels={labels} />);
    expect(getByText("Ctrl+Shift+T")).toBeInTheDocument();
  });

  it("recording status shows recording label", () => {
    const { getByText } = render(() => <ShortcutChip shortcut="" status="recording" labels={labels} />);
    expect(getByText("Recording…")).toBeInTheDocument();
  });

  it("conflict status shows conflict label", () => {
    const { getByText } = render(() => <ShortcutChip shortcut="Ctrl+X" status="conflict" labels={labels} />);
    expect(getByText("Conflict")).toBeInTheDocument();
  });

  it("conflict status applies conflict visual class", () => {
    const { container } = render(() => <ShortcutChip shortcut="Ctrl+X" status="conflict" labels={labels} />);
    expect(container.querySelector(".shortcut-chip--conflict")).not.toBeNull();
  });

  it("onClear fires when clear button clicked", () => {
    const onClear = vi.fn();
    const { getByLabelText } = render(() => <ShortcutChip shortcut="Ctrl+X" status="clear" onClear={onClear} labels={labels} />);
    fireEvent.click(getByLabelText("Clear shortcut"));
    expect(onClear).toHaveBeenCalledOnce();
  });

  it("disabled hides clear button", () => {
    const onClear = vi.fn();
    const { queryByLabelText } = render(() => <ShortcutChip shortcut="Ctrl+X" status="clear" onClear={onClear} disabled labels={labels} />);
    expect(queryByLabelText("Clear shortcut")).toBeNull();
  });

  it("no axe violations", async () => {
    render(() => <ShortcutChip shortcut="Ctrl+T" status="clear" labels={labels} />);
    await assertNoAxeViolations();
  });
});
