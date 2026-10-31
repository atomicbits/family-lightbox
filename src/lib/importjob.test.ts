import { describe, expect, it } from "vitest";
import { doneText, importLine, pendingText } from "./importjob";

describe("a running import (F018/S05)", () => {
  it("says what it is doing", () => {
    expect(importLine("Hero13", { step: "nas", done: 2, total: 5 }, null)).toBe("importeren Hero13: naar de NAS 2/5");
    expect(importLine("Hero13", null, { files_done: 10, files_total: 153, bytes_done: 0, bytes_total: 0 })).toBe("importeren Hero13: bestanden lezen 10/153");
    expect(importLine("Hero13", null, null)).toBe("importeren Hero13…");
  });
  it("says how it ended, and where an unfinished one stopped", () => {
    const d = { id: "1", name: "Hero13", step: "done" as const, copied: 82, duplicates: 71, failed: 1, problems: [], error: null };
    expect(doneText(d)).toBe("Hero13: 82 geïmporteerd · 71 al in de bibliotheek · 1 mislukt");
    expect(doneText({ ...d, error: "de NAS is niet verbonden" })).toBe("Hero13: gestopt (de NAS is niet verbonden)");
    expect(pendingText({ id: "1", name: "Max2", source: "/x", step: "nas", created: "" })).toBe("Max2, gestopt bij: naar de NAS");
  });
});
