import { describe, expect, it } from "vitest";
import { countTitle, shownCount, visibleItems, withHidden } from "./hidden";
import type { Item, MonthSummary } from "./api";

const m = (month: string, photos: number): MonthSummary => ({ month, photos, videos: 1, previews: 0, local: 0 });
const item = (hash: string): Item => ({ path: hash, folder: "f", kind: "photo", taken: null, taken_from: null, source: null, size: 1, hash, local: false, preview: true });

describe("hidden photos", () => {
  it("count what a month shows", () => {
    const months = [m("2026/07", 9), m("2026/06", 4)];
    const w = withHidden(months, { "2026/07": 3 });
    expect(w.map(shownCount)).toEqual([7, 5]);
    expect(w[1]).toBe(months[1]);
    expect(withHidden(w, { "2026/07": 3 })[0]).toBe(w[0]);
  });

  it("leave hidden items out, keeping the same array when none is hidden", () => {
    const items = [item("a"), item("b"), item("c")];
    expect(visibleItems(items, { b: {} }).map((i) => i.hash)).toEqual(["a", "c"]);
    expect(visibleItems(items, {})).toBe(items);
  });

  it("say how many are hidden", () => {
    expect(countTitle(312, 20)).toBe("312 foto's en video's, 20 verborgen");
    expect(countTitle(1, 0)).toBe("1 foto of video");
  });
});
