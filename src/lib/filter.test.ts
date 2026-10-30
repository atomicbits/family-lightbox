import { describe, expect, it } from "vitest";
import { NO_FILTER, filterText, isSet, marksSet, matches, readFilter, toCore, type FilterState } from "./filter";
import { NO_MARKS } from "./marks";

const r = (stars: number) => ({ stars, by: "x", at: "" });
const m = { ratings: { a: r(5), b: r(3), c: r(1) }, hearts: { a: { lynn: "" }, d: { peter: "" } } };
const f = (over: Partial<FilterState>): FilterState => ({ ...NO_FILTER, ...over });
const sorted = (s: ReturnType<typeof marksSet>) => s && { ...s, hashes: [...s.hashes].sort() };

describe("the filter", () => {
  it("passes stars by ≥, = and ≤, and without stars", () => {
    expect(sorted(marksSet(f({ stars: { op: "gte", n: 3 } }), m))).toEqual({ hashes: ["a", "b"], exclude: false });
    expect(sorted(marksSet(f({ stars: { op: "eq", n: 1 } }), m))).toEqual({ hashes: ["c"], exclude: false });
    expect(sorted(marksSet(f({ stars: { op: "lte", n: 3 } }), m))).toEqual({ hashes: ["b", "c"], exclude: false });
    expect(sorted(marksSet(f({ stars: { op: "none", n: 0 } }), m))).toEqual({ hashes: ["a", "b", "c"], exclude: true });
  });

  it("passes hearts of anyone or of chosen people, and stars and hearts together", () => {
    expect(sorted(marksSet(f({ hearts: { anyone: true, people: [] } }), m))?.hashes).toEqual(["a", "d"]);
    expect(marksSet(f({ hearts: { anyone: false, people: ["peter"] } }), m)?.hashes).toEqual(["d"]);
    expect(marksSet(f({ stars: { op: "gte", n: 4 }, hearts: { anyone: true, people: [] } }), m)?.hashes).toEqual(["a"]);
    expect(marksSet(f({ stars: { op: "none", n: 0 }, hearts: { anyone: true, people: [] } }), m)?.hashes).toEqual(["d"]);
    expect(marksSet(NO_FILTER, m)).toBeNull();
  });

  it("matches a list's photos as the core does", () => {
    const c = toCore(f({ kinds: ["live"], stars: { op: "gte", n: 4 } }), { ...NO_MARKS, ...m }, new Set());
    expect(matches({ hash: "a", kind: "video", live: "x.HEIC" }, c)).toBe(true);
    expect(matches({ hash: "a", kind: "video", live: null }, c)).toBe(false);
    expect(matches({ hash: "b", kind: "video", live: "x.HEIC" }, c)).toBe(false);
  });

  it("says what is set, and reads a kept filter back safely", () => {
    expect(isSet(NO_FILTER)).toBe(false);
    expect(filterText(f({ stars: { op: "gte", n: 4 }, hearts: { anyone: false, people: ["lynn"] }, kinds: ["photo", "live"] }))).toBe("★ ≥ 4 · ♥ lynn · foto, Live Photo");
    expect(readFilter(null)).toEqual(NO_FILTER);
    expect(readFilter({ stars: { op: "x", n: 9 }, kinds: ["photo", "bad"] })).toEqual({ ...NO_FILTER, kinds: ["photo"] });
  });
});
