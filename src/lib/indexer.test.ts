import { describe, expect, it } from "vitest";
import { indexLine, outcomeText, reselect, type BuildOutcome, type IndexState } from "./indexer";
import type { Item } from "./api";

const outcome: BuildOutcome = {
  manifests: 364, items: 124837, photos: 104751, videos: 19821, sidecars: 10, other: 255,
  local: 30004, previews: 38698, broken: [], millis: 1336,
};
const idle: IndexState = { running: false, outcome, error: null, builds: 1 };

describe("index line in the header", () => {
  it("shows the build only when someone waits for it", () => {
    expect(indexLine({ ...idle, running: true }, true)).toEqual({ text: "index bijwerken…", tone: "muted" });
    expect(indexLine({ ...idle, running: true }, false)).toBeNull();
    expect(indexLine(idle, false)).toBeNull();
    expect(indexLine(null, true)).toBeNull();
  });

  it("names broken manifests by folder, with the reasons in the title", () => {
    const l = indexLine({ ...idle, outcome: { ...outcome, broken: ["2016/12: expected value at line 1"] } }, false);
    expect(l?.text).toBe("index: 1 manifest onleesbaar: 2016/12");
    expect(l?.tone).toBe("amber");
    expect(l?.title).toContain("expected value");
    const two = indexLine({ ...idle, outcome: { ...outcome, broken: ["2016/11: x", "2016/12: y"] } }, false);
    expect(two?.text).toBe("index: 2 manifesten onleesbaar: 2016/11, 2016/12");
  });

  it("shows a failure in amber", () => {
    const l = indexLine({ ...idle, error: "werkkopie niet gevonden: /x" }, false);
    expect(l).toEqual({ text: "index: werkkopie niet gevonden: /x", tone: "amber" });
  });

  it("sums up a build for the settings panel", () => {
    expect(outcomeText(outcome)).toBe("124 837 items uit 364 manifesten · 1,3 s");
  });
});

describe("selection after a rebuild", () => {
  const item = (path: string) => ({ path }) as Item;
  const sections = [
    { month: "2026/08", count: 2 },
    { month: "2026/07", count: 3 },
  ];

  it("follows the item by path when it moved within its month", () => {
    const items = [item("a"), item("new"), item("b")];
    expect(reselect({ month: "2026/07", index: 1, path: "b" }, sections, items)).toEqual({ section: 1, index: 2 });
  });

  it("keeps the position when the item is gone, clamped to the month", () => {
    expect(reselect({ month: "2026/07", index: 1, path: "gone" }, sections, [item("a")])).toEqual({ section: 1, index: 1 });
    expect(reselect({ month: "2026/07", index: 9, path: null }, sections, undefined)).toEqual({ section: 1, index: 2 });
  });

  it("finds the month at its new place, and drops a month that is gone", () => {
    const shifted = [{ month: "2026/09", count: 1 }, ...sections];
    expect(reselect({ month: "2026/08", index: 0, path: null }, shifted, undefined)).toEqual({ section: 1, index: 0 });
    expect(reselect({ month: "2025/01", index: 0, path: null }, sections, undefined)).toBeNull();
    expect(reselect(null, sections, undefined)).toBeNull();
  });
});
