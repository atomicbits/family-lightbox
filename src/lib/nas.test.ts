import { describe, expect, it } from "vitest";
import type { Access, Attempt, Mount, NasCheck, NasMounts } from "./api";
import { nasSummary } from "./nas";

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
