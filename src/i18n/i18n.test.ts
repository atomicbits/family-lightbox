import { describe, expect, it } from "vitest";
import { count, decimal, pickLanguage, setLanguage, t } from ".";
import { monthLabel } from "../lib/months";
import { freeText, setUpText } from "../lib/device";

describe("languages (F011/S01)", () => {
  it("follows the Mac unless a setting says otherwise", () => {
    expect(pickLanguage("", ["nl-BE", "en-US"])).toBe("nl");
    expect(pickLanguage(undefined, ["nl"])).toBe("nl");
    expect(pickLanguage("", ["en-GB", "nl-BE"])).toBe("en");
    expect(pickLanguage("", [])).toBe("en");
    expect(pickLanguage("en", ["nl-BE"])).toBe("en");
    expect(pickLanguage("nl", ["fr-FR"])).toBe("nl");
  });
  it("numbers, months and texts in English", () => {
    setLanguage("en");
    expect(count(124837)).toBe("124,837");
    expect(decimal(1.25)).toBe("1.3");
    expect(monthLabel("2026/07")).toBe("July");
    expect(freeText(1.2e12)).toBe("1.2 TB free");
    expect(setUpText("Photos", ["Library"])).toBe("Makes the folders Previews, Lists and Exports in 'Photos' and uses Library, which is there already, and writes familylightbox.json next to them. Nothing is overwritten.");
    expect(t().welcome.title).toBe("Welcome to FamilyLightbox");
  });
  it("and in Dutch", () => {
    expect(count(124837)).toBe("124 837");
    expect(decimal(1.25)).toBe("1,3");
    expect(monthLabel("2026/07")).toBe("juli");
    expect(freeText(812e9)).toBe("812 GB vrij");
  });
});

describe("the four parts in English (F011/S02)", () => {
  it("builds their texts", async () => {
    setLanguage("en");
    const { fetchPlanText } = await import("../lib/fetcher");
    const { photosText } = await import("../lib/lists");
    const { countTitle } = await import("../lib/hidden");
    const { checkedText } = await import("../lib/update");
    const { tabs } = await import("../lib/settingsTabs");
    const out = [
      fetchPlanText({ files: [{ path: "a", size: 1, sidecar: false }], bytes: 1.2e9, here: 3, conflicts: [], unknown: [], fits: true }),
      photosText(1),
      photosText(1234),
      countTitle(10, 2),
      checkedText(null),
      tabs().map((x) => x.label).join(" "),
    ];
    for (const s of out) expect(s).not.toMatch(/\b(foto|originelen|verborgen|staan|Algemeen|bijgewerkt)\b/);
    expect(out[2]).toContain("1,234");
  });
});
