import { describe, expect, it } from "vitest";
import { arrange, buildStacks, folded, lastOfStack, stackActions, stackColor, stackDelta, type Member, type View } from "./stacks";

const m = (hash: string, taken: string, month = "2026/07"): Member => ({
  path: `${month}/${hash}`, folder: "f", kind: "photo", taken, taken_from: null, source: null, size: 1, hash, local: false, preview: true, month,
});
const noMarks = { ratings: {}, hearts: {} };

// Peter's camera a1–a4, Lynn's iPhone l1–l2 in between; a1–a4 are a stack
const a1 = m("a1", "10:00"), l1 = m("l1", "10:01"), a2 = m("a2", "10:02"), l2 = m("l2", "10:03"), a3 = m("a3", "10:04"), a4 = m("a4", "10:05");
const month = [a1, l1, a2, l2, a3, a4];
const ids = { a1: "s", a2: "s", a3: "s", a4: "s" };
const view = (over: Partial<View> = {}): View => ({
  stacks: buildStacks([a4, a2, a1, a3], ids), unfolded: new Set(), marks: noMarks, isHidden: () => false, ...over,
});
const hashes = (items: { hash: string }[]) => items.map((i) => i.hash);

describe("stacks in the grid", () => {
  it("folds a stack to its first photo, at its place, leaving the other device's photos", () => {
    expect(hashes(arrange("2026/07", month, ids, view()))).toEqual(["a1", "l1", "l2"]);
  });

  it("unfolds it with all its photos together, in the order taken", () => {
    expect(hashes(arrange("2026/07", month, ids, view({ unfolded: new Set(["s"]) })))).toEqual(["a1", "a2", "a3", "a4", "l1", "l2"]);
  });

  it("shows hearts and 4–5 stars, else the most stars, else the first", () => {
    const r = (stars: number) => ({ stars, by: "x", at: "" });
    const members = [a1, a2, a3, a4];
    expect(hashes(folded(members, { ratings: { a3: r(5) }, hearts: { a2: { lynn: "" } } }))).toEqual(["a2", "a3"]);
    expect(hashes(folded(members, { ratings: { a2: r(2), a3: r(3), a4: r(3) }, hearts: {} }))).toEqual(["a3", "a4"]);
    expect(hashes(folded(members, noMarks))).toEqual(["a1"]);
  });

  it("stands in the month of its first visible photo, with its photos from the next month", () => {
    const b1 = m("b1", "2026-07-31T23:58", "2026/07"), b2 = m("b2", "2026-08-01T00:01", "2026/08");
    const v: View = { ...view(), stacks: buildStacks([b1, b2], { b1: "t", b2: "t" }), unfolded: new Set(["t"]) };
    const ids2 = { b1: "t", b2: "t" };
    expect(hashes(arrange("2026/07", [b1], ids2, v))).toEqual(["b1", "b2"]);
    expect(hashes(arrange("2026/08", [b2], ids2, v))).toEqual([]);
    expect(stackDelta("2026/07", v)).toEqual({ delta: 1, stacks: 1 });
    expect(stackDelta("2026/08", v)).toEqual({ delta: -1, stacks: 0 });
  });

  it("leaves hidden photos out, and moves the stack to its first visible one", () => {
    const v = view({ isHidden: (x) => x.hash === "a1" });
    const items = month.filter((i) => i.hash !== "a1");
    expect(hashes(arrange("2026/07", items, ids, v))).toEqual(["l1", "a2", "l2"]);
  });

  it("counts the tiles a month loses to its folded stacks", () => {
    expect(stackDelta("2026/07", view())).toEqual({ delta: -3, stacks: 1 });
    expect(stackDelta("2026/07", view({ unfolded: new Set(["s"]) }))).toEqual({ delta: 0, stacks: 1 });
  });
});

describe("the looks of a stack", () => {
  it("has the same color for the same stack", () => {
    expect(stackColor("mbp-peter-1")).toEqual(stackColor("mbp-peter-1"));
    const colors = new Set(["a", "b", "c", "d", "e", "f"].map((x) => stackColor(`mbp-peter-${x}`).border));
    expect(colors.size).toBeGreaterThan(2);
    expect(stackColor("mbp-peter-1", "groen").name).toBe("groen");
    expect(stackColor("mbp-peter-1", "rood")).toEqual(stackColor("mbp-peter-1"));
  });

  it("puts the symbol on the last shown tile of the stack", () => {
    const tiles = [{ hash: "a1" }, { hash: "a2" }, { hash: "l1" }];
    expect([0, 1, 2].map((i) => lastOfStack(tiles, i, { a1: "s", a2: "s" }))).toEqual([false, true, false]);
  });
});

describe("the menu for stacks", () => {
  const ids = { a1: "s", a2: "s", b1: "t" };
  it("makes a stack from two or more photos outside any stack", () => {
    expect(stackActions(["x", "y"], ids)).toMatchObject({ make: true, addTo: null, takeOut: [], dissolve: null });
    expect(stackActions(["x"], ids).make).toBe(false);
  });
  it("adds to the one stack selected, never with two", () => {
    expect(stackActions(["a1", "x", "y"], ids)).toMatchObject({ make: false, addTo: { stack: "s", photos: ["x", "y"] }, takeOut: ["a1"], dissolve: "s" });
    expect(stackActions(["a1", "b1", "x"], ids)).toMatchObject({ addTo: null, takeOut: ["a1", "b1"], dissolve: null });
  });
});
