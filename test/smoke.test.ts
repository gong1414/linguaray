import { describe, it, expect } from "vitest";
import { Spinner } from "@linguaray/ui";

describe("production test harness", () => {
  it("resolves the workspace @linguaray/ui package", () => {
    // Spinner is a Solid component (a function). Importing it without error
    // proves the workspace alias + Vitest Solid environment both resolve.
    expect(typeof Spinner).toBe("function");
  });
});
