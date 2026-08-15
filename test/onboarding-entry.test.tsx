import { describe, expect, it, vi, beforeEach } from "vitest";
import { readFileSync } from "node:fs";
import { join } from "node:path";

/**
 * R6 regression: the onboarding ENTRY must load the design-system stylesheet
 * and run initTheme() before rendering — the shipped v0.1.0 onboarding looked
 * like an unstyled webpage exactly because both were missing.
 */

const { initThemeMock, invokeMock } = vi.hoisted(() => ({
  initThemeMock: vi.fn(),
  invokeMock: vi.fn(async (cmd: string) => {
    if (cmd === "onboarding_status") return { complete: false, step: "welcome" };
    if (cmd === "a11y_status") return false;
    if (cmd === "screen_capture_status") return true;
    return undefined;
  }),
}));

vi.mock("../src/theme", () => ({ initTheme: initThemeMock }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: invokeMock }));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    hide: vi.fn(async () => undefined),
    onFocusChanged: () => Promise.resolve(() => {}),
  }),
}));
vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: vi.fn(async () => undefined) }));

import { mountOnboarding } from "../src/onboarding-entry";

beforeEach(() => {
  localStorage.setItem("linguaray.locale", "en");
  // NOTE: initThemeMock is NOT cleared — the entry executes it at import
  // time (static import at the top of this file), and the assertion is
  // "the wiring ran", not a per-call count.
  invokeMock.mockClear();
});

describe("onboarding entry", () => {
  it("loads the design-system stylesheet and runs initTheme", async () => {
    // The stylesheet import is side-effect-only and inert under jsdom (vitest
    // resolves it via alias, so vi.mock never sees it) — assert it exists in
    // the entry SOURCE. This is exactly the v0.1.0 regression: the import was
    // missing and the window rendered unstyled.
    const entry = readFileSync(join(__dirname, "..", "src", "onboarding-entry.tsx"), "utf8");
    expect(entry).toMatch(/import "@linguaray\/ui\/styles";/);
    await import("../src/onboarding-entry");
    expect(initThemeMock).toHaveBeenCalled();
  });

  it("mountOnboarding renders the onboarding surface", () => {
    const root = document.createElement("div");
    mountOnboarding(root);
    expect(root.querySelector('[data-testid="onboarding"]')).not.toBeNull();
    // Design-system buttons are present (lr-btn class from @linguaray/ui).
    expect(root.querySelector(".lr-btn")).not.toBeNull();
  });
});
