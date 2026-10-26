import { describe, expect, it } from "vitest";
import { TILE_DEFAULT, TILE_MAX, TILE_MIN, clampTile, columnsFor, loadTile, moveInGrid, rowWidth, saveTile } from "./grid";

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

describe("tile size", () => {
  it("clamps to the slider's range and falls back to the default", () => {
    expect(clampTile(50)).toBe(TILE_MIN);
    expect(clampTile(1000)).toBe(TILE_MAX);
    expect(clampTile(200.4)).toBe(200);
    expect(clampTile(Number.NaN)).toBe(TILE_DEFAULT);
  });

  it("remembers the size, and survives storage that is missing or broken", () => {
    const store = new Map<string, string>();
    globalThis.localStorage = {
      getItem: (k: string) => store.get(k) ?? null,
      setItem: (k: string, v: string) => void store.set(k, v),
    } as Storage;
    expect(loadTile()).toBe(TILE_DEFAULT);
    saveTile(240);
    expect(loadTile()).toBe(240);
    store.set("familylightbox.tile", "veel");
    expect(loadTile()).toBe(TILE_DEFAULT);
    globalThis.localStorage = { getItem: () => { throw new Error("blocked"); }, setItem: () => { throw new Error("blocked"); } } as unknown as Storage;
    expect(loadTile()).toBe(TILE_DEFAULT);
    expect(() => saveTile(200)).not.toThrow();
  });

  it("computes the width of a full row for centring", () => {
    expect(rowWidth(1, 176, 8)).toBe(176);
    expect(rowWidth(4, 176, 8)).toBe(4 * 176 + 3 * 8);
  });
});
