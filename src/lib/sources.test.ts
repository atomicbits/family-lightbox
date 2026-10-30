import { describe, expect, it } from "vitest";
import { NO_IMPORT, baseName, layoutText, patterns, withSource } from "./sources";

describe("import sources (F018/S03)", () => {
  it("says how a source is laid out", () => {
    expect(layoutText({ split: false, day: true })).toBe("met dagmap");
    expect(layoutText({ split: true, day: true })).toBe("foto's en video's apart, met dagmap");
    expect(layoutText({ split: false, day: false })).toBe("zonder dagmap");
  });
  it("keeps only settings that differ from the default", () => {
    const a = withSource(NO_IMPORT, "Hero13", { ignore: true });
    expect(a.sources.Hero13).toMatchObject({ ignore: true, name: "" });
    expect(withSource(a, "Hero13", { ignore: false }).sources).toEqual({});
  });
  it("reads names and exclusions", () => {
    expect(baseName("/Users/x/Media Offload Folder/")).toBe("Media Offload Folder");
    expect(patterns(" *-mask.png, ,*.psd ")).toEqual(["*-mask.png", "*.psd"]);
  });
});
