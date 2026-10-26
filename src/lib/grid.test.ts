import { describe, expect, it } from "vitest";
import { columnsFor, moveInGrid } from "./grid";

// 10 items in 4 columns:   0 1 2 3 / 4 5 6 7 / 8 9
describe("moveInGrid", () => {
  it("moves by one left and right, clamped at the ends", () => {
    expect(moveInGrid(5, "left", 4, 10)).toBe(4);
    expect(moveInGrid(5, "right", 4, 10)).toBe(6);
    expect(moveInGrid(0, "left", 4, 10)).toBe(0);
    expect(moveInGrid(9, "right", 4, 10)).toBe(9);
  });

  it("moves by a row up and down", () => {
    expect(moveInGrid(5, "up", 4, 10)).toBe(1);
    expect(moveInGrid(1, "down", 4, 10)).toBe(5);
    expect(moveInGrid(5, "down", 4, 10)).toBe(9);
  });

  it("stays on the first row going up and on the last row going down", () => {
    expect(moveInGrid(2, "up", 4, 10)).toBe(2);
    expect(moveInGrid(8, "down", 4, 10)).toBe(8);
  });

  it("lands on the last item when the row below is shorter", () => {
    expect(moveInGrid(7, "down", 4, 10)).toBe(9);
    expect(moveInGrid(6, "down", 4, 10)).toBe(9);
  });

  it("selects the first item when nothing is selected, and nothing in an empty month", () => {
    expect(moveInGrid(null, "down", 4, 10)).toBe(0);
    expect(moveInGrid(null, "left", 4, 10)).toBe(0);
    expect(moveInGrid(3, "down", 4, 0)).toBeNull();
  });
});

describe("columnsFor", () => {
  // tile 176, gap 8: n tiles need n*176 + (n-1)*8, plus 2*8 padding
  it("fits as many tiles as the width allows, never more", () => {
    expect(columnsFor(176 + 16, 176, 8)).toBe(1);
    expect(columnsFor(2 * 176 + 8 + 16, 176, 8)).toBe(2);
    expect(columnsFor(2 * 176 + 8 + 16 - 1, 176, 8)).toBe(1);
    expect(columnsFor(10 * 176 + 9 * 8 + 16, 176, 8)).toBe(10);
  });

  it("is at least 1, also for a very narrow or not yet laid out box", () => {
    expect(columnsFor(100, 176, 8)).toBe(1);
    expect(columnsFor(0, 176, 8)).toBe(1);
  });
});
