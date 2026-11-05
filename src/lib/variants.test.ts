import { describe, expect, it } from "vitest";
import { allVariants, chosenVariant, variantLabel } from "./variants";

describe("Lightroom's versions (F025/S04)", () => {
  it("chooses as the contract says", () => {
    expect(chosenVariant(undefined, undefined)).toBe("");
    expect(chosenVariant(["lr-zw", "lr"], undefined)).toBe("lr-zw");
    expect(chosenVariant(["lr-zw", "lr"], "lr")).toBe("lr");
    expect(chosenVariant(["lr-zw", "lr"], "")).toBe("");
    expect(chosenVariant(["lr"], "lr-weg")).toBe("lr");
  });
  it("names them", () => {
    expect(variantLabel("")).toBe("Origineel");
    expect(variantLabel("lr")).toBe("Bewerkt");
    expect(variantLabel("lr-zwart-wit")).toBe("Zwart wit");
    expect(allVariants(["lr"])).toEqual(["", "lr"]);
  });
});
