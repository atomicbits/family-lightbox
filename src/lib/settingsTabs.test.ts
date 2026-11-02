import { describe, expect, it } from "vitest";
import { changedTabs, readTab } from "./settingsTabs";
import type { Settings } from "./api";

const saved: Settings = {
  nas_host: "raven",
  home_share: "Photos",
  shares: { library: "Library", previews: "Previews", lists: "Lists", exports: "Exports" },
  local: { library: "/l", previews: "/p", lists: "/s" },
  import: { drop_folders: [], synced: [], sources: {} },
};

describe("the settings' tabs", () => {
  it("know which tab a change belongs to", () => {
    expect([...changedTabs(saved, saved)]).toEqual([]);
    expect([...changedTabs({ ...saved, nas_host: "nas" }, saved)]).toEqual(["nas"]);
    expect([...changedTabs({ ...saved, home_share: "" }, saved)]).toEqual(["nas"]);
    expect([...changedTabs({ ...saved, import: { ...saved.import, drop_folders: ["/x"] } }, saved)]).toEqual(["import"]);
    const both = changedTabs({ ...saved, nas_host: "nas", import: { ...saved.import, synced: [{ path: "/d", photos: "", videos: "" }] } }, saved);
    expect([...both].sort()).toEqual(["import", "nas"]);
  });

  it("read the remembered tab safely", () => {
    expect(readTab("import")).toBe("import");
    expect(readTab("library")).toBe("library");
    expect(readTab("gone")).toBe("general");
    expect(readTab(null)).toBe("general");
  });
});
