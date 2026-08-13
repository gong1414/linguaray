import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { initTheme } from "../src/theme";

describe("initTheme", () => {
  const original = { ...document.documentElement.dataset };

  beforeEach(() => {
    document.documentElement.removeAttribute("data-theme");
    document.documentElement.removeAttribute("data-motion");
    document.documentElement.removeAttribute("lang");
    localStorage.clear();
    document.querySelectorAll("meta[name=theme-color]").forEach((m) => m.remove());
  });

  afterEach(() => {
    vi.restoreAllMocks();
    for (const k of Object.keys(document.documentElement.dataset)) {
      delete document.documentElement.dataset[k];
    }
    for (const [k, v] of Object.entries(original)) {
      if (v !== undefined) document.documentElement.dataset[k] = v;
    }
    document.querySelectorAll("meta[name=theme-color]").forEach((m) => m.remove());
  });

  it("sets data-theme from localStorage when present", () => {
    localStorage.setItem("linguaray.theme", "dark");
    initTheme();
    expect(document.documentElement.dataset.theme).toBe("dark");
  });

  it("falls back to prefers-color-scheme when localStorage is unset", () => {
    vi.spyOn(window, "matchMedia").mockImplementation((q) => ({
      matches: q.includes("dark"),
      media: q,
      addEventListener: () => {},
      removeEventListener: () => {},
      addListener: () => {},
      removeListener: () => {},
      onchange: null,
      dispatchEvent: () => true,
    } as unknown as MediaQueryList));
    initTheme();
    expect(document.documentElement.dataset.theme).toBe("dark");
  });

  it("sets data-motion=reduced when prefers-reduced-motion matches", () => {
    vi.spyOn(window, "matchMedia").mockImplementation((q) => ({
      matches: q.includes("reduced-motion"),
      media: q,
      addEventListener: () => {},
      removeEventListener: () => {},
      addListener: () => {},
      removeListener: () => {},
      onchange: null,
      dispatchEvent: () => true,
    } as unknown as MediaQueryList));
    initTheme();
    expect(document.documentElement.dataset.motion).toBe("reduced");
  });

  it("sets data-motion=full when reduced-motion does not match", () => {
    vi.spyOn(window, "matchMedia").mockImplementation(() => ({
      matches: false,
      media: "",
      addEventListener: () => {},
      removeEventListener: () => {},
      addListener: () => {},
      removeListener: () => {},
      onchange: null,
      dispatchEvent: () => true,
    } as unknown as MediaQueryList));
    initTheme();
    expect(document.documentElement.dataset.motion).toBe("full");
  });

  it("sets lang to the detected locale", () => {
    localStorage.setItem("linguaray.locale", "zh");
    initTheme();
    expect(document.documentElement.lang).toBe("zh");
  });

  it("activates the resolved-scheme theme-color meta with media=all and disables the other", () => {
    const light = document.createElement("meta");
    light.setAttribute("name", "theme-color");
    light.setAttribute("media", "(prefers-color-scheme: light)");
    light.setAttribute("content", "#F8FAFC");
    light.setAttribute("data-theme-scheme", "light");
    const dark = document.createElement("meta");
    dark.setAttribute("name", "theme-color");
    dark.setAttribute("media", "(prefers-color-scheme: dark)");
    dark.setAttribute("content", "#020617");
    dark.setAttribute("data-theme-scheme", "dark");
    document.head.append(light, dark);

    localStorage.setItem("linguaray.theme", "dark");
    initTheme();

    const metas = document.querySelectorAll<HTMLMetaElement>("meta[name=theme-color]");
    expect(metas.length).toBe(2);
    const activeDark = Array.from(metas).find((m) => m.getAttribute("content") === "#020617");
    expect(activeDark, "dark theme-color meta must exist").toBeTruthy();
    expect(activeDark!.getAttribute("media")).toBe("all");
    const disabled = Array.from(metas).find((m) => m.getAttribute("media") === "disabled");
    expect(disabled, "non-current scheme meta must be disabled").toBeTruthy();
  });

  it("a FORCED theme wins over the OS preference (user Dark while OS Light)", () => {
    vi.spyOn(window, "matchMedia").mockImplementation((q) => ({
      matches: q.includes("light"),
      media: q,
      addEventListener: () => {},
      removeEventListener: () => {},
      addListener: () => {},
      removeListener: () => {},
      onchange: null,
      dispatchEvent: () => true,
    } as unknown as MediaQueryList));

    const light = document.createElement("meta");
    light.setAttribute("name", "theme-color");
    light.setAttribute("media", "(prefers-color-scheme: light)");
    light.setAttribute("content", "#F8FAFC");
    light.setAttribute("data-theme-scheme", "light");
    const dark = document.createElement("meta");
    dark.setAttribute("name", "theme-color");
    dark.setAttribute("media", "(prefers-color-scheme: dark)");
    dark.setAttribute("content", "#020617");
    dark.setAttribute("data-theme-scheme", "dark");
    document.head.append(light, dark);

    localStorage.setItem("linguaray.theme", "dark");
    initTheme();

    const metas = document.querySelectorAll<HTMLMetaElement>("meta[name=theme-color]");
    const activeDark = Array.from(metas).find((m) => m.getAttribute("content") === "#020617");
    expect(activeDark, "forced-dark meta must be the active one").toBeTruthy();
    expect(activeDark!.getAttribute("media")).toBe("all");
    const activeLight = Array.from(metas).find((m) => m.getAttribute("content") === "#F8FAFC");
    expect(activeLight!.getAttribute("media")).toBe("disabled");
  });

  it("remains valid when initialized without metas then theme changes (0-meta → dark → light)", () => {
    localStorage.setItem("linguaray.theme", "dark");
    initTheme();
    localStorage.setItem("linguaray.theme", "light");
    initTheme();
    const metas = [...document.querySelectorAll<HTMLMetaElement>('meta[name="theme-color"]')];
    expect(metas.filter((m) => m.getAttribute("media") === "all")).toHaveLength(1);
    expect(metas.find((m) => m.getAttribute("media") === "all")?.getAttribute("data-theme-scheme")).toBe("light");
    expect(metas.find((m) => m.getAttribute("media") === "all")?.getAttribute("content")).toBe("#F8FAFC");
  });
});
