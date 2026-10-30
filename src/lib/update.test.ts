import { describe, expect, it } from "vitest";
import { checkedText, initial, updateLine, UP_TO_DATE } from "./update";

const offer = { version: "0.2.1", notes: "" };

describe("updateLine", () => {
  it("shows nothing without a newer version", () => {
    expect(updateLine(initial, false)).toBeNull();
  });

  it("offers Bijwerken for a newer version", () => {
    const line = updateLine({ ...initial, offer }, false)!;
    expect(line.text).toBe("Versie 0.2.1 beschikbaar");
    expect(line.button).toMatchObject({ label: "Bijwerken", disabled: false });
  });

  it("waits for a running sync", () => {
    const line = updateLine({ ...initial, offer }, true)!;
    expect(line.button).toMatchObject({ disabled: true, title: "na de synchronisatie" });
    expect(line.text).toBe("Versie 0.2.1 beschikbaar, na de synchronisatie");
  });

  it("shows the download progress, without a button", () => {
    const line = updateLine({ ...initial, offer, installing: true, progress: { done: 1_500_000, total: 3_000_000 } }, false)!;
    expect(line.text).toBe("Bijwerken… 50 %");
    expect(line.button).toBeNull();
    expect(updateLine({ ...initial, offer, installing: true }, false)!.text).toBe("Bijwerken…");
  });

  it("shows a refused signature without offering it again", () => {
    const line = updateLine({ ...initial, installError: "Update geweigerd: de handtekening klopt niet" }, false)!;
    expect(line).toMatchObject({ tone: "red", button: null });
  });

  it("keeps the button after a failed download", () => {
    const line = updateLine({ ...initial, offer, installError: "bijwerken mislukt: netwerk" }, false)!;
    expect(line.text).toBe("bijwerken mislukt: netwerk");
    expect(line.button?.label).toBe("Bijwerken");
  });
});

describe("checkedText", () => {
  it("reports the outcome of a check by hand", () => {
    expect(checkedText(null)).toBe(UP_TO_DATE);
    expect(checkedText(offer)).toBe("Versie 0.2.1 is beschikbaar");
  });
});
