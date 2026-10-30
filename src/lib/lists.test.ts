import { describe, expect, it } from "vitest";
import { addedText, changeBlocked, defaultYear, groupLists, manyChange, type ListInfo, type ListsState, type Member } from "./lists";
import type { Item } from "./api";

const list = (id: string, archived = false): ListInfo => ({
  id, name: id.split("/")[1], year: Number(id.slice(0, 4)), archived, count: 0,
  created_by: "lynn", created_at: "", updated_by: "", updated_at: "",
});
const state = (source: ListsState["source"], problem: string | null = null): ListsState => ({ source, lists: [], ignored: 0, problem });

describe("lists", () => {
  it("groups per year as they come, newest first, archived only when asked", () => {
    const lists = [list("2026/b"), list("2026/ski", true), list("2025/zee")];
    expect(groupLists(lists, false).map((g) => [g.year, g.lists.map((l) => l.id)])).toEqual([[2026, ["2026/b"]], [2025, ["2025/zee"]]]);
    expect(groupLists(lists, true)[0].lists.length).toBe(2);
  });

  it("can change only from the NAS and without a problem", () => {
    expect(changeBlocked(state("nas"))).toBeNull();
    expect(changeBlocked(state("local"))).toBe("lijsten wijzigen kan alleen met de NAS");
    expect(changeBlocked(state("none"))).toBe("lijsten wijzigen kan alleen met de NAS");
    expect(changeBlocked(state("nas", "devices.json klopt niet"))).toBe("devices.json klopt niet");
    expect(changeBlocked(null)).toBe("lijsten laden…");
  });

  it("takes the year of the month on screen for a new list", () => {
    const now = new Date(2026, 9, 29);
    expect(defaultYear("2019/05", now)).toBe(2019);
    expect(defaultYear("2012/scans Peter", now)).toBe(2012);
    expect(defaultYear("0000/zonder-datum", now)).toBe(2026);
    expect(defaultYear(null, now)).toBe(2026);
  });

  it("says who added a photo and when", () => {
    expect(addedText({ photo: "h", path: null, added_by: "lynn", added_at: new Date(2026, 9, 20, 19, 42, 30).toISOString() })).toMatch(/^toegevoegd door lynn, 20 okt/);
    expect(addedText({ photo: "h", path: null, added_by: "peter", added_at: "gisteren" })).toBe("toegevoegd door peter, gisteren");
  });
});

describe("several photos at once", () => {
  const item = (hash: string): Item => ({ path: `p/${hash}`, folder: "f", kind: "photo", taken: null, taken_from: null, source: null, size: 1, hash, local: false, preview: true });
  const member = (photo: string): Member => ({ photo, path: null, added_by: "lynn", added_at: "" });
  it("adds the missing ones, or removes all when all are in", () => {
    const members = new Map([["a", member("a")]]);
    expect(manyChange([item("a"), item("b"), item("b")], members)).toEqual({ op: "add", items: [item("b")] });
    expect(manyChange([item("a")], members)).toEqual({ op: "remove", items: [item("a")] });
  });
});
