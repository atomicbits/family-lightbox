import { describe, expect, it } from "vitest";
import { addedText, changeBlocked, defaultYear, groupLists, manyChange, type ListInfo, type ListsState, type Member, menuDirect, menuSearch } from "./lists";
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
    expect(changeBlocked(state("local"))).toBeNull();
    expect(changeBlocked(state("none"))).toBe("nog geen lijsten op deze Mac: verbind eerst één keer met de NAS");
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

describe("lists in the photo menu (F003/S05)", () => {
  const l = (name: string, year: number, updated: string) => ({ id: `${year}/${name}`, name, year, archived: false, count: 1, created_by: "x", created_at: "", updated_by: "x", updated_at: updated });
  const few = [l("a", 2026, "2026-11-01"), l("b", 2025, "2026-10-01")];
  const many = Array.from({ length: 10 }, (_, i) => l(`lijst ${i}`, 2026 - (i % 3), `2026-10-${String(10 + i).padStart(2, "0")}`));
  it("shows a few lists all, many by the most recent five", () => {
    expect(menuDirect(few)).toEqual({ direct: few, more: false });
    const m = menuDirect(many);
    expect(m.more).toBe(true);
    expect(m.direct.map((x) => x.name)).toEqual(["lijst 9", "lijst 8", "lijst 7", "lijst 6", "lijst 5"]);
  });
  it("finds a list by name or year, per year", () => {
    const sorted = [...many].sort((a, b) => b.year - a.year || a.name.localeCompare(b.name));
    expect(menuSearch(sorted, "lijst 4").flatMap((g) => g.lists.map((x) => x.name))).toEqual(["lijst 4"]);
    expect(menuSearch(sorted, "2024").map((g) => g.year)).toEqual([2024]);
    expect(menuSearch(sorted, "").length).toBe(3);
  });
});
