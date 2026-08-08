import { describe, it, expect, vi } from "vitest";
import { render, fireEvent } from "@solidjs/testing-library";
import WindowChrome from "./WindowChrome";
import { assertNoAxeViolations } from "../../test/setup";

describe("WindowChrome", () => {
  const labels = { minimize: "Minimize", close: "Close" };
  it("renders children", () => {
    const { getByText } = render(() => <WindowChrome labels={labels}>Hello</WindowChrome>);
    expect(getByText("Hello")).toBeInTheDocument();
  });

  it("renders title when provided", () => {
    const { getByText } = render(() => <WindowChrome labels={labels} title="Settings">Body</WindowChrome>);
    expect(getByText("Settings")).toBeInTheDocument();
  });

  it("renders sidebar when provided", () => {
    const { getByText } = render(() => <WindowChrome labels={labels} sidebar={<nav>Sidebar</nav>}>Body</WindowChrome>);
    expect(getByText("Sidebar")).toBeInTheDocument();
  });

  it("onClose fires", () => {
    const onClose = vi.fn();
    const { getByLabelText } = render(() => <WindowChrome labels={labels} onClose={onClose}>Body</WindowChrome>);
    fireEvent.click(getByLabelText("Close"));
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("onMinimize fires", () => {
    const onMinimize = vi.fn();
    const { getByLabelText } = render(() => <WindowChrome labels={labels} onMinimize={onMinimize}>Body</WindowChrome>);
    fireEvent.click(getByLabelText("Minimize"));
    expect(onMinimize).toHaveBeenCalledOnce();
  });

  it("title bar is a drag region", () => {
    const { container } = render(() => <WindowChrome labels={labels} title="App" onClose={() => {}} onMinimize={() => {}}>Body</WindowChrome>);
    const header = container.querySelector(".window-chrome__header");
    expect(header).not.toBeNull();
    expect(header!).toHaveAttribute("data-tauri-drag-region");
  });

  it("minimize/close buttons opt out of drag region", () => {
    const { getByLabelText } = render(() => <WindowChrome labels={labels} title="App" onClose={() => {}} onMinimize={() => {}}>Body</WindowChrome>);
    expect(getByLabelText("Minimize")).toHaveAttribute("data-tauri-drag-region", "false");
    expect(getByLabelText("Close")).toHaveAttribute("data-tauri-drag-region", "false");
  });

  // MASTER §7 WindowChrome: minimize/close are ghost IconButtons (not bare
  // <button>). They must carry the IconButton class + ghost variant.
  it("minimize/close use ghost IconButton", () => {
    const { getByLabelText } = render(() => (
      <WindowChrome labels={labels} title="App" onClose={() => {}} onMinimize={() => {}}>Body</WindowChrome>
    ));
    const minimize = getByLabelText("Minimize");
    const close = getByLabelText("Close");
    expect(minimize.classList.contains("lr-icon-btn")).toBe(true);
    expect(minimize.classList.contains("lr-icon-btn--ghost")).toBe(true);
    expect(close.classList.contains("lr-icon-btn")).toBe(true);
    expect(close.classList.contains("lr-icon-btn--ghost")).toBe(true);
  });

  it("no axe violations", async () => {
    render(() => <WindowChrome labels={labels} title="Test"><p>Content</p></WindowChrome>);
    await assertNoAxeViolations();
  });
});
