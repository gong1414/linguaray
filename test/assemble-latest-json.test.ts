import { describe, expect, it } from "vitest";
import { mkdtempSync, mkdirSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { assembleLatestJson, collectArtifacts } from "../scripts/assemble-latest-json.mjs";

const base = {
  tag: "v0.2.0",
  repo: "gong1414/linguaray",
  pubDate: "2026-08-14T12:00:00Z",
};

const goodFiles = new Map([
  ["LinguaRay_0.2.0_aarch64.app.tar.gz", "sig-arm64=="],
  ["LinguaRay_0.2.0_x64.app.tar.gz", "sig-intel=="],
  ["LinguaRay_0.2.0_x64-setup.exe", "sig-win=="],
  // Download-only assets that must NOT interfere with payload matching.
  ["LinguaRay_0.2.0_aarch64.dmg", null],
  ["LinguaRay_0.2.0_x64.dmg", null],
  ["LinguaRay_0.2.0_x64_en-US.msi", null],
]);

describe("assembleLatestJson", () => {
  it("maps all three platforms with embedded signatures and tag URLs", () => {
    const manifest = assembleLatestJson({ ...base, files: goodFiles });
    expect(manifest.version).toBe("0.2.0");
    expect(manifest.pub_date).toBe("2026-08-14T12:00:00Z");
    expect(manifest.platforms).toEqual({
      "darwin-aarch64": {
        signature: "sig-arm64==",
        url: "https://github.com/gong1414/linguaray/releases/download/v0.2.0/LinguaRay_0.2.0_aarch64.app.tar.gz",
      },
      "darwin-x86_64": {
        signature: "sig-intel==",
        url: "https://github.com/gong1414/linguaray/releases/download/v0.2.0/LinguaRay_0.2.0_x64.app.tar.gz",
      },
      "windows-x86_64": {
        signature: "sig-win==",
        url: "https://github.com/gong1414/linguaray/releases/download/v0.2.0/LinguaRay_0.2.0_x64-setup.exe",
      },
    });
  });

  it("keeps prerelease suffixes in the version", () => {
    const manifest = assembleLatestJson({
      ...base,
      tag: "v0.3.0-beta.1",
      files: goodFiles,
    });
    expect(manifest.version).toBe("0.3.0-beta.1");
  });

  it("fails closed when a platform payload is missing", () => {
    const files = new Map(goodFiles);
    files.delete("LinguaRay_0.2.0_x64-setup.exe");
    expect(() => assembleLatestJson({ ...base, files })).toThrow(/windows-x86_64/);
  });

  it("fails closed when a payload has no .sig", () => {
    const files = new Map(goodFiles);
    files.set("LinguaRay_0.2.0_x64.app.tar.gz", null);
    expect(() => assembleLatestJson({ ...base, files })).toThrow(/empty signature/);
  });

  it("fails closed on ambiguous payloads for one platform", () => {
    const files = new Map(goodFiles);
    files.set("Other_0.2.0_x64-setup.exe", "sig2==");
    expect(() => assembleLatestJson({ ...base, files })).toThrow(/ambiguous.*windows-x86_64/);
  });

  it("rejects tags that are not versions", () => {
    expect(() => assembleLatestJson({ ...base, tag: "nightly", files: goodFiles })).toThrow(
      /does not look like/,
    );
  });
});

describe("collectArtifacts", () => {
  it("walks nested bundle dirs (download-artifact preserves dmg/macos/msi/nsis layout)", () => {
    const dir = mkdtempSync(join(tmpdir(), "r5-artifacts-"));
    // Mirror the real merged layout: one subdir per bundle format.
    mkdirSync(join(dir, "macos"));
    mkdirSync(join(dir, "nsis"));
    writeFileSync(join(dir, "macos", "LinguaRay_0.2.0_aarch64.app.tar.gz"), "payload");
    writeFileSync(join(dir, "macos", "LinguaRay_0.2.0_aarch64.app.tar.gz.sig"), "sig-arm==");
    writeFileSync(join(dir, "nsis", "LinguaRay_0.2.0_x64-setup.exe"), "payload");
    // No .sig for the exe — must surface as null, not throw.

    const files = collectArtifacts(dir);
    expect(files.get("LinguaRay_0.2.0_aarch64.app.tar.gz")).toBe("sig-arm==");
    expect(files.get("LinguaRay_0.2.0_x64-setup.exe")).toBeNull();

    // And the full pipeline works on the nested layout.
    writeFileSync(join(dir, "macos", "LinguaRay_0.2.0_x64.app.tar.gz"), "p");
    writeFileSync(join(dir, "macos", "LinguaRay_0.2.0_x64.app.tar.gz.sig"), "sig-intel==");
    writeFileSync(join(dir, "nsis", "LinguaRay_0.2.0_x64-setup.exe.sig"), "sig-win==");
    const manifest = assembleLatestJson({
      files: collectArtifacts(dir),
      tag: "v0.2.0",
      repo: "gong1414/linguaray",
      pubDate: "2026-08-14T12:00:00Z",
    });
    expect(Object.keys(manifest.platforms).sort()).toEqual([
      "darwin-aarch64",
      "darwin-x86_64",
      "windows-x86_64",
    ]);
  });
});
