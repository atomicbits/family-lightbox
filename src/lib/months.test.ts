import { describe, expect, it } from "vitest";
import { fileName, groupByYear, monthLabel, takenLabel, yearOf } from "./months";

describe("maanden", () => {
  it("labelt een maand in het Nederlands en een map zonder maand met haar naam", () => {
    expect(monthLabel("2026/07")).toBe("juli");
    expect(monthLabel("2012/scans Peter")).toBe("scans Peter");
    expect(yearOf("2007/02")).toBe("2007");
  });

  it("groepeert per jaar, nieuwste eerst, en telt op", () => {
    const g = groupByYear([
      { month: "2026/07", photos: 10, videos: 2, previews: 12, local: 12 },
      { month: "2026/06", photos: 5, videos: 0, previews: 5, local: 5 },
      { month: "2007/02", photos: 3, videos: 0, previews: 0, local: 0 },
    ]);
    expect(g.map((x) => x.year)).toEqual(["2026", "2007"]);
    expect(g[0].photos).toBe(15);
    expect(g[0].months.map((m) => m.month)).toEqual(["2026/07", "2026/06"]);
  });

  it("toont de opnametijd leesbaar", () => {
    expect(takenLabel("2026-07-30T10:05:00+02:00")).toBe("30 juli 2026, 10:05");
    expect(takenLabel(null)).toBe("");
    expect(fileName("2026/07/EOSR/photos/2026-07-30/CO7A9123.CR3")).toBe("CO7A9123.CR3");
  });
});
