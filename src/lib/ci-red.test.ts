import { expect, it } from "vitest";

// Throwaway (F004/S02): proves that a failing test turns CI red. Never merged.
it("fails on purpose", () => {
  expect(1 + 1).toBe(3);
});
