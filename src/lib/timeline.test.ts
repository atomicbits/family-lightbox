import { describe, expect, it } from "vitest";
import { buildLayout, moveAcross, rowOf, sectionsInRange } from "./timeline";

// three months: 10 items, 1 item, 6 items; 4 columns
//   header 0 | 0 1 2 3 | 4 5 6 7 | 8 9
//   header 1 | 0
//   header 2 | 0 1 2 3 | 4 5
const sections = [
  { month: "2026/05", count: 10 },
  { month: "2026/04", count: 1 },
  { month: "2026/03", count: 6 },
];
const counts = sections.map((s) => s.count);

describe("buildLayout", () => {
  it("puts a header before each month's rows", () => {
    const l = buildLayout(sections, 4);
    expect(l.rows.length).toBe(1 + 3 + 1 + 1 + 1 + 2);
    expect(l.headerRow).toEqual([0, 4, 6]);
    expect(l.rows[0]).toEqual({ kind: "header", section: 0 });
    expect(l.rows[3]).toEqual({ kind: "tiles", section: 0, row: 2 });
    expect(l.rows[5]).toEqual({ kind: "tiles", section: 1, row: 0 });
  });

  it("finds the row of a selection and the months in a range", () => {
    const l = buildLayout(sections, 4);
    expect(rowOf(l, { section: 0, index: 9 }, 4)).toBe(3);
    expect(rowOf(l, { section: 2, index: 4 }, 4)).toBe(8);
    expect(sectionsInRange(l, 2, 6)).toEqual([0, 1, 2]);
    expect(sectionsInRange(l, 7, 100)).toEqual([2]);
  });

  it("follows the column count", () => {
    expect(buildLayout(sections, 1).rows.length).toBe(3 + 17);
    expect(buildLayout(sections, 20).rows.length).toBe(6);
  });
});

describe("moveAcross", () => {
  const mv = (section: number, index: number, move: "left" | "right" | "up" | "down") =>
    moveAcross({ section, index }, move, 4, counts);

  it("moves inside a month as before", () => {
    expect(mv(0, 5, "up")).toEqual({ section: 0, index: 1 });
    expect(mv(0, 7, "down")).toEqual({ section: 0, index: 9 });
  });

  it("left and right cross the month boundary", () => {
    expect(mv(0, 9, "right")).toEqual({ section: 1, index: 0 });
    expect(mv(1, 0, "right")).toEqual({ section: 2, index: 0 });
    expect(mv(2, 0, "left")).toEqual({ section: 1, index: 0 });
    expect(mv(1, 0, "left")).toEqual({ section: 0, index: 9 });
  });

  it("down from the last row goes to the same column of the next month, clamped", () => {
    expect(mv(0, 9, "down")).toEqual({ section: 1, index: 0 });
    expect(mv(0, 8, "down")).toEqual({ section: 1, index: 0 });
    expect(mv(1, 0, "down")).toEqual({ section: 2, index: 0 });
    // a short last row in the month itself: land on its last item first
    expect(mv(2, 1, "down")).toEqual({ section: 2, index: 5 });
  });

  it("up from the first row goes to the same column of the last row above, clamped", () => {
    expect(mv(2, 2, "up")).toEqual({ section: 1, index: 0 });
    expect(mv(1, 0, "up")).toEqual({ section: 0, index: 8 });
    expect(moveAcross({ section: 2, index: 3 }, "up", 4, [8, 6])).toEqual({ section: 1, index: 5 });
  });

  it("stays at the very top and bottom, and starts at the first photo", () => {
    expect(mv(0, 0, "left")).toEqual({ section: 0, index: 0 });
    expect(mv(0, 2, "up")).toEqual({ section: 0, index: 2 });
    expect(mv(2, 5, "right")).toEqual({ section: 2, index: 5 });
    expect(mv(2, 4, "down")).toEqual({ section: 2, index: 4 });
    expect(moveAcross(null, "down", 4, counts)).toEqual({ section: 0, index: 0 });
    expect(moveAcross(null, "down", 4, [])).toBeNull();
  });
});
