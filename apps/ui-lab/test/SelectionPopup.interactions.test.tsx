import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, fireEvent, cleanup } from "@solidjs/testing-library";
import App from "../src/App";
import { strings, type SelectionState } from "../src/i18n";

/**
 * Interaction + async-safety regression tests for the Selection Popup.
 *
 * These reproduce the exact P1 scenarios from review round 2:
 *  - Retry on network error, then switch to 401 before the timer fires → the
 *    stale retry callback must NOT overwrite 401 with a success card.
 *  - Pinned → click Unpin → label/aria-pressed/visual must actually change.
 *  - Speak → observable playing/stopped toggle.
 *  - Copy → visible "Copied" feedback.
 *
 * Uses Vitest fake timers so the 1.2s retry window is deterministic.
 */

describe("Selection Popup — async safety (fake timers)", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });
  afterEach(() => {
    vi.useRealTimers();
    cleanup();
  });

  it("a stale retry callback does not overwrite a later state switch", () => {
    const { getByRole, queryByRole } = render(() => <App />);

    // 1. Network error
    fireEvent.click(getByRole("button", { name: /Error · network/ }));
    expect(getByRole("alert")).toBeTruthy();

    // 2. Click Retry — starts the 1200ms timer
    fireEvent.click(getByRole("button", { name: "Retry" }));

    // 3. Switch to 401 BEFORE the timer fires
    fireEvent.click(getByRole("button", { name: /Error · 401/ }));
    expect(getByRole("alert").textContent).toContain("401");

    // 4. Advance past the retry window — the stale callback must NOT replace
    //    the 401 error with a success card.
    vi.advanceTimersByTime(1400);
    // Still showing the 401 alert (not a success ResultCard)
    expect(getByRole("alert").textContent).toContain("401");
    // No success result card appeared (the retry's success was suppressed).
    expect(document.querySelector(".lr-result-card")).toBeNull();
  });

  it("retry succeeds when the state is not switched away", () => {
    const { getByRole, queryByRole } = render(() => <App />);
    fireEvent.click(getByRole("button", { name: /Error · network/ }));
    fireEvent.click(getByRole("button", { name: "Retry" }));
    vi.advanceTimersByTime(1300);
    // Error alert gone, success region now present
    expect(queryByRole("alert")).toBeNull();
    expect(getByRole("region")).toBeTruthy();
  });

  it("Pinned → Retry keeps the result card and shows a busy Retry button", () => {
    const { getByRole, container } = render(() => <App />);
    fireEvent.click(getByRole("button", { name: "Pinned" }));

    // One result card present before retry
    expect(container.querySelectorAll(".lr-result-card").length).toBe(1);

    // Click the pinned Retry button
    fireEvent.click(getByRole("button", { name: "Retry" }));

    // Result card MUST still be present (not blanked), and the region exposes
    // aria-busy so AT announces the retry. The pinned Retry button is disabled
    // (loading state) but still in the DOM.
    expect(container.querySelectorAll(".lr-result-card").length).toBe(1);
    const region = getByRole("region");
    expect(region.getAttribute("aria-busy")).toBe("true");
    // Body is not empty
    expect(region.textContent?.trim().length).toBeGreaterThan(0);
    // Pinned retry bar button still present, now disabled (loading)
    const pinnedBarBtn = container.querySelector(
      ".sel-popup__pinned-bar button",
    ) as HTMLElement;
    expect(pinnedBarBtn).toBeTruthy();
    expect(pinnedBarBtn.getAttribute("aria-busy")).toBe("true");
    expect(pinnedBarBtn.disabled).toBe(true);

    // After the retry window, busy state clears and card remains
    vi.advanceTimersByTime(1300);
    expect(container.querySelectorAll(".lr-result-card").length).toBe(1);
    expect(getByRole("region").getAttribute("aria-busy")).toBeNull();
  });
});

describe("Selection Popup — action feedback", () => {
  afterEach(() => cleanup());

  it("Pinned → Unpin: clicking Unpin flips label and aria-pressed", () => {
    const { getByRole, queryByRole } = render(() => <App />);
    fireEvent.click(getByRole("button", { name: "Pinned" }));

    // Initially pinned → button is "Unpin" and pressed
    const unpin = getByRole("button", { name: "Unpin" });
    expect(unpin.getAttribute("aria-pressed")).toBe("true");

    // Click to unpin
    fireEvent.click(unpin);

    // Now it flips back to "Pin". aria-pressed is absent when not active
    // (undefined renders no attribute), so it must be null — NOT "true".
    const pin = getByRole("button", { name: "Pin" });
    expect(pin.getAttribute("aria-pressed")).toBeNull();
    expect(queryByRole("button", { name: "Unpin" })).toBeNull();
  });

  it("Speak toggles between Speak and Stop", () => {
    const { getByRole, queryByRole } = render(() => <App />);
    fireEvent.click(getByRole("button", { name: "Success · single" }));

    const speak = getByRole("button", { name: "Speak" });
    fireEvent.click(speak);
    // Now shows Stop (active)
    const stop = getByRole("button", { name: "Stop" });
    expect(stop.getAttribute("aria-pressed")).toBe("true");

    // Click Stop → back to Speak
    fireEvent.click(stop);
    expect(queryByRole("button", { name: "Speak" })).toBeTruthy();
  });

  it("Copy shows visible 'Copied' feedback (check icon + aria-pressed)", () => {
    const { getByRole } = render(() => <App />);
    fireEvent.click(getByRole("button", { name: "Success · single" }));

    const copy = getByRole("button", { name: "Copy" });
    fireEvent.click(copy);
    // Label swaps to "Copied" and is active
    const copied = getByRole("button", { name: "Copied" });
    expect(copied.getAttribute("aria-pressed")).toBe("true");
  });
});

describe("Selection Popup — window states", () => {
  afterEach(() => cleanup());

  it("initial-hidden renders NO popup frame and NO popup region", () => {
    const { getByRole, queryByRole, container } = render(() => <App />);
    fireEvent.click(getByRole("button", { name: /Initial \(hidden\)/ }));
    // No popup region and no lab frame element
    expect(queryByRole("region")).toBeNull();
    expect(container.querySelector(".lab__frame")).toBeNull();
  });

  it("loading renders a compact frame (not 400×300) containing the loading card", () => {
    const { getByRole, container } = render(() => <App />);
    fireEvent.click(getByRole("button", { name: "Loading" }));
    const frame = container.querySelector(".lab__frame") as HTMLElement;
    expect(frame).toBeTruthy();
    // Compact width ~200px (style sets width via inline style)
    expect(frame.style.width).toBe("200px");
    expect(frame.style.height).toBe("40px");
  });
});

describe("Selection Popup — offline fallback", () => {
  afterEach(() => cleanup());

  it("offline-fallback uses a traditional MT engine, not a dictionary", () => {
    const { getByRole, queryByText } = render(() => <App />);
    fireEvent.click(getByRole("button", { name: "Offline · engine fallback" }));
    // The fallback result label must reference Google (traditional MT) +
    // the "offline fallback" suffix, and must NOT be labeled as a dictionary.
    const region = getByRole("region");
    expect(region.textContent).toContain("Google");
    expect(region.textContent).toContain("offline fallback");
    expect(queryByText(/Offline dictionary|离线词典/)).toBeNull();
  });
});

describe("Selection Popup — all states render", () => {
  // Parametrize every SelectionState (en labels) so none can silently regress.
  const STATES: SelectionState[] = [
    "initial-hidden",
    "loading",
    "success-single",
    "success-dual",
    "success-multi",
    "partial",
    "error-network",
    "error-config-key",
    "error-config-401",
    "error-no-selection",
    "error-no-provider",
    "error-no-permission",
    "keystore-corrupt",
    "offline-fallback",
    "offline-error",
    "pinned",
  ];

  it.each(STATES)("renders state '%s' without throwing", (state) => {
    const label = strings.en.selection.states[state];
    const { getByRole, container } = render(() => <App />);
    // Click the state chip by its exact visible label.
    fireEvent.click(getByRole("button", { name: label }));
    // Something rendered in the stage (frame for visible states, or the
    // hidden-note for initial-hidden).
    const hasContent =
      container.querySelector(".lab__frame") ||
      container.querySelector(".lab__hidden-note");
    expect(hasContent).toBeTruthy();
    cleanup();
  });
});
