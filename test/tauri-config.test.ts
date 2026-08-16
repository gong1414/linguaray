import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

/**
 * R6 startup-window regression: the OCR overlay window must NOT be
 * pre-declared in tauri.conf.json. A hidden pre-created WebView still boots
 * ocr.html at startup (tauri#10950) and shipped builds occasionally surfaced
 * it as a stray normal window next to onboarding. On macOS region OCR uses
 * the system picker; on Windows the overlay is created ON DEMAND by
 * ocr_capture. Startup must show ONLY the declared windows below.
 */

type WindowDef = { label?: string; [k: string]: unknown };

const conf = JSON.parse(
  readFileSync(join(__dirname, "..", "src-tauri", "tauri.conf.json"), "utf8"),
) as { app: { windows: WindowDef[] } };

const labels = conf.app.windows.map((w) => w.label);

describe("tauri.conf.json window declarations", () => {
  it("declares exactly the four startup windows", () => {
    expect([...labels].sort()).toEqual(["input", "main", "onboarding", "popup"]);
  });

  it("does NOT pre-declare the OCR overlay window", () => {
    expect(labels).not.toContain("ocr");
  });

  it("onboarding window keeps its 600×400 stage", () => {
    const onboarding = conf.app.windows.find((w) => w.label === "onboarding");
    expect(onboarding).toBeDefined();
    expect(onboarding!.width).toBe(600);
    expect(onboarding!.height).toBe(400);
  });
});

/**
 * Hygiene-3: Debug/Release identity split. `pnpm dev:app` merges
 * tauri.debug.conf.json on top of tauri.conf.json (CLI --config has the
 * highest merge priority and REPLACES the windows array wholesale, hence the
 * duplicated array). Dev builds get a distinct bundle id, product name and
 * window titles so a running dev instance can never be confused with the
 * installed release (different app-data dir, different menu name, "Dev"
 * fingerprint in every decorated window title).
 */
const debugConf = JSON.parse(
  readFileSync(join(__dirname, "..", "src-tauri", "tauri.debug.conf.json"), "utf8"),
) as {
  productName: string;
  identifier: string;
  app: { windows: WindowDef[] };
};

describe("tauri.debug.conf.json (dev identity)", () => {
  it("uses a separate bundle id + product name", () => {
    expect(debugConf.identifier).toBe("io.github.gong1414.linguaray.debug");
    expect(debugConf.productName).toBe("LinguaRay Dev");
  });

  it("declares the same window labels as the base config", () => {
    expect(debugConf.app.windows.map((w) => w.label)).toEqual(labels);
  });

  it("mirrors every base window field EXCEPT title (drift gate)", () => {
    // --config merge replaces arrays wholesale, so the debug windows array is
    // a copy. If tauri.conf.json window settings change, this fails until the
    // debug copy is updated — no silent drift between dev and release.
    for (const dw of debugConf.app.windows) {
      const base = conf.app.windows.find((w) => w.label === dw.label);
      expect(base, `base window ${dw.label} missing`).toBeDefined();
      for (const key of [...new Set([...Object.keys(dw), ...Object.keys(base!)])]) {
        if (key === "title") continue;
        expect(dw[key], `debug window ${dw.label} drifted on ${key}`).toEqual(base![key]);
      }
    }
  });

  it("decorated windows show a Dev title fingerprint (popup is titleless)", () => {
    const title = (label: string) =>
      debugConf.app.windows.find((w) => w.label === label)!.title;
    expect(title("main")).toBe("LinguaRay Dev");
    expect(title("onboarding")).toBe("LinguaRay Dev");
    expect(title("input")).toContain("Dev");
    expect(title("popup")).toBe("");
  });
});

describe("tauri.noupdater.conf.json (local build)", () => {
  it("disables updater artifacts so unsigned local builds need no signing key", () => {
    const noUpdater = JSON.parse(
      readFileSync(join(__dirname, "..", "src-tauri", "tauri.noupdater.conf.json"), "utf8"),
    ) as { bundle: { createUpdaterArtifacts: boolean } };
    expect(noUpdater.bundle.createUpdaterArtifacts).toBe(false);
  });
});
