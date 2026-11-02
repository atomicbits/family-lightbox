import { describe, expect, it } from "vitest";
import { NO_IMPORT, baseName, layoutText, patterns, withSource, syncedHint } from "./sources";

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

describe("the hint under a synced folder", () => {
  it("says what a source is, and where the files go", () => {
    expect(syncedHint("", "")).toContain("bv. <jaar>/<maand>/iPhone-photos/");
    expect(syncedHint("Phone-photos", "")).toContain("Geef ook de bron voor de video's");
    expect(syncedHint("", "Phone-videos")).toContain("Geef ook de bron voor de foto's");
    expect(syncedHint("Phone", "Phone")).toBe("Foto's en video's komen samen in <jaar>/<maand>/Phone/.");
    expect(syncedHint("Phone-photos", "Phone-videos")).toBe("Foto's komen in <jaar>/<maand>/Phone-photos/, video's in <jaar>/<maand>/Phone-videos/.");
  });
});
