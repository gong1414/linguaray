import { readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

/**
 * Hygiene-2 window permission gate.
 *
 * Tauri window operations (`getCurrentWindow().hide()` …) are IPC commands
 * gated by the calling window's capability file. A missing
 * `core:window:allow-<op>` permission means the button/flow silently does
 * nothing (the R6 dead permission-button failure mode). This test:
 *
 *  1. DISCOVERY — scans src/ for every `getCurrentWindow().<op>()` /
 *     `gw().<op>()` call and fails if the calling file is not registered in
 *     WINDOW_FILES (so a new window consumer can never appear unmapped).
 *  2. COVERAGE — for every window, each op its files actually call must be
 *     granted by src-tauri/capabilities/<label>.json.
 *  3. LEAST PRIVILEGE — the main (settings) window runs inside the OS-native
 *     title bar and must grant NO window ops. Custom chrome on a decorated
 *     window produced the double title bar; this lock makes its return (or an
 *     accidental permission dump into main.json) a deliberate test change.
 */

const ROOT = join(__dirname, "..");

/** Entry-file → window-label map (mirrors the tauri.conf.json windows).
 * All windows are React (flattened src/{app,features}) after the Phase 4/5
 * migration + refactor P1.1. */
const WINDOW_FILES: Record<string, string[]> = {
  main: ["src/features/shell/controller.ts"],
  popup: ["src/features/translation/popupController.ts"],
  onboarding: ["src/features/onboarding/controller.ts"],
  ocr: ["src/features/ocr/controller.ts"],
};

/** Window ops that map 1:1 to a `core:window:allow-<kebab>` permission. */
const WINDOW_OPS = new Set([
  "close", "destroy", "hide", "show", "minimize", "maximize", "unmaximize",
  "setTitle", "setFocus", "setPosition", "setSize", "startDragging",
  "setAlwaysOnTop", "setResizable", "center",
]);

/** Event subscriptions are covered by the `core:default` permission set. */
const EVENT_METHODS = new Set(["onFocusChanged", "onResized", "onMoved"]);

const TRACKED_METHODS = new Set([...WINDOW_OPS, ...EVENT_METHODS]);

function walk(dir: string, out: string[] = []): string[] {
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    if (statSync(p).isDirectory()) walk(p, out);
    else if (/\.(ts|tsx)$/.test(name) && !/\.test\./.test(name)) out.push(p);
  }
  return out;
}

type Call = { rel: string; method: string };

const calls: Call[] = [];
for (const abs of walk(join(ROOT, "src"))) {
  const text = readFileSync(abs, "utf8");
  const rel = abs.slice(ROOT.length + 1);
  // `getCurrentWindow().op(` — including call sites split across lines
  // (`void getCurrentWindow()\n  .show()`), hence \s* around the dot.
  for (const m of text.matchAll(/getCurrentWindow\(\)\s*\.\s*(\w+)\(/g)) {
    calls.push({ rel, method: m[1] });
  }
  // Onboarding aliases the dynamic import (`gw().onFocusChanged(`).
  for (const m of text.matchAll(/\bgw\(\)\s*\.\s*(\w+)\(/g)) {
    calls.push({ rel, method: m[1] });
  }
}
const tracked = calls.filter((c) => TRACKED_METHODS.has(c.method));

const kebab = (m: string) => m.replace(/[A-Z]/g, (c) => `-${c.toLowerCase()}`);

function capabilityPermissions(label: string): string[] {
  const cap = JSON.parse(
    readFileSync(join(ROOT, "src-tauri", "capabilities", `${label}.json`), "utf8"),
  ) as { permissions: (string | { identifier: string })[] };
  return cap.permissions.map((p) => (typeof p === "string" ? p : p.identifier));
}

describe("window permission gate (hygiene-2)", () => {
  it("every src file calling window APIs is registered in WINDOW_FILES", () => {
    const mapped = new Set(Object.values(WINDOW_FILES).flat());
    const unmapped = [...new Set(
      tracked
        .filter((c) => !mapped.has(c.rel))
        .map((c) => `${c.rel}: ${c.method}()`),
    )];
    expect(
      unmapped,
      "Unregistered window-API consumer(s). Add the file to WINDOW_FILES in " +
      "test/window-permission-gate.test.ts so its capability stays covered.",
    ).toEqual([]);
  });

  it("every mapped file exists", () => {
    for (const files of Object.values(WINDOW_FILES)) {
      for (const f of files) {
        expect(statSync(join(ROOT, f)).isFile(), `${f} listed in WINDOW_FILES but missing`).toBe(true);
      }
    }
  });

  for (const [label, files] of Object.entries(WINDOW_FILES)) {
    it(`${label} capability grants every window op its frontend calls`, () => {
      const perms = capabilityPermissions(label);
      const ops = [
        ...new Set(tracked.filter((c) => files.includes(c.rel) && WINDOW_OPS.has(c.method)).map((c) => c.method)),
      ].sort();
      for (const op of ops) {
        expect(
          perms,
          `${label}.json must grant core:window:allow-${kebab(op)} for the ${op}() call in ${files.join(", ")}`,
        ).toContain(`core:window:allow-${kebab(op)}`);
      }
      if (tracked.some((c) => files.includes(c.rel) && EVENT_METHODS.has(c.method))) {
        expect(perms).toContain("core:default");
      }
    });
  }

  it("main (settings) window grants NO window ops — native title bar only", () => {
    const windowPerms = capabilityPermissions("main").filter((p) => p.startsWith("core:window:allow-"));
    expect(
      windowPerms,
      "The settings window uses the OS-native title bar. If a window op is " +
      "genuinely needed, call it from src/ and update this lock deliberately.",
    ).toEqual([]);
  });
});
