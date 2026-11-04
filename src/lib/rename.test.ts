import { describe, expect, it } from "vitest";
import { renameSummary, toggleSource, type RenamePlan } from "./rename";

const plan: RenamePlan = { from: ["GoProH7", "Hero7"], to: "GoPro-Hero7", months: [], files: 209, sidecars: 86, previews: 183, bytes: 75.0e9, drive_bytes: 74.3e9, duplicates: 0, conflicts: 0 };

describe("renaming sources (F022/S05)", () => {
  it("sums up the plan", () => {
    const months = Array.from({ length: 4 }, (_, i) => ({ month: `2020/0${i + 1}`, moves: [], duplicates: [] as [string, string][], conflicts: 0 }));
    expect(renameSummary({ ...plan, months })).toBe("209 bestanden in 4 maanden (75,0 GB), 86 sidecars, 183 previews · Drive laadt 74,3 GB opnieuw op");
    expect(renameSummary({ ...plan, files: 0 })).toBe("niets te hernoemen");
  });
  it("ticks sources", () => {
    expect(toggleSource(toggleSource([], "Hero7"), "GoProH7")).toEqual(["Hero7", "GoProH7"]);
    expect(toggleSource(["Hero7"], "Hero7")).toEqual([]);
  });
});

import { renamingText } from "./renamer";
describe("a rename running (F022)", () => {
  it("is named in the footer", () => {
    expect(renamingText([2, 4])).toBe("hernoemen: maand 2 van 4");
    expect(renamingText([0, 0])).toBe("hernoemen…");
  });
});
