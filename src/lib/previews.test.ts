import { describe, expect, it } from "vitest";
import { addToMonths, markMade, missingText, previewsLine, type MakeState } from "./previews";
import type { Item, MonthSummary } from "./api";

const base: MakeState = {
  running: false, paused: false, total: 0, done: 0, made: 0, copied: 0, failed: 0, skipped: 0, nas_failed: 0, message: null,
};

describe("previews line", () => {
  it("shows the progress with Pauzeren while making", () => {
    expect(previewsLine({ ...base, running: true, total: 85874, done: 1204 })).toEqual({
      text: "Previews maken: 1 204 / 85 874", tone: "muted", button: { label: "Pauzeren", action: "pause" },
    });
  });

  it("shows Hervatten when paused, with or without a run", () => {
    expect(previewsLine({ ...base, paused: true, total: 85874, done: 1204 })?.text).toBe("Previews gepauzeerd: 1 204 / 85 874");
    expect(previewsLine({ ...base, paused: true })).toMatchObject({ text: "Previews gepauzeerd", button: { action: "resume" } });
  });

  it("says nothing while planning or when all is done, and warns when the NAS didn't get them", () => {
    expect(previewsLine({ ...base, running: true })).toBeNull();
    expect(previewsLine({ ...base, total: 10, done: 10, made: 10 })).toBeNull();
    expect(previewsLine({ ...base, total: 10, done: 10, made: 10, nas_failed: 3 })).toMatchObject({
      text: "3 previews staan nog niet op de NAS", tone: "amber",
    });
    expect(previewsLine({ ...base, nas_failed: 1 })?.text).toBe("1 preview staat nog niet op de NAS");
    expect(previewsLine(null)).toBeNull();
  });
});

const item = (path: string, preview = false): Item => ({
  path, folder: "f", kind: "photo", taken: null, taken_from: null, source: null, size: 1, hash: "h", local: false, preview,
});

describe("new previews in the grid", () => {
  it("marks the items that got a preview, and leaves a month without them alone", () => {
    const items = [item("a"), item("b"), item("c", true)];
    const next = markMade(items, new Set(["b", "c"]));
    expect(next?.map((i) => i.preview)).toEqual([false, true, true]);
    expect(next?.[0]).toBe(items[0]);
    expect(markMade(items, new Set(["x"]))).toBeNull();
  });

  it("adds them to the month counts, never above the number of photos and videos", () => {
    const months: MonthSummary[] = [
      { month: "2019/05", photos: 3, videos: 0, previews: 1, local: 0 },
      { month: "2012/scans", photos: 1, videos: 0, previews: 0, local: 0 },
    ];
    const next = addToMonths(months, [{ path: "a", month: "2019/05" }, { path: "b", month: "2019/05" }, { path: "c", month: "2019/05" }]);
    expect(next.map((m) => m.previews)).toEqual([3, 0]);
    expect(next[1]).toBe(months[1]);
  });

  it("says how many are missing", () => {
    expect(missingText({ missing: 85874, failed: [] })).toBe("85 874 foto's en video's zonder preview");
    expect(missingText({ missing: 1, failed: [] })).toBe("1 foto of video zonder preview");
    expect(missingText({ missing: 0, failed: [] })).toBe("alle foto's en video's hebben een preview");
  });
});
