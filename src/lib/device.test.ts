import { describe, expect, it } from "vitest";
import { anyMissing, mountLabel } from "./api";
import { deviceNameProblem, hostProblem, ownerProblem, proposeDeviceName, shareProblem } from "./device";

describe("device identity", () => {
  it("proposes mbp-<owner>", () => {
    expect(proposeDeviceName("lynn")).toBe("mbp-lynn");
    expect(proposeDeviceName(" Lynn ")).toBe("mbp-lynn");
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
