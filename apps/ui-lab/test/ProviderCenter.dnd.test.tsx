/**
 * Provider Center — HTML5 drag-to-reorder (real DnD event path).
 *
 * These exercise the production dragstart/dragover/drop handlers — NOT the
 * keyboard Move up/down path. They prove the DataTransfer write, the
 * before/after midpoint indicator, the reorder commit, and the rollback.
 *
 * jsdom does not implement DataTransfer or layout, so we synthesize a DragEvent
 * carrying a minimal DataTransfer shim and mock getBoundingClientRect to give
 * the midpoint computation a real rect.
 */
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, fireEvent, screen, cleanup } from "@solidjs/testing-library";
import App from "../src/App";

// jsdom matchMedia mock
if (!window.matchMedia) {
  // @ts-expect-error partial mock
  window.matchMedia = () => ({
    matches: false,
    addEventListener() {},
    removeEventListener() {},
  });
}

function goToProviderCenter() {
  fireEvent.click(screen.getByRole("button", { name: "Provider Center" }));
}

function clickStateChip(label: string) {
  const stateBar = screen.getByRole("group", { name: "State" });
  const chips = [...stateBar.querySelectorAll("button")];
  const chip = chips.find((b) => b.textContent === label);
  expect(chip, `state chip "${label}" must exist`).toBeTruthy();
  fireEvent.click(chip!);
}

// Minimal DataTransfer shim — supports setData/getData + effectAllowed.
class MockDataTransfer {
  private store: Record<string, string> = {};
  dropEffect = "none";
  effectAllowed = "none";
  setData(type: string, val: string) { this.store[type] = val; }
  getData(type: string) { return this.store[type] ?? ""; }
}

// Build a DragEvent with a dataTransfer, since jsdom's DragEvent has none.
function makeDragEvent(
  type: "dragstart" | "dragover" | "drop" | "dragend",
  opts: { clientY?: number; dataTransfer?: MockDataTransfer; currentTarget?: HTMLElement } = {},
): DragEvent {
  const evt = new Event(type, { bubbles: true, cancelable: true });
  Object.defineProperty(evt, "dataTransfer", {
    value: opts.dataTransfer ?? new MockDataTransfer(),
    configurable: true,
  });
  Object.defineProperty(evt, "clientY", { value: opts.clientY ?? 0, configurable: true });
  if (opts.currentTarget) {
    Object.defineProperty(evt, "currentTarget", { value: opts.currentTarget, configurable: true });
  }
  return evt as DragEvent;
}

// Give a row a deterministic rect so the midpoint logic is testable.
function mockRowRect(row: HTMLElement, top: number, height: number) {
  row.getBoundingClientRect = () => ({
    top, height, bottom: top + height, left: 0, right: 0, width: 0, x: 0, y: top, toJSON() {},
  });
}

describe("Provider Center — HTML5 drag-to-reorder", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });
  afterEach(() => {
    vi.useRealTimers();
    cleanup();
  });

  it("dragstart writes the uuid to dataTransfer and marks the row dragging", () => {
    render(() => <App />);
    goToProviderCenter();
    clickStateChip("Drag to reorder");
    // First provider's drag handle
    const handle = document.querySelector(".pc__drag-handle") as HTMLElement;
    expect(handle).toBeTruthy();
    const dt = new MockDataTransfer();
    fireEvent(handle, makeDragEvent("dragstart", { dataTransfer: dt }));
    // Production handler sets text/plain + effectAllowed
    expect(dt.getData("text/plain")).toBe("mock-openai-1");
    expect(dt.effectAllowed).toBe("move");
    // The dragged row gets the dragging class
    const row = handle.closest(".pc__provider-row");
    expect(row?.classList.contains("pc__provider-row--dragging")).toBe(true);
  });

  it("dragover above the midpoint sets the 'before' indicator; below sets 'after'", () => {
    render(() => <App />);
    goToProviderCenter();
    clickStateChip("Drag to reorder");
    const handles = [...document.querySelectorAll(".pc__drag-handle")] as HTMLElement[];
    const rows = [...document.querySelectorAll(".pc__provider-row")] as HTMLElement[];
    // Start dragging OpenAI #1 (row 0)
    fireEvent(handles[0]!, makeDragEvent("dragstart"));
    // Target = DeepSeek row (index 2). Give it a rect: top=100, height=40 → midpoint=120
    const deepseekRow = rows[2]!;
    mockRowRect(deepseekRow, 100, 40);
    // clientY = 105 (< 120) → before
    fireEvent(deepseekRow, makeDragEvent("dragover", { clientY: 105, currentTarget: deepseekRow }));
    expect(deepseekRow.classList.contains("pc__provider-row--drag-over-before")).toBe(true);
    expect(deepseekRow.classList.contains("pc__provider-row--drag-over-after")).toBe(false);
    // clientY = 130 (> 120) → after
    fireEvent(deepseekRow, makeDragEvent("dragover", { clientY: 130, currentTarget: deepseekRow }));
    expect(deepseekRow.classList.contains("pc__provider-row--drag-over-after")).toBe(true);
    expect(deepseekRow.classList.contains("pc__provider-row--drag-over-before")).toBe(false);
  });

  it("drop on a target reorders the provider (before position)", () => {
    render(() => <App />);
    goToProviderCenter();
    clickStateChip("Drag to reorder");
    const handles = [...document.querySelectorAll(".pc__drag-handle")] as HTMLElement[];
    const rows = [...document.querySelectorAll(".pc__provider-row")] as HTMLElement[];
    const namesBefore = [...document.querySelectorAll(".lr-provider-card")]
      .map((c) => c.getAttribute("data-template"));
    // Drag DeepSeek (row 2) → drop on OpenAI #1 (row 0) BEFORE
    fireEvent(handles[2]!, makeDragEvent("dragstart"));
    const openaiRow = rows[0]!;
    mockRowRect(openaiRow, 0, 40);
    fireEvent(openaiRow, makeDragEvent("dragover", { clientY: 5, currentTarget: openaiRow }));
    fireEvent(openaiRow, makeDragEvent("drop"));
    const namesAfter = [...document.querySelectorAll(".lr-provider-card")]
      .map((c) => c.getAttribute("data-template"));
    // deepseek moved to index 0; order changed
    expect(namesAfter[0]).toBe("deepseek");
    expect(namesBefore[0]).toBe("openai");
    expect(namesAfter).not.toEqual(namesBefore);
  });

  it("drop without a prior dragstart is a no-op (no reorder)", () => {
    render(() => <App />);
    goToProviderCenter();
    clickStateChip("Drag to reorder");
    const rows = [...document.querySelectorAll(".pc__provider-row")] as HTMLElement[];
    const namesBefore = [...document.querySelectorAll(".lr-provider-card")]
      .map((c) => c.getAttribute("data-template"));
    // Drop with no draggedUuid set
    fireEvent(rows[0]!, makeDragEvent("drop"));
    const namesAfter = [...document.querySelectorAll(".lr-provider-card")]
      .map((c) => c.getAttribute("data-template"));
    expect(namesAfter).toEqual(namesBefore);
  });

  it("reorder-failed: drag reorder commits then rolls back after the persist timeout", () => {
    render(() => <App />);
    goToProviderCenter();
    clickStateChip("Reorder failed");
    const handles = [...document.querySelectorAll(".pc__drag-handle")] as HTMLElement[];
    const rows = [...document.querySelectorAll(".pc__provider-row")] as HTMLElement[];
    const orderBefore = [...document.querySelectorAll(".lr-provider-card")]
      .map((c) => c.getAttribute("data-template"));
    // Drag DeepSeek (row 2) before OpenAI #1 (row 0)
    fireEvent(handles[2]!, makeDragEvent("dragstart"));
    mockRowRect(rows[0]!, 0, 40);
    fireEvent(rows[0]!, makeDragEvent("dragover", { clientY: 5, currentTarget: rows[0]! }));
    fireEvent(rows[0]!, makeDragEvent("drop"));
    // Immediately after: order changed (deepseek first)
    const orderImmediately = [...document.querySelectorAll(".lr-provider-card")]
      .map((c) => c.getAttribute("data-template"));
    expect(orderImmediately[0]).toBe("deepseek");
    expect(orderImmediately).not.toEqual(orderBefore);
    // Advance past the rollback timer (800ms)
    vi.advanceTimersByTime(900);
    // Order reverted to the snapshot
    const orderAfterRollback = [...document.querySelectorAll(".lr-provider-card")]
      .map((c) => c.getAttribute("data-template"));
    expect(orderAfterRollback).toEqual(orderBefore);
  });

  it("drag end clears all drag state (no lingering indicator)", () => {
    render(() => <App />);
    goToProviderCenter();
    clickStateChip("Drag to reorder");
    const handles = [...document.querySelectorAll(".pc__drag-handle")] as HTMLElement[];
    const rows = [...document.querySelectorAll(".pc__provider-row")] as HTMLElement[];
    fireEvent(handles[0]!, makeDragEvent("dragstart"));
    expect(rows[0]!.classList.contains("pc__provider-row--dragging")).toBe(true);
    // dragend clears state
    fireEvent(handles[0]!, makeDragEvent("dragend"));
    expect(rows[0]!.classList.contains("pc__provider-row--dragging")).toBe(false);
  });
});
