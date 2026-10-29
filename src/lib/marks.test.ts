import { describe, expect, it } from "vitest";
import { heartView, markSize, marksLines, starsText } from "./marks";

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
