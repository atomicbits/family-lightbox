// Device and settings rules for instant feedback in the UI. Rust checks again on save and is the
// authority (src-tauri/src/config.rs); keep both in step. Texts from the catalog (F011).
import { decimal, t } from "../i18n";

const lower = (c: string) => c >= "a" && c <= "z";
const digit = (c: string) => c >= "0" && c <= "9";

/** `^[a-z][a-z0-9-]{1,31}$` (data contract 1); null when fine, otherwise the reason. */
export function deviceNameProblem(name: string): string | null {
  const ok = name.length >= 2 && name.length <= 32 && lower(name[0]) && [...name].every((c) => lower(c) || digit(c) || c === "-");
  return ok ? null : t().device.nameRule;
}

/** `^[a-z][a-z0-9]{1,31}$`: a short lowercase name such as `anna`. */
export function ownerProblem(owner: string): string | null {
  const ok = owner.length >= 2 && owner.length <= 32 && lower(owner[0]) && [...owner].every((c) => lower(c) || digit(c));
  return ok ? null : t().device.ownerRule;
}

/** The device name is taken on the NAS by someone else: their name, or null. */
export function takenBy(name: string, owner: string, known: { name: string; owner: string }[]): string | null {
  return known.find((k) => k.name === name && k.owner !== owner)?.owner ?? null;
}

/** The people the NAS knows, each with their devices, for the welcome. */
export function people(known: { name: string; owner: string }[]): { owner: string; devices: string[] }[] {
  const by = new Map<string, string[]>();
  for (const k of known) by.set(k.owner, [...(by.get(k.owner) ?? []), k.name]);
  return [...by].map(([owner, devices]) => ({ owner, devices })).sort((a, b) => a.owner.localeCompare(b.owner));
}

/** "812 GB free", "1.2 TB free", or "" when unknown. */
export function freeText(bytes: number | null): string {
  if (bytes === null) return "";
  if (bytes >= 1e12) return t().device.freeTB(decimal(bytes / 1e12));
  return t().device.freeGB(Math.floor(bytes / 1e9));
}

/** The folder that holds the three local folders, if they share one. */
export function parentOf(l: { library: string; previews: string; lists: string }): string | null {
  const up = (p: string) => p.replace(/\/[^/]+\/?$/, "");
  const a = up(l.library);
  return a && a === up(l.previews) && a === up(l.lists) ? a : null;
}

/** Same rule as `host_problem` in Rust. */
export function hostProblem(host: string): string | null {
  if (host === "") return t().device.hostEmpty;
  const scheme = host.indexOf("://");
  if (scheme >= 0) return t().device.hostScheme(host.slice(0, scheme + 3));
  if (/[\s/]/.test(host)) return t().device.hostChars;
  return null;
}

/** Same rule as `share_problem` in Rust. */
export function shareProblem(share: string): string | null {
  if (share === "") return t().device.shareEmpty;
  if (share.includes("/")) return t().device.shareSlash(share);
  return null;
}

/** A folder inside a share (F019): relative, with '/', not leaving it. As `location_problem` in layout.rs. */
export function pathProblem(path: string): string | null {
  if (path.startsWith("/") || path.includes("\\") || path.split("/").includes("..")) return t().device.pathInside(path);
  return null;
}

/** What setting up a new library in `share` will do, given the folders that are there (F019/S02). */
export function setUpText(share: string, existing: string[]): string {
  const all = ["Library", "Previews", "Lists", "Exports"];
  return t().device.setUp(share, all.filter((f) => !existing.includes(f)), existing);
}
