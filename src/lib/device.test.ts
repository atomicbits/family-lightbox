import { describe, expect, it } from "vitest";
import { accessLabel, anyMissing, mountLabel, registrationLabel } from "./api";
import { deviceNameProblem, freeText, hostProblem, ownerProblem, parentOf, pathProblem, people, setUpText, shareProblem, takenBy } from "./device";

describe("the local folder (F006/S07)", () => {
  it("says how much room there is", () => {
    expect(freeText(812.4e9)).toBe("812 GB vrij");
    expect(freeText(1.23e12)).toBe("1,2 TB vrij");
    expect(freeText(null)).toBe("");
  });
  it("finds the folder the three share", () => {
    expect(parentOf({ library: "/a/FL/Library", previews: "/a/FL/Previews", lists: "/a/FL/Lists" })).toBe("/a/FL");
    expect(parentOf({ library: "/a/LibraryLocal", previews: "/b/Previews", lists: "/a/Lists" })).toBeNull();
  });
});

describe("device identity", () => {
  it("knows whose device name is taken and who the NAS knows (F006/S06)", () => {
    const known = [{ name: "mbp-anna", owner: "anna" }, { name: "imac", owner: "anna" }, { name: "mbp-jan", owner: "jan" }];
    expect(takenBy("mbp-jan", "anna", known)).toBe("jan");
    expect(takenBy("mbp-anna", "anna", known)).toBeNull();
    expect(takenBy("new", "anna", known)).toBeNull();
    expect(people(known)).toEqual([{ owner: "anna", devices: ["mbp-anna", "imac"] }, { owner: "jan", devices: ["mbp-jan"] }]);
  });

  it("checks device names like the Rust rules", () => {
    for (const good of ["mbp-lynn", "mbp-2", "ab", "a".repeat(32)]) expect(deviceNameProblem(good)).toBeNull();
    for (const bad of ["Mbp-lynn", "m", "-x", "mbp_lynn", "2mbp", "mbp lynn", "a".repeat(33)])
      expect(deviceNameProblem(bad)).not.toBeNull();
  });

  it("checks owners like the Rust rules", () => {
    for (const good of ["lynn", "peter", "karel2"]) expect(ownerProblem(good)).toBeNull();
    for (const bad of ["Lynn", "l", "lynn-r", "", "2lynn"]) expect(ownerProblem(bad)).not.toBeNull();
  });
});

describe("settings", () => {
  it("checks the NAS host and share names like the Rust rules", () => {
    expect(hostProblem("raven")).toBeNull();
    expect(hostProblem("raven.local")).toBeNull();
    expect(hostProblem("smb://raven")).toContain("smb://");
    for (const bad of ["", "raven/Library", "ra ven"]) expect(hostProblem(bad)).not.toBeNull();
    expect(shareProblem("Library")).toBeNull();
    expect(shareProblem("")).not.toBeNull();
    expect(shareProblem("Lists/sub")).not.toBeNull();
  });
});

describe("mounts", () => {
  it("labels the chosen mount, or null when there is none", () => {
    const m = (fs_type: string, mount_point: string, chosen: boolean) => ({ mount_point, fs_type, server: "raven", user: "peter", chosen });
    expect(mountLabel([m("afpfs", "/Volumes/Library", false), m("smbfs", "/Volumes/Library-1", true)])).toBe("/Volumes/Library-1 · SMB");
    expect(mountLabel([m("afpfs", "/Volumes/Library", true)])).toBe("/Volumes/Library · AFP");
    expect(mountLabel([])).toBeNull();
  });
});

describe("connect", () => {
  it("knows when something is still missing", () => {
    const m = { mount_point: "/Volumes/X", fs_type: "smbfs", server: "raven", user: null, chosen: true };
    expect(anyMissing({ library: [m], previews: [m], lists: [m], exports: [m] })).toBe(false);
    expect(anyMissing({ library: [m], previews: [], lists: [m], exports: [m] })).toBe(true);
  });
});

describe("access and registration", () => {
  it("labels access per share and the registration outcome", () => {
    expect(accessLabel({ mounted: false, readable: false, writable: null, problem: null })).toBeNull();
    expect(accessLabel({ mounted: true, readable: true, writable: null, problem: null })).toBe("leesbaar");
    expect(accessLabel({ mounted: true, readable: true, writable: true, problem: null })).toBe("leesbaar en schrijfbaar");
    expect(accessLabel({ mounted: true, readable: true, writable: false, problem: "niet schrijfbaar: x" })).toBe("niet schrijfbaar: x");
    expect(registrationLabel({ kind: "already_registered" })).toBe("geregistreerd op de NAS");
    expect(registrationLabel({ kind: "refused", message: "toestelnaam 'mbp-lynn' is al van karel" })).toContain("al van karel");
  });
});

describe("a new library (F019/S02)", () => {
  it("says what it makes and what it uses", () => {
    expect(setUpText("Fotos", [])).toBe("Maakt in 'Fotos' de mappen Library, Previews, Lists en Exports, en schrijft er familylightbox.json bij. Er wordt niets overschreven.");
    expect(setUpText("Fotos", ["Library"])).toBe("Maakt in 'Fotos' de mappen Previews, Lists en Exports en gebruikt Library die er al is, en schrijft er familylightbox.json bij. Er wordt niets overschreven.");
    expect(setUpText("Fotos", ["Library", "Previews", "Lists", "Exports"])).toBe("Gebruikt Library, Previews, Lists en Exports die er al zijn, en schrijft er familylightbox.json bij. Er wordt niets overschreven.");
  });
  it("keeps a path inside its share", () => {
    expect(pathProblem("")).toBeNull();
    expect(pathProblem("FamilyLightbox/Library")).toBeNull();
    for (const bad of ["/x", "../x", "a/../b", "a\\b"]) expect(pathProblem(bad)).not.toBeNull();
  });
});
