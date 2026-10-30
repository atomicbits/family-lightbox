import { describe, expect, it } from "vitest";
import { confirmText, count, estimate, gb, syncLine, type SyncState } from "./sync";

const base: SyncState = {
  phase: "idle", running: false, folder: "", files_done: 0, files_total: 0, bytes_done: 0, bytes_total: 0,
  summary: null, manifests_copied: 0, lists_copied: 0, manifests_changed: [], previews_copied: 0, previews_bytes: 0, failed_count: 0, failed: [], skipped: [], message: null,
};

describe("sync line", () => {
  it("shows each running phase", () => {
    expect(syncLine({ ...base, phase: "manifests", files_done: 120 })?.text).toBe("manifesten 120");
    expect(syncLine({ ...base, phase: "manifests", files_done: 1, files_total: 2 })?.text).toBe("manifesten 1/2");
    expect(syncLine({ ...base, phase: "counting", folder: "2023/04/EOSR/photos" })?.text).toBe("previews tellen… 2023/04");
    expect(syncLine({ ...base, phase: "lists" })?.text).toBe("lijsten…");
    const copying = syncLine({ ...base, phase: "copying", files_done: 1234, files_total: 13076, bytes_done: 1.2e9, bytes_total: 4.99e9 });
    expect(copying?.text).toBe("previews 1 234/13 076 · 1,2/5,0 GB");
    expect(copying?.cancellable).toBe(true);
  });

  it("asks for confirmation, shows failures in amber, and ends quietly when nothing changed", () => {
    expect(syncLine({ ...base, phase: "awaiting_confirmation" })).toMatchObject({ text: "Previews ophalen…", confirm: true });
    expect(syncLine({ ...base, phase: "failed", message: "nodig 29,1 GB, vrij 35,2 GB, reserve 20 GB: 13,9 GB tekort" }))
      .toMatchObject({ tone: "amber", text: expect.stringContaining("tekort") });
    expect(syncLine({ ...base, phase: "done" })).toBeNull();
    expect(syncLine({ ...base, phase: "idle" })).toBeNull();
  });

  it("says what was brought over, without pointing at the CLI", () => {
    const l = syncLine({ ...base, phase: "done", manifests_copied: 2, previews_copied: 40 });
    expect(l?.text).toBe("bijgewerkt: 2 manifesten, 40 previews");
    expect(JSON.stringify(l)).not.toContain("./media");
    expect(syncLine({ ...base, phase: "done", manifests_copied: 1 })?.text).toBe("bijgewerkt: 1 manifest");
  });
});

describe("confirmation dialog", () => {
  it("gives the size and an estimate at 6 MB/s", () => {
    expect(gb(29_100_000_000)).toBe("29,1");
    expect(count(77396)).toBe("77 396");
    expect(estimate(29_100_000_000)).toBe("ongeveer 81 minuten");
    expect(estimate(10_000_000)).toBe("minder dan een minuut");
    expect(confirmText({ copy_files: 77396, copy_bytes: 29_100_000_000, kept: 0, skipped: [], free: null, problem: null, needs_confirmation: true }))
      .toBe("29,1 GB previews ophalen (77 396 bestanden)? Dat duurt ongeveer 81 minuten.");
  });
});

describe("files skipped in a sync", () => {
  it("are counted in the done line, which fades like any other", () => {
    expect(syncLine({ ...base, phase: "done", previews_copied: 120, failed_count: 1, failed: ["x: timed out"] })?.text).toBe("bijgewerkt: 120 previews · 1 overgeslagen");
    expect(syncLine({ ...base, phase: "done", failed_count: 2 })?.text).toBe("2 overgeslagen");
  });
});
