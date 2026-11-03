import { describe, expect, it } from "vitest";
import { importActivity } from "./importplans";

const pr = (bytes_done: number, bytes_total: number) => ({ files_done: 0, files_total: 4, bytes_done, bytes_total });

describe("the footer's Importeren (F018/S05)", () => {
  it("is quiet without a plan or an import", () => {
    expect(importActivity({}, null, null, null)).toBeNull();
  });
  it("shows a plan being made, moving before its size is known", () => {
    expect(importActivity({ "/d/Max2": true }, null, null, null)).toEqual({ text: "plan Max2: bestanden zoeken…", done: null });
    expect(importActivity({ "/d/Max2": pr(5, 20), "/s/iPhoneSync": true }, null, null, null)).toEqual({ text: "plan Max2: 25% (+1)", done: 0.25 });
  });
  it("shows the import running before any plan", () => {
    const a = importActivity({ "/d/Max2": true }, { name: "Hero13" }, { step: "nas", done: 2, total: 5 }, null);
    expect(a).toEqual({ text: "importeren Hero13: naar de NAS 2/5", done: 0.4 });
  });
  it("shows the NAS step by its bytes", () => {
    const a = importActivity({}, { name: "Lynn" }, { step: "nas", done: 10, total: 100, bytes_done: 3e9, bytes_total: 12e9 }, null);
    expect(a?.done).toBe(0.25);
    expect(a?.text).toMatch(/^importeren Lynn: naar de NAS 10\/100 · 3,0\/12,0 GB$/);
  });
});
