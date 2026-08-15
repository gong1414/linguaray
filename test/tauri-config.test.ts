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
