import { describe, expect, it } from "vitest";
import type { Access, Attempt, Mount, NasCheck, NasMounts } from "./api";
import { layoutSourceText, locationText, nasSummary, shouldConnect, shouldSync, syncSettled } from "./nas";

const m: Mount = { mount_point: "/Volumes/X", fs_type: "smbfs", server: "raven", user: "lynn", chosen: true };
const all: NasMounts = { library: [m], previews: [m], lists: [m], exports: [m] };
const ok: Access = { mounted: true, readable: true, writable: null, problem: null };
const good: NasCheck = {
  layout_source: "settings",
  layout_problem: null,
  library: ok,
  previews: ok,
  exports: ok,
  lists: { ...ok, writable: true },
  registration: { kind: "already_registered" },
};
const unreachable: Attempt = { share: "lists", name: "Lists", outcome: { kind: "unreachable" }, message: "raven is niet bereikbaar" };

describe("nasSummary", () => {
  it("is connected when all four are mounted and usable", () => {
    expect(nasSummary(all, good, [], false)).toMatchObject({ state: "connected", tone: "green", action: null });
  });

  it("is partial when a share is missing or Lists isn't writable", () => {
    expect(nasSummary({ ...all, previews: [] }, good, [], false)).toMatchObject({ state: "partial", action: "connect" });
    const readOnly = { ...good, lists: { ...ok, writable: false, problem: "niet schrijfbaar: x" } };
    expect(nasSummary(all, readOnly, [], false)).toMatchObject({ state: "partial", tone: "amber" });
  });

  it("is disconnected when nothing is mounted", () => {
    const none = { library: [], previews: [], lists: [], exports: [] };
    expect(nasSummary(none, null, [], false)).toMatchObject({ state: "disconnected", label: "niet verbonden", action: "connect" });
  });

  it("says unreachable after an unreachable attempt, even with old mounts listed", () => {
    expect(nasSummary(all, good, [unreachable], false)).toMatchObject({ state: "unreachable", action: "retry" });
  });

  it("busy wins over everything, and so does not having looked yet", () => {
    expect(nasSummary(all, good, [unreachable], true).state).toBe("busy");
    expect(nasSummary(null, null, [], false).state).toBe("busy");
  });
});

describe("without a NAS set up (F006/S06)", () => {
  it("says so and offers no connecting", () => {
    const none = { host: "", library: [], previews: [], lists: [], exports: [] };
    expect(nasSummary(none, null, [], false)).toMatchObject({ label: "niet ingesteld", action: null });
  });
});

describe("the layout in the settings (F019/S03)", () => {
  it("says where it came from and where each location is", () => {
    expect(layoutSourceText("nas")).toBe("uit familylightbox.json op de NAS");
    expect(layoutSourceText("settings")).toContain("geen familylightbox.json");
    expect(layoutSourceText(null)).toBe("nog niet gelezen");
    expect(locationText({ share: "Photos", path: "Library" })).toBe("Photos / Library");
    expect(locationText({ share: "Lists", path: "" })).toBe("Lists");
  });
});

describe("staying connected (F006/S08)", () => {
  const share = (chosen: boolean) => [{ mount_point: "/Volumes/Photos", server: "raven", user: "lynn", fs_type: "smbfs", chosen }];
  const up = { host: "raven", library: share(true), previews: share(true), lists: share(true), exports: share(true) };
  const down = { host: "raven", library: [], previews: [], lists: [], exports: [] };
  it("connects quietly only while a share is missing", () => {
    expect(shouldConnect(null, false)).toBe(false);
    expect(shouldConnect(down, false)).toBe(true);
    expect(shouldConnect(down, true)).toBe(false);
    expect(shouldConnect(up, false)).toBe(false);
  });
  it("syncs when Library is there and no sync settled it", () => {
    expect(shouldSync(up, false, false)).toBe(true);
    expect(shouldSync(up, true, false)).toBe(false);
    expect(shouldSync(up, false, true)).toBe(false);
    expect(shouldSync(down, false, false)).toBe(false);
  });
  it("is settled by done, the confirmation or a cancel, not by a failure", () => {
    expect(["done", "awaiting_confirmation", "cancelled", "failed"].map((phase) => syncSettled({ running: false, phase }))).toEqual([true, true, true, false]);
    expect(syncSettled({ running: true, phase: "done" })).toBe(false);
  });
});
