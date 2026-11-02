import { describe, expect, it } from "vitest";
import { fraction } from "./progress";

describe("a progress bar", () => {
  it("fills by what is done, and moves when the total isn't known", () => {
    expect(fraction(5, 10)).toBe(0.5);
    expect(fraction(12, 10)).toBe(1);
    expect(fraction(0, 0)).toBeNull();
    expect(fraction(undefined, 10)).toBeNull();
    expect(fraction(3, undefined)).toBeNull();
  });
});
