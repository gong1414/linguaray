import { describe, expect, it } from "vitest";
import {
  applyCheck,
  applyInstallDone,
  applyProgress,
  isUpdateCheck,
  isUpdaterProgress,
  type UpdaterPhase,
} from "../src/features/settings/updater-types";

const available = {
  state: "available" as const,
  current: "0.1.0",
  next: "0.2.0",
  notes: "fixes",
};

describe("isUpdateCheck", () => {
  it("accepts all three wire variants", () => {
    expect(isUpdateCheck({ state: "up_to_date", version: "0.1.0" })).toBe(true);
    expect(isUpdateCheck(available)).toBe(true);
    expect(isUpdateCheck({ state: "error", message: "offline" })).toBe(true);
  });

  it("rejects malformed payloads", () => {
    expect(isUpdateCheck(null)).toBe(false);
    expect(isUpdateCheck("up_to_date")).toBe(false);
    expect(isUpdateCheck({ state: "available", current: 1, next: "0.2", notes: "" })).toBe(false);
    expect(isUpdateCheck({ state: "unknown" })).toBe(false);
  });
});

describe("isUpdaterProgress", () => {
  it("accepts chunk and finished payloads", () => {
    expect(isUpdaterProgress({ downloaded: 10, total: null, bucket: 0 })).toBe(true);
    expect(isUpdaterProgress({ downloaded: 10, total: 100, bucket: 10 })).toBe(true);
    expect(isUpdaterProgress({ finished: true })).toBe(true);
    expect(isUpdaterProgress({ finished: "yes" })).toBe(false);
    expect(isUpdaterProgress({ downloaded: "10" })).toBe(false);
  });
});

describe("applyCheck", () => {
  it("maps all three results", () => {
    expect(applyCheck({ kind: "checking" }, { state: "up_to_date", version: "0.1.0" })).toEqual({
      kind: "upToDate",
      version: "0.1.0",
    });
    expect(applyCheck({ kind: "checking" }, available)).toEqual({ kind: "available", update: available });
    expect(applyCheck({ kind: "checking" }, { state: "error", message: "offline" })).toEqual({
      kind: "error",
      message: "offline",
    });
  });

  it("does not interrupt an in-flight install with a late check response", () => {
    const downloading: UpdaterPhase = {
      kind: "downloading",
      update: available,
      percent: 40,
      downloaded: 4000,
    };
    expect(applyCheck(downloading, { state: "up_to_date", version: "0.9" })).toBe(downloading);
    const installing: UpdaterPhase = { kind: "installing", update: available };
    expect(applyCheck(installing, available)).toBe(installing);
  });
});

describe("applyProgress", () => {
  const downloading: UpdaterPhase = { kind: "downloading", update: available, percent: null, downloaded: 0 };

  it("computes percent from a known total and clamps at 100", () => {
    const half = applyProgress(downloading, { downloaded: 5_000_000, total: 10_000_000, bucket: 50 });
    expect(half).toEqual({
      kind: "downloading",
      update: available,
      percent: 50,
      downloaded: 5_000_000,
    });
    const done = applyProgress(downloading, { downloaded: 10_000_000, total: 10_000_000, bucket: 100 });
    expect(done.kind === "downloading" && done.percent).toBe(100);
  });

  it("keeps percent null when the total is unknown", () => {
    const got = applyProgress(downloading, { downloaded: 3_000_000, total: null, bucket: 2 });
    expect(got.kind === "downloading" && got.percent).toBeNull();
  });

  it("switches to installing on the finished event", () => {
    expect(applyProgress(downloading, { finished: true })).toEqual({
      kind: "installing",
      update: available,
    });
  });

  it("ignores progress outside the downloading phase", () => {
    expect(applyProgress({ kind: "available", update: available }, { finished: true })).toEqual({
      kind: "available",
      update: available,
    });
  });
});

describe("applyInstallDone", () => {
  it("lands on readyToRelaunch when the install resolved", () => {
    expect(applyInstallDone({ kind: "installing", update: available }, available)).toEqual({
      kind: "readyToRelaunch",
      update: available,
    });
  });

  it("handles the release vanishing between check and install", () => {
    expect(
      applyInstallDone({ kind: "installing", update: available }, { state: "up_to_date", version: "0.1.0" }),
    ).toEqual({ kind: "upToDate", version: "0.1.0" });
  });

  it("ignores results outside the installing phase", () => {
    expect(applyInstallDone({ kind: "upToDate", version: "0.1.0" }, available)).toEqual({
      kind: "upToDate",
      version: "0.1.0",
    });
  });
});
