import { describe, expect, it } from "vitest";
import { actionText, datesText, monthsText, planSummary, progressText, type SourcePlan } from "./importplan";

const plan: SourcePlan = {
  name: "Hero13", path: "/x/Hero13", files: [], copy_files: 82, copy_bytes: 93.04e9, duplicates: 71, conflicts: 1, excluded: 1,
  months: { "2025/03": 2, "2026/08": 59 }, not_from_file: { mtime: 3, name: 1 }, problems: [],
};

describe("the import view (F018/S04)", () => {
  it("sums up a plan", () => {
    expect(planSummary(plan)).toBe("82 nieuw (93,0 GB) · 71 al in de bibliotheek · 1 naamconflict");
    expect(planSummary({ ...plan, copy_files: 0, duplicates: 0, conflicts: 0 })).toBe("niets nieuw");
  });
  it("says the months, the dates and the files", () => {
    expect(monthsText(plan.months)).toBe("2025/03 (2) · 2026/08 (59)");
    expect(datesText(plan.not_from_file)).toBe("datum niet uit het bestand: 3 uit de wijzigingstijd, 1 uit de bestandsnaam");
    expect(datesText({})).toBe("");
    expect(actionText({ what: "duplicate", of: "2019/01/X/a.jpg" })).toBe("al in de bibliotheek: 2019/01/X/a.jpg");
    expect(progressText({ files_done: 1234, files_total: 5000, bytes_done: 1.2e9, bytes_total: 93e9 })).toBe("1 234 / 5 000 bestanden · 1,2 / 93,0 GB");
  });
});
