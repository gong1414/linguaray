import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, fireEvent, screen, cleanup } from "@solidjs/testing-library";
import App from "../src/App";
import { strings } from "../src/i18n";

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

function selectProvider(name: string) {
  // Click the edit button on a provider card to open detail
  const editBtn = screen.getByRole("button", { name: `Edit ${name}` });
  fireEvent.click(editBtn);
}

describe("Provider Center — async transitions (fake timers)", () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => {
    vi.useRealTimers();
    cleanup();
  });

  it("connection test: testing → ok after delay", () => {
    render(() => <App />);
    goToProviderCenter();
    fireEvent.click(screen.getByRole("button", { name: "Connection OK" }));
    selectProvider("OpenAI #1");
    // Click Test
    fireEvent.click(screen.getByRole("button", { name: "Test" }));
    // Advance past the 1.2s timer
    vi.advanceTimersByTime(1300);
    // Should show connection OK + latency
    expect(screen.getByText(/Connected/)).toBeTruthy();
  });

  it("save key: saving → saved badge appears", () => {
    render(() => <App />);
    goToProviderCenter();
    fireEvent.click(screen.getByRole("button", { name: "Connection OK" }));
    // Select OpenAI #2 which has no key → shows the key input
    selectProvider("OpenAI #2 (copy)");
    const keyInput = screen.getByPlaceholderText(strings.en.provider.apiKeyPlaceholder);
    fireEvent.input(keyInput, { target: { value: "sk-fake-test-key-1234567890" } });
    fireEvent.click(screen.getByRole("button", { name: "Save key" }));
    vi.advanceTimersByTime(1100);
    // After save, the key input is gone (badge replaces it)
    expect(screen.queryByPlaceholderText(strings.en.provider.apiKeyPlaceholder)).toBeNull();
    // Key saved appears in the detail panel
    expect(screen.getAllByText(strings.en.provider.keySaved).length).toBeGreaterThan(0);
  });

  it("delete: confirm → deleting → removed", () => {
    render(() => <App />);
    goToProviderCenter();
    fireEvent.click(screen.getByRole("button", { name: "Connection OK" }));
    // Count providers before
    const cardsBefore = document.querySelectorAll(".lr-provider-card").length;
    // Click delete on the first card
    const deleteBtn = screen.getAllByRole("button", { name: /Delete OpenAI #1/ })[0];
    fireEvent.click(deleteBtn);
    // Confirm dialog appears — click Delete in dialog
    fireEvent.click(screen.getAllByRole("button", { name: "Delete" }).slice(-1)[0]!);
    // Advance past the 1.5s delete timer
    vi.advanceTimersByTime(1600);
    // One fewer card
    const cardsAfter = document.querySelectorAll(".lr-provider-card").length;
    expect(cardsAfter).toBe(cardsBefore - 1);
  });

  it("reorder-failed state reverts order on persist failure", () => {
    render(() => <App />);
    goToProviderCenter();
    fireEvent.click(screen.getByRole("button", { name: "Reorder failed" }));
    // Move the second provider up — triggers the persist-failure path
    const moveUpBtns = screen.getAllByRole("button", { name: "Move up" });
    fireEvent.click(moveUpBtns[1]);
    vi.advanceTimersByTime(900);
    // Revert message appears (aria-live + toast)
    expect(screen.getAllByText(strings.en.provider.reorderReverted).length).toBeGreaterThan(0);
  });
});

describe("Provider Center — consent flow", () => {
  afterEach(() => cleanup());

  it("add-to-parallel opens consent dialog", () => {
    render(() => <App />);
    goToProviderCenter();
    fireEvent.click(screen.getByRole("button", { name: "Connection OK" }));
    // Click "Add to parallel" on a non-parallel provider
    const addBtns = screen.getAllByRole("button", { name: "Add to parallel" });
    fireEvent.click(addBtns[0]);
    // Consent dialog should appear
    expect(screen.getByText(strings.en.provider.consentTitle)).toBeTruthy();
  });

  it("consent Cancel does not add to parallel", () => {
    render(() => <App />);
    goToProviderCenter();
    fireEvent.click(screen.getByRole("button", { name: "Connection OK" }));
    const parallelBefore = document.querySelectorAll('[data-role="parallel"]').length;
    const addBtns = screen.getAllByRole("button", { name: "Add to parallel" });
    // addBtns[1] = OpenAI #2 (not primary, not parallel) — safe to add
    fireEvent.click(addBtns[1]);
    // Cancel the consent
    fireEvent.click(screen.getByRole("button", { name: "Cancel" }));
    const parallelAfter = document.querySelectorAll('[data-role="parallel"]').length;
    expect(parallelAfter).toBe(parallelBefore);
  });

  it("consent Confirm adds to parallel", () => {
    render(() => <App />);
    goToProviderCenter();
    fireEvent.click(screen.getByRole("button", { name: "Connection OK" }));
    const parallelBefore = document.querySelectorAll('[data-role="parallel"]').length;
    const addBtns = screen.getAllByRole("button", { name: "Add to parallel" });
    // addBtns[1] = OpenAI #2 (not primary, not parallel)
    fireEvent.click(addBtns[1]);
    // Click the consent dialog's Confirm button
    const confirmBtns = screen.getAllByRole("button", { name: "Confirm" });
    fireEvent.click(confirmBtns[confirmBtns.length - 1]!);
    const parallelAfter = document.querySelectorAll('[data-role="parallel"]').length;
    expect(parallelAfter).toBe(parallelBefore + 1);
  });
});
