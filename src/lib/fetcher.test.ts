import { describe, expect, it } from "vitest";
import { fetchDoneText, fetchPlanText, fetchingText, type FetchPlan } from "./fetcher";

const plan: FetchPlan = { files: [{ path: "a", size: 1, sidecar: false }, { path: "a.xmp", size: 0, sidecar: true }], bytes: 1.2e9, here: 3, conflicts: [], unknown: [], fits: true };

describe("fetching originals (F009/S03)", () => {
  it("says what the plan takes", () => {
    expect(fetchPlanText(plan)).toBe("1 origineel ophalen (1,2 GB) · 3 staan er al");
    expect(fetchPlanText({ ...plan, files: [], here: 1 })).toBe("niets op te halen · 1 staat er al");
    expect(fetchPlanText({ ...plan, conflicts: ["x"], unknown: ["y"] })).toContain("1 met een andere grootte hier");
  });
  it("names the progress and the end", () => {
    expect(fetchingText({ done: 12, total: 40, bytes_done: 0, bytes_total: 0 })).toBe("originelen ophalen 12/40");
    expect(fetchDoneText({ fetched: 40, problems: [], error: null })).toBe("40 originelen opgehaald");
    expect(fetchDoneText({ fetched: 40, problems: ["x"], error: null })).toBe("40 originelen opgehaald · 1 niet (zie de meldingen)");
    expect(fetchDoneText({ fetched: 0, problems: [], error: "de NAS is weg" })).toBe("originelen ophalen gestopt: de NAS is weg");
  });
});
