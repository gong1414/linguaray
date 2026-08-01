import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, fireEvent } from "@solidjs/testing-library";
import { createSignal, Show } from "solid-js";
import Toast from "./Toast";

describe("Toast", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  it("info toast has role=status", () => {
    const { getByRole } = render(() => (
      <Toast variant="info" message="Saved" onDismiss={() => {}} />
    ));
    expect(getByRole("status")).toBeTruthy();
  });

  it("destructive toast has role=alert", () => {
    const { getByRole } = render(() => (
      <Toast variant="destructive" message="Failed" onDismiss={() => {}} />
    ));
    expect(getByRole("alert")).toBeTruthy();
  });

  it("info auto-dismisses after 3s", () => {
    const [show, setShow] = createSignal(true);
    const { queryByText } = render(() => (
      <Show when={show()}>
        <Toast variant="info" message="Saved" onDismiss={() => setShow(false)} />
      </Show>
    ));
    vi.advanceTimersByTime(3100);
    expect(queryByText("Saved")).toBeNull();
  });

  it("destructive does NOT auto-dismiss", () => {
    const [show, setShow] = createSignal(true);
    const { queryByText } = render(() => (
      <Show when={show()}>
        <Toast variant="destructive" message="Failed" onDismiss={() => setShow(false)} />
      </Show>
    ));
    vi.advanceTimersByTime(5000);
    expect(queryByText("Failed")).toBeTruthy();
  });

  it("dismiss button calls onDismiss", () => {
    let dismissed = false;
    const { getByRole } = render(() => (
      <Toast variant="info" message="Hi" onDismiss={() => (dismissed = true)} />
    ));
    fireEvent.click(getByRole("button", { name: "Dismiss" }));
    expect(dismissed).toBe(true);
  });

  it("timer is cleaned up on unmount (no callback after dispose)", () => {
    let dismissed = false;
    const { unmount } = render(() => (
      <Toast variant="info" message="Hi" onDismiss={() => (dismissed = true)} />
    ));
    // Unmount BEFORE the timer fires
    unmount();
    vi.advanceTimersByTime(5000);
    // onDismiss should NOT have been called after unmount
    expect(dismissed).toBe(false);
  });
});
