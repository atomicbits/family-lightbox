import { describe, expect, it } from "vitest";
import { customProblem, customText, exportDoneText, exportPlanText, presetText, shortFolder, type Preset } from "./exporter";

const a4: Preset = { id: "fotoboek-a4", label: "Fotoboek A4", what: "", long: 3508, quality: 0.92, dpi: 300, originals: false };
const orig: Preset = { id: "originelen", label: "Originelen", what: "", long: null, quality: 0, dpi: null, originals: true };

describe("exporting a list (F024/S03)", () => {
  it("names the presets and the plan", () => {
    expect(presetText(a4)).toBe("Fotoboek A4 · 3 508 px");
    expect(presetText(orig)).toBe("Originelen · ongewijzigd");
    expect(exportPlanText({ photos: 602, videos: 33, files: 635, dest: null }, a4)).toBe("602 foto's als JPEG, 33 video's ongewijzigd");
    expect(exportPlanText({ photos: 2, videos: 0, files: 3, dest: null }, orig)).toBe("3 bestanden: de originelen met hun sidecars");
  });
  it("names the folder and the end", () => {
    expect(shortFolder("/Volumes/Fotobibliotheek/Exports/2026/2026-11-aosta-fotoboek-a4")).toBe("2026/2026-11-aosta-fotoboek-a4");
    expect(exportDoneText({ files: 635, dest: "/x/2026/2026-11-aosta", problems: [], error: null })).toBe("635 bestanden geëxporteerd naar 2026/2026-11-aosta");
    expect(exportDoneText({ files: 0, dest: "", problems: [], error: "Exports is niet verbonden" })).toBe("exporteren gestopt: Exports is niet verbonden");
  });
});

describe("an own export setting (F024/S03)", () => {
  it("is checked as the core does, and named", () => {
    expect(customProblem({ long: 3000, quality: 90, dpi: 300 })).toBeNull();
    expect(customProblem({ long: 50, quality: 90, dpi: null })).toContain("lange zijde");
    expect(customProblem({ long: 3000, quality: 0, dpi: null })).toContain("kwaliteit");
    expect(customProblem({ long: 3000, quality: 90, dpi: 5000 })).toContain("dpi");
    expect(customText({ long: 3000, quality: 90, dpi: 300 })).toBe("3 000 px, kwaliteit 90, 300 dpi");
    expect(customText({ long: 3000, quality: 90, dpi: null })).toBe("3 000 px, kwaliteit 90");
  });
});
