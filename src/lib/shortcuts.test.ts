import { describe, expect, it } from "vitest";
import { isShortcutsKey, modKey, shortcuts } from "./shortcuts";

describe("shortcuts", () => {
  it("uses ⌘ on a Mac and Ctrl elsewhere", () => {
    expect(modKey("MacIntel")).toBe("⌘");
    expect(modKey("Win32")).toBe("Ctrl");
    const groups = shortcuts("Ctrl");
    expect(groups[groups.length - 1].items.map((i) => i.keys)).toEqual(["CtrlZ", "Ctrl?"]);
  });

  it("opens with ⌘? or Ctrl+?, not with ? alone", () => {
    const k = (key: string, mods: Partial<{ metaKey: boolean; ctrlKey: boolean; shiftKey: boolean }>) =>
      isShortcutsKey({ key, metaKey: false, ctrlKey: false, shiftKey: false, ...mods });
    expect(k("?", { metaKey: true, shiftKey: true })).toBe(true);
    expect(k("/", { ctrlKey: true, shiftKey: true })).toBe(true);
    expect(k("?", { shiftKey: true })).toBe(false);
    expect(k("/", { metaKey: true })).toBe(false);
  });
});
