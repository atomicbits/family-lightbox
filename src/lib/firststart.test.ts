import { describe, expect, it } from "vitest";
import { firstStartSteps } from "./firststart";
import type { SyncState } from "./sync";

const base: SyncState = {
  phase: "idle", running: false, folder: "", files_done: 0, files_total: 0, bytes_done: 0, bytes_total: 0, summary: null,
  manifests_copied: 0, lists_copied: 0, manifests_changed: [], previews_copied: 0, previews_bytes: 0, failed_count: 0, failed: [], skipped: [], message: null,
};
const states = (s: ReturnType<typeof firstStartSteps>) => s.map((x) => x.state);

describe("the first start", () => {
  it("asks to connect when the NAS isn't", () => {
    const s = firstStartSteps(null, false, "disconnected");
    expect(states(s)).toEqual(["problem", "todo", "todo", "todo"]);
    expect(s[0].detail).toContain("Verbinden");
  });

  it("counts the manifests with a fraction once the total is known", () => {
    const s = firstStartSteps({ ...base, phase: "manifests", running: true, files_done: 120, files_total: 3638 }, false, "connected");
    expect(states(s)).toEqual(["done", "busy", "todo", "todo"]);
    expect(s[1].detail).toBe("120 van 3 638 mappen");
    expect(s[1].fraction).toBeCloseTo(120 / 3638);
    const searching = firstStartSteps({ ...base, phase: "manifests", running: true, folder: "2007" }, false, "connected");
    expect(searching[1].detail).toBe("mappen zoeken… 2007");
  });

  it("builds the index while the previews are counted, then waits for the confirmation", () => {
    const s = firstStartSteps({ ...base, phase: "counting", running: true, files_done: 5000, folder: "2019/07/x" }, true, "connected");
    expect(states(s)).toEqual(["done", "done", "done", "busy", "busy"]);
    expect(s[4].detail).toBe("5 000 gevonden… 2019/07");
    const w = firstStartSteps({ ...base, phase: "awaiting_confirmation" }, false, "connected");
    expect(w[4]).toMatchObject({ label: "Previews ophalen", state: "waiting" });
  });

  it("shows the copying with its bytes, and a failure where it happened", () => {
    const c = firstStartSteps({ ...base, phase: "copying", running: true, files_done: 10, files_total: 100, bytes_done: 1e9, bytes_total: 4e9 }, false, "connected");
    expect(c[4]).toMatchObject({ detail: "10 van 100 · 1,0 van 4,0 GB", fraction: 0.25 });
    const f = firstStartSteps({ ...base, phase: "failed", message: "Library is niet leesbaar" }, false, "connected");
    expect(f[1]).toMatchObject({ state: "problem", detail: "Library is niet leesbaar" });
  });
});
