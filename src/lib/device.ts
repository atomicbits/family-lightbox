// Device and settings rules for instant feedback in the UI. Rust checks again on save and is the
// authority (src-tauri/src/config.rs); keep both in step.

/** Proposed device name for an owner: `mbp-<owner>`. */
export function proposeDeviceName(owner: string): string {
  return `mbp-${owner.trim().toLowerCase()}`;
}

const lower = (c: string) => c >= "a" && c <= "z";
const digit = (c: string) => c >= "0" && c <= "9";

/** `^[a-z][a-z0-9-]{1,31}$` (data contract 1); null when fine, otherwise the reason. */
export function deviceNameProblem(name: string): string | null {
  const ok = name.length >= 2 && name.length <= 32 && lower(name[0]) && [...name].every((c) => lower(c) || digit(c) || c === "-");
  return ok ? null : "2 tot 32 tekens: kleine letters, cijfers en '-', beginnend met een letter";
}

/** `^[a-z][a-z0-9]{1,31}$`: a short lowercase name such as `lynn`. */
export function ownerProblem(owner: string): string | null {
  const ok = owner.length >= 2 && owner.length <= 32 && lower(owner[0]) && [...owner].every((c) => lower(c) || digit(c));
  return ok ? null : "2 tot 32 tekens: kleine letters en cijfers, beginnend met een letter";
}

/** Same rule as `host_problem` in Rust. */
export function hostProblem(host: string): string | null {
  if (host === "") return "de naam van de NAS is leeg";
  const scheme = host.indexOf("://");
  if (scheme >= 0) return `geef alleen de naam van de NAS, zonder '${host.slice(0, scheme + 3)}'`;
  if (/[\s/]/.test(host)) return "de naam van de NAS mag geen '/' of spaties bevatten";
  return null;
}

/** Same rule as `share_problem` in Rust. */
export function shareProblem(share: string): string | null {
  if (share === "") return "de naam van een gedeelde map is leeg";
  if (share.includes("/")) return `'${share}': een gedeelde map mag geen '/' bevatten`;
  return null;
}
