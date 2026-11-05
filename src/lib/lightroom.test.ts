import { describe, expect, it } from "vitest";
import { pluginText } from "./lightroom";

describe("the Lightroom plug-in in the settings (F023/S03)", () => {
  it("says its state", () => {
    expect(pluginText("missing")).toBe("de plugin is nog niet geïnstalleerd");
    expect(pluginText("current")).toContain("geïnstalleerd");
    expect(pluginText("different")).toContain("werk ze bij");
  });
});
