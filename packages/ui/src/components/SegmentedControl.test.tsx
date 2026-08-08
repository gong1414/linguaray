import { describe, it, expect, vi } from "vitest";
import { render, fireEvent } from "@solidjs/testing-library";
import SegmentedControl from "./SegmentedControl";
import { assertNoAxeViolations } from "../../test/setup";

const opts = [
  { value: "a", label: "Alpha" },
  { value: "b", label: "Beta" },
  { value: "c", label: "Gamma" },
];

describe("SegmentedControl", () => {
  it("renders all options", () => {
    const { getByText } = render(() => <SegmentedControl options={opts} value="a" onChange={() => {}} ariaLabel="View" />);
    expect(getByText("Alpha")).toBeInTheDocument();
    expect(getByText("Gamma")).toBeInTheDocument();
  });

  it("aria-checked on correct radio", () => {
    const { getByText } = render(() => <SegmentedControl options={opts} value="b" onChange={() => {}} ariaLabel="View" />);
    expect(getByText("Beta").closest("[role='radio']")).toHaveAttribute("aria-checked", "true");
    expect(getByText("Alpha").closest("[role='radio']")).toHaveAttribute("aria-checked", "false");
  });

  it("click calls onChange", () => {
    const onChange = vi.fn();
    const { getByText } = render(() => <SegmentedControl options={opts} value="a" onChange={onChange} ariaLabel="View" />);
    fireEvent.click(getByText("Beta"));
    expect(onChange).toHaveBeenCalledWith("b");
  });

  it("disabled prevents onChange", () => {
    const onChange = vi.fn();
    const { getByText } = render(() => <SegmentedControl options={opts} value="a" onChange={onChange} ariaLabel="V" disabled />);
    fireEvent.click(getByText("Beta"));
    expect(onChange).not.toHaveBeenCalled();
  });

  it("ArrowRight activates next and moves DOM focus", () => {
    const onChange = vi.fn();
    const { getByText } = render(() => <SegmentedControl options={opts} value="a" onChange={onChange} ariaLabel="ViewMode" />);
    const tab = getByText("Alpha").closest("[role='radio']") as HTMLElement;
    tab.focus();
    expect(document.activeElement).toBe(tab);
    fireEvent.keyDown(tab, { key: "ArrowRight" });
    expect(onChange).toHaveBeenCalledWith("b");
    // 焦点应移到 Beta radio
    const betaTab = getByText("Beta").closest("[role='radio']") as HTMLElement;
    expect(document.activeElement).toBe(betaTab);
  });

  it("ArrowDown activates next and moves DOM focus", () => {
    const onChange = vi.fn();
    const { getByText } = render(() => <SegmentedControl options={opts} value="a" onChange={onChange} ariaLabel="ViewMode2" />);
    const tab = getByText("Alpha").closest("[role='radio']") as HTMLElement;
    tab.focus();
    fireEvent.keyDown(tab, { key: "ArrowDown" });
    expect(onChange).toHaveBeenCalledWith("b");
    expect(document.activeElement).toBe(getByText("Beta").closest("[role='radio']"));
  });

  it("ArrowLeft wraps to last and moves DOM focus", () => {
    const onChange = vi.fn();
    const { getByText } = render(() => <SegmentedControl options={opts} value="a" onChange={onChange} ariaLabel="ViewMode3" />);
    const tab = getByText("Alpha").closest("[role='radio']") as HTMLElement;
    tab.focus();
    fireEvent.keyDown(tab, { key: "ArrowLeft" });
    expect(onChange).toHaveBeenCalledWith("c");
    expect(document.activeElement).toBe(getByText("Gamma").closest("[role='radio']"));
  });

  it("ArrowUp wraps to last and moves DOM focus", () => {
    const onChange = vi.fn();
    const { getByText } = render(() => <SegmentedControl options={opts} value="a" onChange={onChange} ariaLabel="ViewMode4" />);
    const tab = getByText("Alpha").closest("[role='radio']") as HTMLElement;
    tab.focus();
    fireEvent.keyDown(tab, { key: "ArrowUp" });
    expect(onChange).toHaveBeenCalledWith("c");
    expect(document.activeElement).toBe(getByText("Gamma").closest("[role='radio']"));
  });

  it("Home/End", () => {
    const onChange = vi.fn();
    const { getByText } = render(() => <SegmentedControl options={opts} value="b" onChange={onChange} ariaLabel="V" />);
    const tab = getByText("Beta").closest("[role='radio']")!;
    fireEvent.keyDown(tab, { key: "Home" });
    expect(onChange).toHaveBeenLastCalledWith("a");
    fireEvent.keyDown(tab, { key: "End" });
    expect(onChange).toHaveBeenLastCalledWith("c");
  });

  it("roving tabindex", () => {
    const { getByText } = render(() => <SegmentedControl options={opts} value="b" onChange={() => {}} ariaLabel="V" />);
    expect(getByText("Beta").closest("[role='radio']")).toHaveAttribute("tabindex", "0");
    expect(getByText("Alpha").closest("[role='radio']")).toHaveAttribute("tabindex", "-1");
  });

  it("role=radiogroup aria-label", () => {
    const { getByRole } = render(() => <SegmentedControl options={opts} value="a" onChange={() => {}} ariaLabel="Mode" />);
    expect(getByRole("radiogroup")).toHaveAttribute("aria-label", "Mode");
  });

  it("no axe violations", async () => {
    render(() => <SegmentedControl options={opts} value="a" onChange={() => {}} ariaLabel="V" />);
    await assertNoAxeViolations({ disableRules: ["region"] });
  });
});
