import { describe, it, expect, vi } from "vitest";
import { render, fireEvent } from "@solidjs/testing-library";
import HistoryRow from "./HistoryRow";
import { assertNoAxeViolations } from "../../test/setup";

const baseProps = {
  sourceText: "Hello",
  resultPreview: "你好",
  timestamp: "2026-01-01 12:00",
  engineLabel: "Google",
  labels: { addFavorite: "Add to favorites", removeFavorite: "Remove from favorites" },
};

describe("HistoryRow", () => {
  it("renders texts", () => {
    const { getByText } = render(() => <HistoryRow {...baseProps} />);
    expect(getByText("Hello")).toBeInTheDocument();
    expect(getByText("你好")).toBeInTheDocument();
    expect(getByText("Google")).toBeInTheDocument();
    expect(getByText("2026-01-01 12:00")).toBeInTheDocument();
  });

  it("onClick fires", () => {
    const onClick = vi.fn();
    const { getByText } = render(() => <HistoryRow {...baseProps} onClick={onClick} />);
    fireEvent.click(getByText("Hello"));
    expect(onClick).toHaveBeenCalledOnce();
  });

  it("onToggleFavorite fires", () => {
    const onToggle = vi.fn();
    const { getByLabelText } = render(() => <HistoryRow {...baseProps} onToggleFavorite={onToggle} />);
    fireEvent.click(getByLabelText("Add to favorites"));
    expect(onToggle).toHaveBeenCalledOnce();
  });

  it("favorite button is NOT nested inside onClick button (DOM structure)", () => {
    const { container } = render(() => (
      <HistoryRow {...baseProps} onClick={() => {}} onToggleFavorite={() => {}} />
    ));
    const buttons = container.querySelectorAll("button");
    expect(buttons.length).toBe(2);
    expect(buttons[0].contains(buttons[1])).toBe(false);
    expect(buttons[1].contains(buttons[0])).toBe(false);
  });

  it("no onClick renders non-interactive div (not button)", () => {
    const { container } = render(() => <HistoryRow {...baseProps} />);
    expect(container.querySelector("button.history-row__content")).toBeNull();
    expect(container.querySelector("div.history-row__content")).not.toBeNull();
  });

  it("favorite button always has a non-empty aria-label (labels required)", () => {
    const { getByRole } = render(() => (
      <HistoryRow {...baseProps} favorite={false} onToggleFavorite={() => {}} />
    ));
    const fav = getByRole("button", { name: "Add to favorites" });
    expect(fav.getAttribute("aria-label") ?? "").not.toBe("");
  });

  it("with onClick renders button", () => {
    const { container } = render(() => <HistoryRow {...baseProps} onClick={() => {}} />);
    expect(container.querySelector("button.history-row__content")).not.toBeNull();
  });

  it("no axe violations", async () => {
    render(() => <HistoryRow {...baseProps} onToggleFavorite={() => {}} />);
    // Isolated row lacks a landmark ancestor (matches ResultCard/Banner pattern).
    await assertNoAxeViolations({ disableRules: ["region"] });
  });
});
