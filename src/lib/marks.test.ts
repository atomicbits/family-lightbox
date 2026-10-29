import { describe, expect, it } from "vitest";
import { changeText, heartAfterClick, heartState, heartView, markSize, marksBlocked, marksLines, sharedStars, starsAfterClick, starsText, NO_MARKS } from "./marks";

describe("marks on a tile", () => {
  it("scales with the tile within readable bounds", () => {
    expect(markSize(96)).toBe(10);
    expect(markSize(200)).toBe(15);
    expect(markSize(480)).toBe(18);
  });

  it("shows only the filled stars", () => {
    expect(starsText(4)).toBe("★★★★");
    expect(starsText(undefined)).toBe("");
    expect(starsText(9)).toBe("★★★★★");
  });

  it("knows whose hearts are on a photo", () => {
    expect(heartView(undefined, "peter")).toBeNull();
    expect(heartView({ lynn: "t" }, "peter")).toEqual({ mine: false, count: 1, who: ["lynn"] });
    expect(heartView({ peter: "t", lynn: "t" }, "peter")).toEqual({ mine: true, count: 2, who: ["lynn", "peter"] });
  });

  it("says who rated and hearted it", () => {
    expect(marksLines({ stars: 4, by: "lynn", at: "" }, { peter: "t", lynn: "t" })).toEqual(["4 sterren (lynn)", "hartje van lynn en peter"]);
    expect(marksLines({ stars: 1, by: "peter", at: "" }, undefined)).toEqual(["1 ster (peter)"]);
    expect(marksLines(undefined, { a: "", b: "", c: "" })).toEqual(["hartje van a, b en c"]);
  });
});

describe("setting marks for a selection", () => {
  const ratings = { a: { stars: 3, by: "lynn", at: "" }, b: { stars: 3, by: "peter", at: "" }, c: { stars: 5, by: "lynn", at: "" } };
  const hearts = { a: { peter: "t" }, b: { lynn: "t" } };
  it("shows the shared rating, or none when they differ", () => {
    expect(sharedStars(["a", "b"], ratings)).toBe(3);
    expect(sharedStars(["a", "c"], ratings)).toBeNull();
    expect(sharedStars(["x", "y"], ratings)).toBe(0);
    expect(sharedStars(["a", "x"], ratings)).toBeNull();
  });
  it("knows whether my heart is on all, some or none", () => {
    expect(heartState(["a"], hearts, "peter")).toBe("all");
    expect(heartState(["a", "b"], hearts, "peter")).toBe("some");
    expect(heartState(["b"], hearts, "peter")).toBe("none");
    expect(heartState(["a"], hearts, null)).toBe("none");
  });
  it("a click sets, or removes what all of them have", () => {
    expect(starsAfterClick(3, 3)).toBe(0);
    expect(starsAfterClick(null, 3)).toBe(3);
    expect(starsAfterClick(0, 5)).toBe(5);
    expect([heartAfterClick("all"), heartAfterClick("some"), heartAfterClick("none")]).toEqual([false, true, true]);
  });
  it("is blocked without the NAS or an owner, and says what happened", () => {
    expect(marksBlocked(NO_MARKS, "peter")).toBe("sterren en hartjes wijzigen kan alleen met de NAS");
    expect(marksBlocked({ ...NO_MARKS, source: "nas" }, null)).toBe("dit toestel heeft nog geen eigenaar");
    expect(marksBlocked({ ...NO_MARKS, source: "nas" }, "peter")).toBeNull();
    expect(changeText(12, { stars: 3 })).toBe("12 foto's: 3 sterren");
    expect(changeText(1, { stars: 0 })).toBe("1 foto: sterren weg");
    expect(changeText(3, { heart: true })).toBe("hartje op 3 foto's");
    expect(changeText(1, { heart: false })).toBe("hartje weg bij 1 foto");
  });
});
