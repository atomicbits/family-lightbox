import { describe, expect, it } from "vitest";
import { listText, skippedText, undoHeart, undoList, undoStars, type Step } from "./undo";
import { NO_MARKS } from "./marks";

const r = (stars: number, by = "peter") => ({ stars, by, at: "" });

describe("undo", () => {
  it("puts stars back per photo, leaving alone what someone changed since", () => {
    const step: Extract<Step, { kind: "stars" }> = { kind: "stars", text: "", after: 3, before: { a: 0, b: 5, c: 1, d: 3 } };
    const s = { ...NO_MARKS, ratings: { a: r(3), b: r(3), c: r(4, "lynn"), d: r(3) } };
    const p = undoStars(step, s);
    expect([...p.writes.entries()]).toEqual([[0, ["a"]], [5, ["b"]]]);
    expect(p.by).toEqual(["lynn"]);
    expect(p.skipped).toBe(1);
  });

  it("puts my heart back where it is still as the step left it", () => {
    const step: Extract<Step, { kind: "heart" }> = { kind: "heart", text: "", after: true, before: { a: false, b: true, c: false } };
    const s = { ...NO_MARKS, hearts: { a: { peter: "t" }, b: { peter: "t" } } };
    const p = undoHeart(step, s, "peter");
    expect([...p.writes.entries()]).toEqual([[false, ["a"]]]);
    expect(p.skipped).toBe(1); // c lost its heart meanwhile
  });

  it("takes added photos out again, unless they were removed meanwhile", () => {
    const done = { id: "2026/ski", name: "Ski", op: "add" as const, photos: [{ photo: "a", path: null }, { photo: "b", path: null }] };
    expect(undoList(done, (p) => p === "a")).toEqual({ photos: [{ photo: "a", path: null }], skipped: 1 });
    const removed = { ...done, op: "remove" as const };
    expect(undoList(removed, (p) => p === "a").photos).toEqual([{ photo: "b", path: null }]);
  });

  it("says what was left alone and what a list change did", () => {
    expect(skippedText(0, [])).toBe("");
    expect(skippedText(1, ["lynn"])).toBe("1 foto intussen door lynn gewijzigd, niet teruggezet");
    expect(skippedText(2, [])).toBe("2 foto's intussen gewijzigd, niet teruggezet");
    const d = { id: "x", name: "Ski", op: "add" as const, photos: [{ photo: "a", path: null }] };
    expect(listText(d)).toBe("1 foto toegevoegd aan Ski");
    expect(listText({ ...d, op: "remove", photos: [d.photos[0], d.photos[0]] })).toBe("2 foto's uit Ski gehaald");
  });
});
