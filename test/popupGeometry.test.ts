import { describe, it, expect } from "vitest";

const LOGICAL_SIZES = {
  loading: { w: 200, h: 40 },
  single: { w: 400, h: 300 },
  multi: { w: 600, h: 400 },
  error: { w: 400, h: 300 },
};

describe("popup geometry contract (frontend mirror)", () => {
  it("loading is 200x40 logical", () => {
    expect(LOGICAL_SIZES.loading).toEqual({ w: 200, h: 40 });
  });
  it("single is 400x300 logical", () => {
    expect(LOGICAL_SIZES.single).toEqual({ w: 400, h: 300 });
  });
  it("multi is 600x400 logical", () => {
    expect(LOGICAL_SIZES.multi).toEqual({ w: 600, h: 400 });
  });
  it("error matches single (400x300)", () => {
    expect(LOGICAL_SIZES.error).toEqual(LOGICAL_SIZES.single);
  });
});
